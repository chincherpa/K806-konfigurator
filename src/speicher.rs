//! Speicherabbild des Geräts aus der Konfiguration erzeugen bzw. zurückübersetzen.
//!
//! Aufbau siehe `doku/PROTOKOLL.md`.

use std::collections::BTreeMap;

use anyhow::{bail, Result};

use crate::funktionen::{self, Funktion, CODE_KOMBI, CODE_MAKRO, CODE_MEDIEN, CODE_TASTE};
use crate::konfig::*;

pub const GROESSE: usize = 0x80;
/// Erster und letzter 8-Byte-Block der Grundeinstellungen
pub const BLOCK_START: u16 = 0x08;
pub const BLOCK_ENDE: u16 = 0x78;
/// Byte mit Profil-Auswahl
pub const ADR_PROFIL: u16 = 0x01;
/// Geräteflags; die unteren 2 Bit = aktives Hardware-Profil
pub const ADR_GERAETEFLAGS: u16 = 0x76;

/// Tabelle mit 10-Byte-Einträgen je (Hardware-Profil, Steckplatz)
pub const TABELLE_START: u16 = 0xA0;
pub const TABELLE_GROESSE: usize = 0x230;
const EINTRAG: usize = 10;
const STECKPLAETZE_JE_PROFIL: usize = 14;
/// Tastendaten beginnen hier, je Taste 8 Byte
const DATEN_START: u16 = 0x400;
const DATEN_ENDE: u16 = 0x7F8;

/// Profil 1–3 der Konfiguration -> Hardware-Profil (wie im Original; 1 ist verborgen)
pub const HW_PROFIL: [u8; 3] = [0, 2, 3];

/// DPI-Bereich 0x13–0x18 der Maus-Firmware, fest mit den Standardwerten des Originals
/// (sechs Stufen, Stufe 1 aktiv) – am Tastenfeld ohne Wirkung
const DPI_BYTES: [u8; 6] = [0x38, 0x08, 0x6C, 0xF2, 0x02, 0x18];
/// Alle vier DPI-Freigabebits in 0x1D gesetzt
const DPI_FREIGABE: u8 = 0x55;
/// DPI-Farben 0x20–0x31 (6 × R,G,B, je 4 Bit), Standardwerte des Originals
const DPI_FARBEN: [u8; 18] = [0, 15, 15, 15, 15, 0, 15, 0, 0, 15, 7, 11, 0, 0, 15, 7, 15, 7];

/// Adresse des Funktionsbytes für Hardware-Profil (0–3) und Steckplatz (1–11)
pub fn tasten_adresse(hw_profil: u8, steckplatz: u8) -> u16 {
    let h = steckplatz as u16;
    match hw_profil {
        0 => match h {
            1..=8 => 0x07 + h,
            9 => 0x10,
            10 => 0x3B,
            _ => 0x3C,
        },
        1 => 0x43 + h,
        2 => 0x4F + h,
        _ => 0x5F + h,
    }
}

fn eintrag_index(hw_profil: u8, steckplatz: u8) -> usize {
    hw_profil as usize * STECKPLAETZE_JE_PROFIL + steckplatz as usize - 1
}

fn abfrage_code(hz: u16) -> u8 {
    match hz {
        1000 => 0,
        500 => 1,
        250 => 2,
        _ => 3,
    }
}

fn abfrage_hz(code: u8) -> u16 {
    [1000, 500, 250, 125][(code & 3) as usize]
}

fn led_byte(b: &Beleuchtung) -> u8 {
    let (modus, reaktion, tempo) = match b.modus {
        LedModus::Aus => (0, 0, 7),
        LedModus::Tastenreaktion => (1, 1, b.geschwindigkeit),
        m => (m.index(), 0, b.geschwindigkeit),
    };
    0x80 | (modus & 7) << 4 | reaktion << 3 | (tempo & 7)
}

/// Funktion jeder Taste: [Hardware-Profil][Steckplatz] (Steckplatz-Index 0 ungenutzt)
fn belegung(k: &Konfiguration) -> Result<Vec<Vec<(Funktion, LedReaktion)>>> {
    let leer = (Funktion::Einfach(0), LedReaktion { aktiv: false, typ: 0 });
    let mut t = vec![vec![leer.clone(); 12]; 4];
    for (i, prof) in k.profil.iter().enumerate() {
        let hw = HW_PROFIL[i] as usize;
        for taste in &k.taste {
            let f = match prof.belegung.get(&taste.name) {
                Some(text) => funktionen::kodieren(text)?,
                None => Funktion::Einfach(0),
            };
            let r = prof.led_reaktion.get(&taste.name).copied().unwrap_or(leer.1);
            t[hw][taste.steckplatz as usize] = (f, r);
        }
    }
    Ok(t)
}

