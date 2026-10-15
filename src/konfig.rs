//! Konfigurationsdatei (TOML) – alle Schlüssel auf Deutsch.

use std::collections::BTreeMap;
use std::path::Path;

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Konfiguration {
    #[serde(default)]
    pub geraet: Geraet,
    #[serde(default)]
    pub allgemein: Allgemein,
    #[serde(default)]
    pub dpi: Dpi,
    #[serde(default)]
    pub beleuchtung: Beleuchtung,
    #[serde(default)]
    pub windows: WindowsEinstellungen,
    /// Physische Tasten: Name -> Hardware-Steckplatz (1–11)
    pub taste: Vec<Taste>,
    /// 1–3 Profile (wie im Original; Hardware-Profil 2 bleibt ungenutzt)
    pub profil: Vec<Profil>,
    #[serde(default)]
    pub erweitert: Erweitert,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields, default)]
pub struct Geraet {
    pub vendor_id: u16,
    pub product_id: u16,
    /// Länge eines Feature-Reports inkl. Report-ID
    pub bericht_laenge: usize,
}

impl Default for Geraet {
    fn default() -> Self {
        Geraet { vendor_id: 0x30FA, product_id: 0x1340, bericht_laenge: 8 }
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields, default)]
pub struct Allgemein {
    /// Profil, das nach dem Übertragen aktiv sein soll (1–3)
    pub aktives_profil: u8,
    /// USB-Abfragerate in Hz: 125, 250, 500 oder 1000
    pub abfragerate_hz: u16,
    /// Intervall der Dauerfeuer-Funktion in Millisekunden (8–2040)
    pub dauerfeuer_ms: u16,
    /// Verzögerung je Tastenereignis beim Tippen von `text:` (1–127, Firmware-Einheit;
    /// Original: 15). Kleiner = schneller – zu schnell verschluckt das System Zeichen.
    pub tippverzoegerung: u8,
}

