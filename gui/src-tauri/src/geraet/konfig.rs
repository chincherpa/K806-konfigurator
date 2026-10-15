//! Konfigurationsdatei (TOML) – alle Schlüssel auf Deutsch.

use std::collections::BTreeMap;
use std::path::Path;

use anyhow::{bail, Context, Result};
use serde::de::IgnoredAny;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Konfiguration {
    #[serde(default)]
    pub geraet: Geraet,
    #[serde(default)]
    pub allgemein: Allgemein,
    #[serde(default)]
    pub beleuchtung: Beleuchtung,
    /// Physische Tasten: Name -> Hardware-Steckplatz (1–11)
    pub taste: Vec<Taste>,
    /// 1–3 Profile (wie im Original; Hardware-Profil 2 bleibt ungenutzt)
    pub profil: Vec<Profil>,
    #[serde(default)]
    pub erweitert: Erweitert,
    /// Frühere Maus-Abschnitte `[dpi]` und `[windows]` – werden beim Laden ignoriert
    #[serde(default, rename = "dpi", skip_serializing)]
    pub(crate) alt_dpi: IgnoredAny,
    #[serde(default, rename = "windows", skip_serializing)]
    pub(crate) alt_windows: IgnoredAny,
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

#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum LedModus {
    /// `dpi_atmen` (Maus) aus älteren Dateien wird hierauf abgebildet
    #[serde(alias = "dpi_atmen")]
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
            .with_context(|| crate::tr!("Datei {} kann nicht gelesen werden", "Cannot read file {}", pfad.display()))?;
        let k: Konfiguration = toml::from_str(&text)
            .with_context(|| crate::tr!("Fehler in {}", "Error in {}", pfad.display()))?;
        k.pruefen()?;
        Ok(k)
    }

    pub fn pruefen(&self) -> Result<()> {
        if self.profil.is_empty() || self.profil.len() > 3 {
            bail!(crate::tr!(
                "Es müssen 1–3 [[profil]]-Einträge vorhanden sein (gefunden: {})",
                "There must be 1–3 [[profil]] entries (found: {})",
                self.profil.len()
            ));
        }
        if !(1..=self.profil.len() as u8).contains(&self.allgemein.aktives_profil) {
            bail!(crate::tr!(
                "allgemein.aktives_profil muss zwischen 1 und {} liegen",
                "allgemein.aktives_profil must be between 1 and {}",
                self.profil.len()
            ));
        }
        if ![125, 250, 500, 1000].contains(&self.allgemein.abfragerate_hz) {
            bail!(crate::tr!("allgemein.abfragerate_hz muss 125, 250, 500 oder 1000 sein", "allgemein.abfragerate_hz must be 125, 250, 500 or 1000"));
        }
        if !(8..=2040).contains(&self.allgemein.dauerfeuer_ms) {
            bail!(crate::tr!("allgemein.dauerfeuer_ms muss zwischen 8 und 2040 liegen", "allgemein.dauerfeuer_ms must be between 8 and 2040"));
        }
        if !(1..=127).contains(&self.allgemein.tippverzoegerung) {
            bail!(crate::tr!("allgemein.tippverzoegerung muss zwischen 1 und 127 liegen", "allgemein.tippverzoegerung must be between 1 and 127"));
        }
        let b = &self.beleuchtung;
        if b.helligkeit > 7 || b.geschwindigkeit > 7 || b.richtung > 3 {
            bail!(crate::tr!("beleuchtung: helligkeit/geschwindigkeit 0–7, richtung 0–3", "beleuchtung: helligkeit/geschwindigkeit 0–7, richtung 0–3"));
        }
        for t in &self.taste {
            if !(1..=11).contains(&t.steckplatz) {
                bail!(crate::tr!("Taste \"{}\": steckplatz muss 1–11 sein", "Key \"{}\": steckplatz must be 1–11", t.name));
            }
        }
        for (pi, p) in self.profil.iter().enumerate() {
            for (name, funktion) in &p.belegung {
                if !self.taste.iter().any(|t| &t.name == name) {
                    bail!(crate::tr!(
                        "Profil {}: unbekannte Taste \"{name}\" (nicht unter [[taste]] definiert)",
                        "Profile {}: unknown key \"{name}\" (not defined under [[taste]])",
                        pi + 1
                    ));
                }
                crate::geraet::funktionen::kodieren(funktion)
                    .with_context(|| crate::tr!("Profil {}, Taste \"{name}\"", "Profile {}, key \"{name}\"", pi + 1))?;
            }
            for (name, r) in &p.led_reaktion {
                if !self.taste.iter().any(|t| &t.name == name) {
                    bail!(crate::tr!("Profil {}: led_reaktion für unbekannte Taste \"{name}\"", "Profile {}: led_reaktion for unknown key \"{name}\"", pi + 1));
                }
                if r.typ > 3 {
                    bail!(crate::tr!("Profil {}: led_reaktion.typ muss 0–3 sein", "Profile {}: led_reaktion.typ must be 0–3", pi + 1));
                }
            }
        }
        for (adr, daten) in &self.erweitert.rohdaten {
            zahl_lesen(adr).with_context(|| crate::tr!("erweitert.rohdaten: Adresse \"{adr}\"", "erweitert.rohdaten: address \"{adr}\""))?;
            if daten.is_empty() || daten.len() > 8 {
                bail!(crate::tr!("erweitert.rohdaten \"{adr}\": 1–8 Byte erlaubt", "erweitert.rohdaten \"{adr}\": 1–8 bytes allowed"));
            }
        }
        Ok(())
    }
}

pub fn zahl_lesen(s: &str) -> Result<u32> {
    let s = s.trim();
    let r = if let Some(h) = s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")) {
        u32::from_str_radix(h, 16)
    } else {
        s.parse()
    };
    r.with_context(|| crate::tr!("\"{s}\" ist keine Zahl", "\"{s}\" is not a number"))
}