/// Baut die Grundeinstellungen 0x00–0x7F.
/// `geraeteflags` ist der aktuell im Gerät gespeicherte Wert von 0x76.
pub fn abbild_erstellen(k: &Konfiguration, geraeteflags: u8) -> Result<[u8; GROESSE]> {
    let mut m = [0u8; GROESSE];
    let profil = HW_PROFIL[(k.allgemein.aktives_profil - 1) as usize];

    m[ADR_PROFIL as usize] = profil << 5 | 0x03;

    let t = belegung(k)?;
    let byte = |p: usize, h: usize| {
        let (f, r) = &t[p][h];
        (r.typ & 3) << 6 | (r.aktiv as u8) << 5 | f.code()
    };
    for p in 0..4u8 {
        for h in 1..=11u8 {
            m[tasten_adresse(p, h) as usize] = byte(p as usize, h as usize);
        }
    }
    // Das Original schreibt den ersten Steckplatz des Folgeprofils zusätzlich ans Blockende
    m[0x3D] = byte(1, 1);
    m[0x4F] = byte(2, 1);
    m[0x5B] = byte(3, 1);

    m[0x13..0x19].copy_from_slice(&DPI_BYTES);

    let b = &k.beleuchtung;
    m[0x19] = b.zyklus_farben.iter().enumerate().fold(0, |acc, (i, &an)| acc | (an as u8) << i);
    m[0x1A..0x1C].copy_from_slice(&(k.allgemein.dauerfeuer_ms / 8).to_le_bytes());
    m[0x1C] = led_byte(b);
    m[0x1D] = DPI_FREIGABE | (b.benutzerfarbe_aktiv as u8) << 7;
    m[0x1E] = abfrage_code(k.allgemein.abfragerate_hz);

    m[0x20..0x32].copy_from_slice(&DPI_FARBEN);
    m[0x32] = 0x07;
    m[0x33] = 0x30;

    m[0x3A] = (b.einzelfarbe as u8) << 7
        | (b.symmetrie as u8) << 6
        | (b.richtung & 3) << 4
        | (k.erweitert.led_effekt_nibble & 0x0F);
    m[0x3F] = (b.helligkeit & 7) << 4;

    m[0x43] = if k.erweitert.flag_0x43 { 0xFD } else { 0xFC };
    m[0x6E..0x70].copy_from_slice(&k.erweitert.wert_0x6e.to_le_bytes());
    for (i, w) in k.erweitert.werte_0x70.iter().enumerate() {
        m[0x70 + i * 2..0x72 + i * 2].copy_from_slice(&w.to_le_bytes());
    }
    m[ADR_GERAETEFLAGS as usize] = (geraeteflags & !3) | (profil & 3);

    for (adr, daten) in &k.erweitert.rohdaten {
        let a = zahl_lesen(adr)? as usize;
        for (i, b) in daten.iter().enumerate() {
            if a + i < GROESSE {
                m[a + i] = *b;
            }
        }
    }
    Ok(m)
}

/// Legt ein Makro im Format des Originals ab: Kopf `01 00`, dann je Ereignis
/// `[Drücken<<7 | Verzögerung, HID-Code]`; das letzte Ereignis trägt 0x0A.
fn makro_ablegen(
    e: &mut [u8],
    adr: &mut u16,
    daten: &mut Vec<(u16, Vec<u8>)>,
    usb: u8,
    ereignisse: &[(bool, u8)],
    verzoegerung: u8,
) -> Result<()> {
    let anzahl = ereignisse.len();
    if anzahl > 0x1FFF {
        bail!("Makro zu lang ({anzahl} Ereignisse)");
    }
    let mut d = vec![0x01, 0x00];
    for (i, &(gedrueckt, u)) in ereignisse.iter().enumerate() {
        let v = if i + 1 == anzahl { 0x0A } else { verzoegerung & 0x7F };
        d.extend([(gedrueckt as u8) << 7 | v, u]);
    }
    d.resize(d.len().div_ceil(8) * 8, 0);
    if *adr + d.len() as u16 > DATEN_ENDE {
        bail!("Makrospeicher voll – Texte/Kombinationen kürzen");
    }
    e[0] = usb;
    e[1] = (anzahl & 0x7F) as u8;
    e[5] = *adr as u8;
    e[6] = (*adr >> 8) as u8 & 0x3F;
    e[7] = (anzahl >> 7) as u8 & 0x3F;
    e[8] = 0x0A;
    for (i, block) in d.chunks(8).enumerate() {
        daten.push((*adr + (i * 8) as u16, block.to_vec()));
    }
    *adr += d.len() as u16;
    Ok(())
}