impl Default for Allgemein {
    fn default() -> Self {
        Allgemein { aktives_profil: 1, abfragerate_hz: 1000, dauerfeuer_ms: 50, tippverzoegerung: 15 }
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields, default)]
pub struct Dpi {
    /// Beim Einschalten aktive DPI-Stufe (1–6)
    pub aktive_stufe: u8,
    pub stufe: Vec<DpiStufe>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct DpiStufe {
    /// Sensorwert 0–16 (wie DPI_SET im Original; 0 = niedrigste Auflösung)
    pub wert: u8,
    #[serde(default = "wahr")]
    pub aktiv: bool,
    /// Farbe als "#RRGGBB" – wird auf 4 Bit je Kanal reduziert
    pub farbe: String,
}

fn wahr() -> bool {
    true
}

impl Default for Dpi {
    fn default() -> Self {
        let s = |wert, farbe: &str| DpiStufe { wert, aktiv: true, farbe: farbe.into() };
        Dpi {
            aktive_stufe: 1,
            stufe: vec![
                s(5, "#00FFFF"),
                s(10, "#FFFF00"),
                s(13, "#FF0000"),
                s(16, "#FF77BB"),
                s(14, "#0000FF"),
                s(16, "#77FF77"),
            ],
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum LedModus {
    DpiAtmen,
    ZyklusAtmen,
    Dauerlicht,
    Tastenreaktion,
    Aus,
    EinzelAtmen,
}

impl LedModus {
    /// Index wie im Original (LED_TYPE)
    pub fn index(self) -> u8 {
        match self {
            LedModus::DpiAtmen => 0,
            LedModus::ZyklusAtmen => 1,
            LedModus::Dauerlicht => 2,
            LedModus::Tastenreaktion => 3,
            LedModus::Aus => 4,
            LedModus::EinzelAtmen => 5,
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields, default)]
pub struct Beleuchtung {
    pub modus: LedModus,
    /// 0–7
    pub helligkeit: u8,
    /// 0–7 (kleiner = schneller)
    pub geschwindigkeit: u8,
    /// Welche der vier Zyklusfarben verwendet werden
    pub zyklus_farben: [bool; 4],
    pub benutzerfarbe_aktiv: bool,
    pub einzelfarbe: bool,
    pub symmetrie: bool,
    /// 0–3
    pub richtung: u8,
}

impl Default for Beleuchtung {
    fn default() -> Self {
        Beleuchtung {
            modus: LedModus::ZyklusAtmen,
            helligkeit: 4,
            geschwindigkeit: 3,
            zyklus_farben: [false; 4],
            benutzerfarbe_aktiv: false,
            einzelfarbe: false,
            symmetrie: false,
            richtung: 0,
        }
    }
}

/// Diese Werte setzt das Original nicht in der Maus, sondern in Windows.
#[derive(Debug, Deserialize, Serialize, Default)]
#[serde(deny_unknown_fields, default)]
pub struct WindowsEinstellungen {
    /// Zeigergeschwindigkeit 1–20 (leer lassen = nicht ändern)
    pub zeigergeschwindigkeit: Option<u32>,
    /// Zeilen pro Mausrad-Raste (leer lassen = nicht ändern)
    pub scrollzeilen: Option<u32>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct Taste {
    pub name: String,
    /// Hardware-Steckplatz 1–11 (siehe Dokumentation)
    pub steckplatz: u8,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Profil {
    #[serde(default)]
    pub name: String,
    /// Tastenname -> Funktion
    pub belegung: BTreeMap<String, String>,
    /// Tastenname -> LED-Reaktion beim Drücken
    #[serde(default)]
    pub led_reaktion: BTreeMap<String, LedReaktion>,
    /// Tastenname -> eigene Bezeichnung (nur Anzeige in der GUI)
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub bezeichnung: BTreeMap<String, String>,
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy)]
#[serde(deny_unknown_fields)]
pub struct LedReaktion {
    pub aktiv: bool,
    /// 0–3
    #[serde(default)]
    pub typ: u8,
}

#[derive(Debug, Deserialize, Serialize, Default)]
#[serde(deny_unknown_fields, default)]
pub struct Erweitert {
    /// Überschreibt einzelne Bytes nach der Kodierung: "0x70" = [0, 0, ...]
    pub rohdaten: BTreeMap<String, Vec<u8>>,
    /// Unbekanntes Flag im Block 0x40 (Original: 0xFC / 0xFD)
    pub flag_0x43: bool,
    /// Unbekannter 16-Bit-Wert an 0x6E
    pub wert_0x6e: u16,
    /// Unbekannte 16-Bit-Werte an 0x70, 0x72, 0x74
    pub werte_0x70: [u16; 3],
    /// Niedriges Nibble des LED-Effekt-Bytes 0x3A
    pub led_effekt_nibble: u8,
}

impl Konfiguration {
    pub fn laden(pfad: &Path) -> Result<Self> {
        let text = std::fs::read_to_string(pfad)
            .with_context(|| format!("Datei {} kann nicht gelesen werden", pfad.display()))?;
        let k: Konfiguration = toml::from_str(&text)
            .with_context(|| format!("Fehler in {}", pfad.display()))?;
        k.pruefen()?;
        Ok(k)
    }

    pub fn pruefen(&self) -> Result<()> {
        if self.profil.is_empty() || self.profil.len() > 3 {
            bail!("Es müssen 1–3 [[profil]]-Einträge vorhanden sein (gefunden: {})", self.profil.len());
        }
        if !(1..=self.profil.len() as u8).contains(&self.allgemein.aktives_profil) {
            bail!("allgemein.aktives_profil muss zwischen 1 und {} liegen", self.profil.len());
        }
        if ![125, 250, 500, 1000].contains(&self.allgemein.abfragerate_hz) {
            bail!("allgemein.abfragerate_hz muss 125, 250, 500 oder 1000 sein");
        }
        if !(8..=2040).contains(&self.allgemein.dauerfeuer_ms) {
            bail!("allgemein.dauerfeuer_ms muss zwischen 8 und 2040 liegen");
        }
        if !(1..=127).contains(&self.allgemein.tippverzoegerung) {
            bail!("allgemein.tippverzoegerung muss zwischen 1 und 127 liegen");
        }
        if self.dpi.stufe.len() != 6 {
            bail!("Es müssen genau 6 [[dpi.stufe]]-Einträge vorhanden sein");
        }
        if !(1..=6).contains(&self.dpi.aktive_stufe) {
            bail!("dpi.aktive_stufe muss zwischen 1 und 6 liegen");
        }
        for (i, s) in self.dpi.stufe.iter().enumerate() {
            if s.wert > 16 {
                bail!("dpi.stufe[{}].wert muss 0–16 sein", i + 1);
            }
            farbe_lesen(&s.farbe).with_context(|| format!("dpi.stufe[{}].farbe", i + 1))?;
        }
        let b = &self.beleuchtung;
        if b.helligkeit > 7 || b.geschwindigkeit > 7 || b.richtung > 3 {
            bail!("beleuchtung: helligkeit/geschwindigkeit 0–7, richtung 0–3");
        }
        for t in &self.taste {
            if !(1..=11).contains(&t.steckplatz) {
                bail!("Taste \"{}\": steckplatz muss 1–11 sein", t.name);
            }
        }
        for (pi, p) in self.profil.iter().enumerate() {
            for (name, funktion) in &p.belegung {
                if !self.taste.iter().any(|t| &t.name == name) {
                    bail!("Profil {}: unbekannte Taste \"{name}\" (nicht unter [[taste]] definiert)", pi + 1);
                }
                crate::funktionen::kodieren(funktion)
                    .with_context(|| format!("Profil {}, Taste \"{name}\"", pi + 1))?;
            }
            for (name, r) in &p.led_reaktion {
                if !self.taste.iter().any(|t| &t.name == name) {
                    bail!("Profil {}: led_reaktion für unbekannte Taste \"{name}\"", pi + 1);
                }
                if r.typ > 3 {
                    bail!("Profil {}: led_reaktion.typ muss 0–3 sein", pi + 1);
                }
            }
        }
        for (adr, daten) in &self.erweitert.rohdaten {
            zahl_lesen(adr).with_context(|| format!("erweitert.rohdaten: Adresse \"{adr}\""))?;
            if daten.is_empty() || daten.len() > 8 {
                bail!("erweitert.rohdaten \"{adr}\": 1–8 Byte erlaubt");
            }
        }
        Ok(())
    }
}

/// "#RRGGBB" -> 4-Bit-Kanäle (0–15)
pub fn farbe_lesen(s: &str) -> Result<[u8; 3]> {
    let h = s.trim().trim_start_matches('#');
    if h.len() != 6 {
        bail!("Farbe \"{s}\" muss die Form #RRGGBB haben");
    }
    let mut out = [0u8; 3];
    for i in 0..3 {
        let v = u8::from_str_radix(&h[i * 2..i * 2 + 2], 16)
            .with_context(|| format!("ungültige Farbe \"{s}\""))?;
        out[i] = v >> 4;
    }
    Ok(out)
}

pub fn zahl_lesen(s: &str) -> Result<u32> {
    let s = s.trim();
    let r = if let Some(h) = s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")) {
        u32::from_str_radix(h, 16)
    } else {
        s.parse()
    };
    r.with_context(|| format!("\"{s}\" ist keine Zahl"))
}
