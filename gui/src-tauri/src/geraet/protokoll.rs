//! HID-Transportschicht für die X806-Maus.
//!
//! Die Maus stellt zwei herstellerspezifische HID-Collections bereit:
//!   * Usage-Page 0xFF00 / Usage 1  – vom Original nur zum Lesen geöffnet (ungenutzt)
//!   * Usage-Page 0xFF01 / Usage 1  – Feature-Reports für alle Befehle
//!
//! Jeder Befehl ist ein Feature-Report mit Report-ID 7:
//!
//! ```text
//!   [0] 0x07  Report-ID
//!   [1] Befehl   (0x18 = Speicherzugriff, 0x11 = Feuer-Steuerung, 0x20 = Freigabe)
//!   [2] Unterbefehl (bei 0x18: 3 = Byte puffern, 9 = übernehmen, 5 = lesen,
//!                    0x10 = dauerhaft speichern, 0 = Leerlauf)
//!   [3] Index innerhalb des Blocks
//!   [4] Adresse Low-Byte
//!   [5] Adresse High-Byte
//!   [6] Datenbyte
//!   [7] Blocklänge - 1
//! ```

use std::thread::sleep;
use std::time::Duration;

use anyhow::{anyhow, bail, Context, Result};
use hidapi::{HidApi, HidDevice};

pub const REPORT_ID: u8 = 0x07;
pub const CMD_SPEICHER: u8 = 0x18;

pub const OP_LEERLAUF: u8 = 0x00;
pub const OP_PUFFERN: u8 = 0x03;
pub const OP_LESEN: u8 = 0x05;
pub const OP_UEBERNEHMEN: u8 = 0x09;
pub const OP_SPEICHERN: u8 = 0x10;

const USAGE_PAGE_BEFEHLE: u16 = 0xFF01;

/// Status, wie ihn die Maus per GET_FEATURE liefert.
#[derive(Debug, Clone)]
pub struct Status {
    pub roh: Vec<u8>,
    /// Aktives Hardware-Profil (0–3)
    pub aktives_profil: u8,
    /// Ende des Makrospeichers (0x7F8 oder 0xFF8)
    pub makro_ende: u16,
}

pub struct Maus {
    geraet: Option<HidDevice>,
    laenge: usize,
    trockenlauf: bool,
    pub ausfuehrlich: bool,
}

/// Listet alle passenden HID-Schnittstellen auf.
pub fn auflisten(vid: u16, pid: u16) -> Result<Vec<String>> {
    let api = HidApi::new().context("HID-Bibliothek konnte nicht initialisiert werden")?;
    Ok(api
        .device_list()
        .filter(|d| d.vendor_id() == vid && d.product_id() == pid)
        .map(|d| {
            format!(
                "{:04X}:{:04X}  UsagePage=0x{:04X} Usage=0x{:04X}  {}  {}",
                d.vendor_id(),
                d.product_id(),
                d.usage_page(),
                d.usage(),
                d.product_string().unwrap_or("?"),
                d.path().to_string_lossy()
            )
        })
        .collect())
}

impl Maus {
    pub fn oeffnen(vid: u16, pid: u16, laenge: usize, trockenlauf: bool) -> Result<Self> {
        if trockenlauf {
            return Ok(Maus { geraet: None, laenge, trockenlauf, ausfuehrlich: true });
        }
        let api = HidApi::new().context("HID-Bibliothek konnte nicht initialisiert werden")?;
        let info = api
            .device_list()
            .find(|d| d.vendor_id() == vid && d.product_id() == pid && d.usage_page() == USAGE_PAGE_BEFEHLE)
            .ok_or_else(|| {
                anyhow!(
                    "Keine Maus {vid:04X}:{pid:04X} mit Usage-Page 0x{USAGE_PAGE_BEFEHLE:04X} gefunden. \
                     Ist sie angeschlossen und das Original-Programm (X806.exe) geschlossen?"
                )
            })?;
        let geraet = info.open_device(&api).context("Gerät konnte nicht geöffnet werden")?;
        Ok(Maus { geraet: Some(geraet), laenge, trockenlauf, ausfuehrlich: false })
    }