/// Tabelle ab 0xA0 und Tastendaten ab 0x400 – nötig für `taste:` und `medien:`.
/// Liefert (Adresse, Bytes) in Sendereihenfolge.
pub fn makro_bloecke(k: &Konfiguration, abbild: &[u8; GROESSE]) -> Result<Vec<(u16, Vec<u8>)>> {
    let t = belegung(k)?;
    let usb = abbild[0x1E];
    let mut tabelle = [0u8; TABELLE_GROESSE];
    let mut daten = Vec::new();
    let mut adr = DATEN_START;

    for p in [0u8, 2, 3] {
        for h in 1..=11u8 {
            let i = eintrag_index(p, h) * EINTRAG;
            let e = &mut tabelle[i..i + EINTRAG];
            match &t[p as usize][h as usize].0 {
                Funktion::Taste(u) => {
                    let u = *u;
                    if adr + 8 > DATEN_ENDE {
                        bail!("Zu viele Tastenbelegungen – Makrospeicher voll");
                    }
                    e[0] = usb;
                    e[1] = 1; // Anzahl Ereignisse
                    e[2] = u;
                    e[5] = adr as u8;
                    e[6] = (adr >> 8) as u8 & 0x3F;
                    daten.push((adr, vec![0x01, 0x00, 0x80, u, 0, 0, 0, 0]));
                    adr += 8;
                }
                Funktion::Kombi(tasten, n) => {
                    let n = *n as usize;
                    let mut ereignisse: Vec<(bool, u8)> = tasten[..n].iter().map(|&u| (true, u)).collect();
                    ereignisse.extend(tasten[..n].iter().rev().map(|&u| (false, u)));
                    makro_ablegen(e, &mut adr, &mut daten, usb, &ereignisse, 0x01)?;
                }
                Funktion::Text(text) => {
                    let ereignisse = funktionen::text_ereignisse(text)?;
                    makro_ablegen(e, &mut adr, &mut daten, usb, &ereignisse, k.allgemein.tippverzoegerung)?;
                }
                Funktion::Medien(c) => {
                    let c = *c;
                    e[0] = usb;
                    e[3..5].copy_from_slice(&c.to_le_bytes());
                    e[5] = adr as u8;
                    e[6] = (adr >> 8) as u8 & 0x3F;
                }
                Funktion::Einfach(_) => {}
            }
        }
    }

    let mut bloecke: Vec<(u16, Vec<u8>)> = tabelle
        .chunks(8)
        .enumerate()
        .map(|(i, c)| (TABELLE_START + (i * 8) as u16, c.to_vec()))
        .collect();
    bloecke.extend(daten);
    for (adr, d) in &k.erweitert.rohdaten {
        let a = zahl_lesen(adr)?;
        if a as usize >= GROESSE {
            bloecke.push((a as u16, d.clone()));
        }
    }
    Ok(bloecke)
}

