//! Höhere Aktionen für die GUI.

use anyhow::{bail, Context, Result};
use serde::Serialize;

use crate::geraet::konfig::{Geraet, Konfiguration};
use crate::geraet::protokoll::{hex, Maus};
use crate::geraet::speicher::{self, HW_PROFIL};

/// Mitgelieferte Beispielkonfiguration
const BEISPIEL: &str = include_str!("konfiguration.toml");

pub fn beispiel() -> Konfiguration {
    toml::from_str(BEISPIEL).expect("Beispielkonfiguration ist gültig")
}

#[derive(Debug, Clone, Serialize)]
pub struct GeraeteStatus {
    /// Profil 1–3, oder `None` wenn das verborgene Hardware-Profil aktiv ist
    pub profil: Option<u8>,
    pub hardware_profil: u8,
    pub makro_ende: u16,
    pub roh: String,
}

pub fn oeffnen(g: &Geraet, trocken: bool, ausfuehrlich: bool) -> Result<Maus> {
    let mut m = Maus::oeffnen(g.vendor_id, g.product_id, g.bericht_laenge, trocken)?;
    m.ausfuehrlich |= ausfuehrlich;
    Ok(m)
}

pub fn status(g: &Geraet) -> Result<GeraeteStatus> {
    let s = oeffnen(g, false, false)?.status()?;
    Ok(GeraeteStatus {
        profil: HW_PROFIL.iter().position(|&p| p == s.aktives_profil).map(|i| i as u8 + 1),
        hardware_profil: s.aktives_profil,
        makro_ende: s.makro_ende,
        roh: hex(&s.roh),
    })
}

/// Überträgt die komplette Konfiguration. `meldung` erhält Fortschrittstexte.
pub fn anwenden(
    k: &Konfiguration,
    trocken: bool,
    ausfuehrlich: bool,
    meldung: &mut dyn FnMut(&str),
) -> Result<()> {
    k.pruefen()?;
    let geraet = oeffnen(&k.geraet, trocken, ausfuehrlich)?;
    let flags = geraet.byte_lesen(speicher::ADR_GERAETEFLAGS).with_context(|| crate::tr!("Geräteflags lesen", "Reading device flags"))?;
    let abbild = speicher::abbild_erstellen(k, flags)?;
    let makros = speicher::makro_bloecke(k, &abbild)?;

    meldung(&crate::tr!("Übertrage Profil-Auswahl …", "Transferring profile selection …"));
    geraet.block_schreiben(speicher::ADR_PROFIL, &abbild[1..2])?;
    meldung(&crate::tr!("Übertrage Grundeinstellungen und Tastenbelegung …", "Transferring settings and key assignments …"));
    let mut a = speicher::BLOCK_START;
    while a <= speicher::BLOCK_ENDE {
        geraet.block_schreiben(a, &abbild[a as usize..a as usize + 8])?;
        a += 8;
    }
    meldung(&crate::tr!("Übertrage Tastentabelle und Tastendaten ({} Blöcke) …", "Transferring key table and key data ({} blocks) …", makros.len()));
    for (adr, daten) in &makros {
        geraet.block_schreiben(*adr, daten)?;
    }
    geraet.speichern()?;
    meldung(&crate::tr!("Fertig. Aktives Profil: {}", "Done. Active profile: {}", k.allgemein.aktives_profil));
    Ok(())
}

/// Wechselt nur das aktive Profil (1–3).
pub fn profil(g: &Geraet, nummer: u8) -> Result<()> {
    if !(1..=3).contains(&nummer) {
        bail!(crate::tr!("Profilnummer muss 1–3 sein", "Profile number must be 1–3"));
    }
    let p = HW_PROFIL[(nummer - 1) as usize];
    let geraet = oeffnen(g, false, false)?;
    let flags = geraet.byte_lesen(speicher::ADR_GERAETEFLAGS)?;
    geraet.block_schreiben(speicher::ADR_PROFIL, &[p << 5 | 0x03])?;
    geraet.block_schreiben(speicher::ADR_GERAETEFLAGS, &[(flags & !3) | p])?;
    geraet.speichern()
}

/// Liest Grundeinstellungen und Tastentabelle. `fortschritt(gelesen, gesamt)`.
pub fn auslesen(
    g: &Geraet,
    tasten: &[crate::geraet::konfig::Taste],
    ausfuehrlich: bool,
    fortschritt: &mut dyn FnMut(usize, usize),
) -> Result<Konfiguration> {
    let geraet = oeffnen(g, false, ausfuehrlich)?;
    let gesamt = speicher::GROESSE + speicher::TABELLE_GROESSE;
    let mut m = [0u8; speicher::GROESSE];
    for (a, b) in m.iter_mut().enumerate() {
        fortschritt(a, gesamt);
        *b = geraet.byte_lesen(a as u16)?;
    }
    let mut tabelle = vec![0u8; speicher::TABELLE_GROESSE];
    for (i, b) in tabelle.iter_mut().enumerate() {
        fortschritt(speicher::GROESSE + i, gesamt);
        *b = geraet.byte_lesen(speicher::TABELLE_START + i as u16)?;
    }
    fortschritt(gesamt, gesamt);
    let mut konfiguration = speicher::dekodieren(&m, &tabelle, tasten);
    // Profilnamen und Geräteeinstellungen bleiben wie übergeben
    konfiguration.geraet = Geraet { ..*g };
    Ok(konfiguration)
}