    fn bericht(&self, nutzdaten: [u8; 7]) -> Vec<u8> {
        let mut buf = vec![0u8; self.laenge.max(8)];
        buf[0] = REPORT_ID;
        buf[1..8].copy_from_slice(&nutzdaten);
        buf
    }

    fn senden(&self, nutzdaten: [u8; 7]) -> Result<()> {
        let buf = self.bericht(nutzdaten);
        if self.ausfuehrlich {
            println!("  -> {}", hex(&buf));
        }
        if self.trockenlauf {
            return Ok(());
        }
        let g = self.geraet.as_ref().expect("Gerät offen");
        g.send_feature_report(&buf)
            .with_context(|| format!("SET_FEATURE fehlgeschlagen (Berichtslänge {} Byte – ggf. `bericht_laenge` anpassen)", buf.len()))
    }

    fn empfangen(&self) -> Result<Vec<u8>> {
        if self.trockenlauf {
            return Ok(vec![0; self.laenge.max(8)]);
        }
        let mut buf = vec![0u8; self.laenge.max(8)];
        buf[0] = REPORT_ID;
        let g = self.geraet.as_ref().expect("Gerät offen");
        let n = g.get_feature_report(&mut buf).context("GET_FEATURE fehlgeschlagen")?;
        buf.truncate(n.max(8));
        if self.ausfuehrlich {
            println!("  <- {}", hex(&buf));
        }
        Ok(buf)
    }

    fn speicher(&self, op: u8, index: u8, adresse: u16, daten: u8, laenge_minus_1: u8) -> Result<()> {
        let [lo, hi] = adresse.to_le_bytes();
        self.senden([CMD_SPEICHER, op, index, lo, hi, daten, laenge_minus_1])
    }

    /// Schreibt einen Block (1–8 Byte) an eine Adresse im Konfigurationsspeicher.
    pub fn block_schreiben(&self, adresse: u16, daten: &[u8]) -> Result<()> {
        if daten.is_empty() || daten.len() > 8 {
            bail!("Blocklänge muss 1–8 Byte sein (ist {})", daten.len());
        }
        let n = daten.len() as u8 - 1;
        for (i, b) in daten.iter().enumerate() {
            self.speicher(OP_PUFFERN, i as u8, adresse, *b, n)?;
        }
        warten(4, self.trockenlauf);
        self.speicher(OP_UEBERNEHMEN, 0, adresse, 0, n)?;
        warten(2, self.trockenlauf);
        self.speicher(OP_LEERLAUF, 0, adresse, 0, 0)
    }

    /// Liest ein einzelnes Byte (Ablauf exakt wie im Original).
    pub fn byte_lesen(&self, adresse: u16) -> Result<u8> {
        self.speicher(OP_PUFFERN, 0, adresse, 0, 0)?;
        warten(2, self.trockenlauf);
        warten(2, self.trockenlauf);
        self.speicher(OP_LEERLAUF, 0, adresse, 0, 0)?;
        self.speicher(OP_LESEN, 0, adresse, 0, 0)?;
        warten(2, self.trockenlauf);
        let antwort = self.empfangen()?;
        warten(2, self.trockenlauf);
        self.speicher(OP_LEERLAUF, 0, 0, 0, 0)?;
        Ok(antwort[1])
    }

    /// Legt alle gepufferten Änderungen dauerhaft im Flash ab.
    pub fn speichern(&self) -> Result<()> {
        self.speicher(OP_SPEICHERN, 0, 0, 0, 0)?;
        warten(20, self.trockenlauf);
        self.speicher(OP_LEERLAUF, 0, 0, 0, 0)
    }

    pub fn status(&self) -> Result<Status> {
        let roh = self.empfangen()?;
        let b6 = roh.get(6).copied().unwrap_or(0);
        Ok(Status {
            aktives_profil: (b6 >> 2) & 3,
            makro_ende: (((b6 & 2) as u16) << 10) | 0x7F8,
            roh,
        })
    }
}

fn warten(ms: u64, trockenlauf: bool) {
    if !trockenlauf {
        sleep(Duration::from_millis(ms));
    }
}

pub fn hex(d: &[u8]) -> String {
    d.iter().map(|b| format!("{b:02X}")).collect::<Vec<_>>().join(" ")
}