/// Übersetzt ausgelesene Daten zurück in eine Konfiguration.
/// `tabelle` = Bytes ab 0xA0 (TABELLE_GROESSE lang).
pub fn dekodieren(m: &[u8; GROESSE], tabelle: &[u8], tasten: &[Taste]) -> Konfiguration {
    let hw_aktiv = m[ADR_GERAETEFLAGS as usize] & 3;
    let aktives_profil = HW_PROFIL.iter().position(|&p| p == hw_aktiv).unwrap_or(0) as u8 + 1;

    let led = m[0x1C];
    let modus = match (led >> 4 & 7, led >> 3 & 1, led & 7) {
        (_, 1, _) => LedModus::Tastenreaktion,
        (0, 0, 7) => LedModus::Aus,
        (0, _, _) => {
            eprintln!("Hinweis: LED-Modus „DPI-Atmen“ (Maus) – als \"zyklus_atmen\" übernommen.");
            LedModus::ZyklusAtmen
        }
        (1, _, _) => LedModus::ZyklusAtmen,
        (2, _, _) => LedModus::Dauerlicht,
        _ => LedModus::EinzelAtmen,
    };

    let mut alle_tasten: Vec<Taste> = tasten.to_vec();
    let profil = HW_PROFIL
        .iter()
        .enumerate()
        .map(|(i, &p)| {
            let mut belegung = BTreeMap::new();
            let mut led_reaktion = BTreeMap::new();
            for h in 1..=11u8 {
                let b = m[tasten_adresse(p, h) as usize];
                let name = match alle_tasten.iter().find(|t| t.steckplatz == h) {
                    Some(t) => t.name.clone(),
                    None if b & 0x1F != 0 => {
                        let n = format!("steckplatz_{h}");
                        alle_tasten.push(Taste { name: n.clone(), steckplatz: h });
                        n
                    }
                    None => continue,
                };
                let e = &tabelle[eintrag_index(p, h) * EINTRAG..][..EINTRAG];
                let f = match b & 0x1F {
                    CODE_TASTE if e[1] == 1 => Funktion::Taste(e[2]),
                    CODE_TASTE | CODE_KOMBI | CODE_MAKRO | 0x1F => {
                        eprintln!(
                            "Hinweis: Profil {}, Taste \"{name}\" enthält ein Makro ({} Ereignisse) – als \"aus\" übernommen.",
                            i + 1,
                            e[1]
                        );
                        Funktion::Einfach(0)
                    }
                    CODE_MEDIEN => Funktion::Medien(u16::from_le_bytes([e[3], e[4]])),
                    c => Funktion::Einfach(c),
                };
                belegung.insert(name.clone(), funktionen::text_von(f));
                if b & 0xE0 != 0 {
                    led_reaktion.insert(name, LedReaktion { aktiv: b & 0x20 != 0, typ: b >> 6 });
                }
            }
            Profil { name: format!("Profil {}", i + 1), belegung, led_reaktion, bezeichnung: BTreeMap::new() }
        })
        .collect();

    let w = |a: usize| u16::from_le_bytes([m[a], m[a + 1]]);
    Konfiguration {
        geraet: Geraet::default(),
        allgemein: Allgemein {
            aktives_profil,
            abfragerate_hz: abfrage_hz(m[0x1E]),
            dauerfeuer_ms: w(0x1A).saturating_mul(8).clamp(8, 2040),
            ..Allgemein::default()
        },
        beleuchtung: Beleuchtung {
            modus,
            helligkeit: m[0x3F] >> 4 & 7,
            geschwindigkeit: led & 7,
            zyklus_farben: [m[0x19] & 1 != 0, m[0x19] & 2 != 0, m[0x19] & 4 != 0, m[0x19] & 8 != 0],
            benutzerfarbe_aktiv: m[0x1D] & 0x80 != 0,
            einzelfarbe: m[0x3A] & 0x80 != 0,
            symmetrie: m[0x3A] & 0x40 != 0,
            richtung: m[0x3A] >> 4 & 3,
        },
        taste: alle_tasten,
        profil,
        erweitert: Erweitert {
            rohdaten: BTreeMap::new(),
            flag_0x43: m[0x43] == 0xFD,
            wert_0x6e: w(0x6E),
            werte_0x70: [w(0x70), w(0x72), w(0x74)],
            led_effekt_nibble: m[0x3A] & 0x0F,
        },
        alt_dpi: Default::default(),
        alt_windows: Default::default(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn beispiel() -> Konfiguration {
        toml::from_str(include_str!("../konfiguration.toml")).unwrap()
    }

    fn tabelle_aus(bloecke: &[(u16, Vec<u8>)]) -> Vec<u8> {
        let mut t = vec![0u8; TABELLE_GROESSE];
        for (a, d) in bloecke {
            let a = *a as usize;
            if (TABELLE_START as usize..TABELLE_START as usize + TABELLE_GROESSE).contains(&a) {
                let o = a - TABELLE_START as usize;
                t[o..o + d.len()].copy_from_slice(d);
            }
        }
        t
    }

    #[test]
    fn alte_maus_abschnitte_werden_ignoriert() {
        let alt = include_str!("../konfiguration.toml").replace(
            "[beleuchtung]
modus = \"aus\"",
            "[dpi]
aktive_stufe = 1

[[dpi.stufe]]
wert = 5
farbe = \"#00FFFF\"

[windows]

[beleuchtung]
modus = \"dpi_atmen\"",
        );
        assert_ne!(alt, include_str!("../konfiguration.toml"));
        let k: Konfiguration = toml::from_str(&alt).unwrap();
        assert_eq!(k.beleuchtung.modus, LedModus::ZyklusAtmen);
        let neu = toml::to_string(&k).unwrap();
        assert!(!neu.contains("dpi") && !neu.contains("[windows]"));
    }

    #[test]
    fn beispiel_ist_gueltig() {
        beispiel().pruefen().unwrap();
    }

    #[test]
    fn adressen_ueberschneiden_sich_nicht() {
        let mut gesehen = std::collections::HashSet::new();
        for p in 0..4 {
            for h in 1..=11 {
                assert!(gesehen.insert(tasten_adresse(p, h)), "doppelt: p{p} h{h}");
            }
        }
    }

    #[test]
    fn hin_und_zurueck() {
        let k = beispiel();
        let m = abbild_erstellen(&k, 0xA0).unwrap();
        let t = tabelle_aus(&makro_bloecke(&k, &m).unwrap());
        let d = dekodieren(&m, &t, &k.taste);
        let m2 = abbild_erstellen(&d, 0xA0).unwrap();
        assert_eq!(m.to_vec(), m2.to_vec());
        assert_eq!(t, tabelle_aus(&makro_bloecke(&d, &m2).unwrap()));
    }

    #[test]
    fn kombi_wie_original_makro() {
        let mut k = beispiel();
        k.profil[0].belegung.insert("t1".into(), "kombi:strg+c".into());
        let m = abbild_erstellen(&k, 0).unwrap();
        assert_eq!(m[0x0E] & 0x1F, CODE_KOMBI);
        let b = makro_bloecke(&k, &m).unwrap();
        let e = &tabelle_aus(&b)[eintrag_index(0, 7) * EINTRAG..][..EINTRAG];
        assert_eq!((e[1], e[2], e[8]), (4, 0, 0x0A));
        let adr = u16::from_le_bytes([e[5], e[6]]);
        let d = &b.iter().find(|(a, _)| *a == adr).unwrap().1;
        assert_eq!(d[..8], [0x01, 0x00, 0x81, 0xE0, 0x81, 0x06, 0x01, 0x06]);
        let d2 = &b.iter().find(|(a, _)| *a == adr + 8).unwrap().1;
        assert_eq!(d2[..2], [0x0A, 0xE0]); // letztes Ereignis wie im Original mit 0x0A
    }

    #[test]
    fn text_als_makro() {
        let mut k = beispiel();
        k.profil[0].belegung.insert("t1".into(), "text:a@b".into());
        let m = abbild_erstellen(&k, 0).unwrap();
        assert_eq!(m[0x0E] & 0x1F, CODE_KOMBI);
        let b = makro_bloecke(&k, &m).unwrap();
        let e = &tabelle_aus(&b)[eintrag_index(0, 7) * EINTRAG..][..EINTRAG];
        assert_eq!((e[1], e[8]), (8, 0x0A)); // a(2) + @(4) + b(2)
        let adr = u16::from_le_bytes([e[5], e[6]]);
        let d = &b.iter().find(|(a, _)| *a == adr).unwrap().1;
        assert_eq!(d[..8], [0x01, 0x00, 0x8F, 0x04, 0x0F, 0x04, 0x8F, 0xE6]);

        k.allgemein.tippverzoegerung = 30;
        let b = makro_bloecke(&k, &m).unwrap();
        let d = &b.iter().find(|(a, _)| *a == adr).unwrap().1;
        assert_eq!(d[2..5], [0x9E, 0x04, 0x1E]);
    }

    #[test]
    fn taste_wie_original() {
        // Original-Standard: Taste K1 (Steckplatz 7) = "1" (HID 0x1E)
        let k = beispiel();
        let m = abbild_erstellen(&k, 0).unwrap();
        assert_eq!(m[0x01], 0x03);
        assert_eq!(m[0x0E] & 0x1F, CODE_TASTE);
        let b = makro_bloecke(&k, &m).unwrap();
        assert_eq!(b.len(), 70 + 16); // Tabelle + 16 Tastatur-Tasten (Profil 1 und 3)
        let t = tabelle_aus(&b);
        let e = &t[eintrag_index(0, 7) * EINTRAG..][..EINTRAG];
        assert_eq!(e[1..3], [1, 0x1E]);
        let adr = u16::from_le_bytes([e[5], e[6]]);
        let daten = b.iter().find(|(a, _)| *a == adr).unwrap();
        assert_eq!(daten.1, vec![1, 0, 0x80, 0x1E, 0, 0, 0, 0]);
    }
}
