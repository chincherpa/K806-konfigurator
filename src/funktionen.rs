//! Tastenfunktionen.
//!
//! Drei Arten:
//!   * einfache Funktionen – nur ein 5-Bit-Code im Tastenbyte
//!   * `taste:<name>`      – eine Tastaturtaste (Code 0x0E + Eintrag im Makrospeicher)
//!   * `medien:<name>`     – Multimedia-Taste (Code 0x0F + HID-Consumer-Code)
//!   * `kombi:a+b+c`       – Tastenkombination, als kurzes Makro gespeichert
//!   * `text:Hallo`        – tippt einen Text (deutsches Tastaturlayout), als Makro gespeichert

use anyhow::{bail, Result};

use crate::konfig::zahl_lesen;

#[derive(Debug, Clone, PartialEq)]
pub enum Funktion {
    Einfach(u8),
    /// HID-Tastatur-Usage (Seite 0x07)
    Taste(u8),
    /// HID-Consumer-Usage (Seite 0x0C)
    Medien(u16),
    /// Bis zu 4 Tasten nacheinander drücken, dann rückwärts loslassen
    Kombi([u8; 4], u8),
    /// Text, der Zeichen für Zeichen getippt wird
    Text(String),
}

/// Höchstlänge für `text:` – der Makrospeicher ist begrenzt
pub const TEXT_MAX: usize = 120;

const UMSCHALT: u8 = 0xE1;
const ALTGR: u8 = 0xE6;

pub const CODE_TASTE: u8 = 0x0E;
pub const CODE_MEDIEN: u8 = 0x0F;

impl Funktion {
    pub fn code(&self) -> u8 {
        match self {
            Funktion::Einfach(c) => *c,
            Funktion::Taste(_) | Funktion::Kombi(..) | Funktion::Text(_) => CODE_TASTE,
            Funktion::Medien(_) => CODE_MEDIEN,
        }
    }
}

pub const EINFACH: &[(&str, u8, &str)] = &[
    ("aus", 0x00, "Taste ohne Funktion"),
    ("linksklick", 0x01, "Linke Maustaste"),
    ("mittelklick", 0x02, "Mittlere Maustaste"),
    ("rechtsklick", 0x03, "Rechte Maustaste"),
    ("zurueck", 0x04, "Maus-Taste 4 (Browser zurück)"),
    ("vor", 0x05, "Maus-Taste 5 (Browser vor)"),
    ("desktop_anzeigen", 0x07, "Desktop anzeigen"),
    ("doppelklick", 0x08, "Doppelklick"),
    ("dauerfeuer", 0x09, "Linksklick-Dauerfeuer"),
    ("led_zyklus", 0x15, "Beleuchtungsmodus wechseln"),
    ("profil_1", 0x17, "Zu Profil 1 wechseln"),
    ("profil_2", 0x19, "Zu Profil 2 wechseln"),
    ("profil_3", 0x1A, "Zu Profil 3 wechseln"),
];

pub const MEDIEN: &[(&str, u16, &str)] = &[
    ("browser", 0x223, "Browser-Startseite"),
    ("player", 0x183, "Medienplayer starten"),
    ("mail", 0x18A, "E-Mail-Programm"),
    ("lauter", 0x0E9, "Lautstärke +"),
    ("leiser", 0x0EA, "Lautstärke −"),
    ("zurueck", 0x0B6, "Vorheriger Titel"),
    ("weiter", 0x0B5, "Nächster Titel"),
    ("stumm", 0x0E2, "Stummschalten"),
    ("abspielen", 0x0CD, "Wiedergabe/Pause"),
    ("rechner", 0x192, "Taschenrechner"),
    ("stopp", 0x0B7, "Wiedergabe stoppen"),
];

/// Tastennamen nach deutschem QWERTZ-Aufdruck -> HID-Usage
pub const TASTEN: &[(&str, u8)] = &[
    ("a", 0x04), ("b", 0x05), ("c", 0x06), ("d", 0x07), ("e", 0x08), ("f", 0x09), ("g", 0x0A),
    ("h", 0x0B), ("i", 0x0C), ("j", 0x0D), ("k", 0x0E), ("l", 0x0F), ("m", 0x10), ("n", 0x11),
    ("o", 0x12), ("p", 0x13), ("q", 0x14), ("r", 0x15), ("s", 0x16), ("t", 0x17), ("u", 0x18),
    ("v", 0x19), ("w", 0x1A), ("x", 0x1B), ("z", 0x1C), ("y", 0x1D),
    ("1", 0x1E), ("2", 0x1F), ("3", 0x20), ("4", 0x21), ("5", 0x22), ("6", 0x23), ("7", 0x24),
    ("8", 0x25), ("9", 0x26), ("0", 0x27),
    ("eingabe", 0x28), ("esc", 0x29), ("ruecktaste", 0x2A), ("tab", 0x2B), ("leertaste", 0x2C),
    ("ß", 0x2D), ("´", 0x2E), ("ü", 0x2F), ("+", 0x30), ("#", 0x32), ("ö", 0x33), ("ä", 0x34),
    ("^", 0x35), (",", 0x36), (".", 0x37), ("-", 0x38), ("feststell", 0x39), ("<", 0x64),
    ("f1", 0x3A), ("f2", 0x3B), ("f3", 0x3C), ("f4", 0x3D), ("f5", 0x3E), ("f6", 0x3F),
    ("f7", 0x40), ("f8", 0x41), ("f9", 0x42), ("f10", 0x43), ("f11", 0x44), ("f12", 0x45),
    ("druck", 0x46), ("rollen", 0x47), ("pause", 0x48), ("einfg", 0x49), ("pos1", 0x4A),
    ("bild_auf", 0x4B), ("entf", 0x4C), ("ende", 0x4D), ("bild_ab", 0x4E),
    ("pfeil_rechts", 0x4F), ("pfeil_links", 0x50), ("pfeil_runter", 0x51), ("pfeil_hoch", 0x52),
    ("num", 0x53), ("num_/", 0x54), ("num_*", 0x55), ("num_-", 0x56), ("num_+", 0x57),
    ("num_eingabe", 0x58), ("num_1", 0x59), ("num_2", 0x5A), ("num_3", 0x5B), ("num_4", 0x5C),
    ("num_5", 0x5D), ("num_6", 0x5E), ("num_7", 0x5F), ("num_8", 0x60), ("num_9", 0x61),
    ("num_0", 0x62), ("num_,", 0x63), ("menue", 0x65),
    ("f13", 0x68), ("f14", 0x69), ("f15", 0x6A), ("f16", 0x6B), ("f17", 0x6C), ("f18", 0x6D),
    ("f19", 0x6E), ("f20", 0x6F), ("f21", 0x70), ("f22", 0x71), ("f23", 0x72), ("f24", 0x73),
    ("plus", 0x30),
    ("strg", 0xE0), ("umschalt", 0xE1), ("alt", 0xE2), ("win", 0xE3),
    ("strg_rechts", 0xE4), ("umschalt_rechts", 0xE5), ("altgr", 0xE6), ("win_rechts", 0xE7),
];

fn taste_lesen(name: &str) -> Result<u8> {
    if let Some((_, u)) = TASTEN.iter().find(|(k, _)| *k == name) {
        return Ok(*u);
    }
    if name.starts_with("0x") {
        let v = zahl_lesen(name)?;
        if v == 0 || v > 0xFF {
            bail!("HID-Code {name} muss 0x01–0xFF sein");
        }
        return Ok(v as u8);
    }
    bail!("unbekannte Taste \"{name}\" – `x806 funktionen` zeigt alle Namen, oder 0xNN")
}

/// Zeichen -> (Modifier, HID-Usage) für ein deutsches QWERTZ-Layout
fn zeichen(c: char) -> Option<(Option<u8>, u8)> {
    let einfach = |u| Some((None, u));
    let gross = |u| Some((Some(UMSCHALT), u));
    let altgr = |u| Some((Some(ALTGR), u));
    match c {
        'a'..='z' => {
            let u = match c {
                'y' => 0x1D,
                'z' => 0x1C,
                _ => 0x04 + (c as u8 - b'a'),
            };
            einfach(u)
        }
        'A'..='Z' => zeichen(c.to_ascii_lowercase()).map(|(_, u)| (Some(UMSCHALT), u)),
        '1'..='9' => einfach(0x1E + (c as u8 - b'1')),
        '0' => einfach(0x27),
        '!' => gross(0x1E), '"' => gross(0x1F), '§' => gross(0x20), '$' => gross(0x21),
        '%' => gross(0x22), '&' => gross(0x23), '/' => gross(0x24), '(' => gross(0x25),
        ')' => gross(0x26), '=' => gross(0x27),
        '²' => altgr(0x1F), '³' => altgr(0x20), '{' => altgr(0x24), '[' => altgr(0x25),
        ']' => altgr(0x26), '}' => altgr(0x27),
        'ß' => einfach(0x2D), '?' => gross(0x2D), '\\' => altgr(0x2D),
        'ü' => einfach(0x2F), 'Ü' => gross(0x2F),
        '+' => einfach(0x30), '*' => gross(0x30), '~' => altgr(0x30),
        '#' => einfach(0x32), '\'' => gross(0x32),
        'ö' => einfach(0x33), 'Ö' => gross(0x33),
        'ä' => einfach(0x34), 'Ä' => gross(0x34),
        '°' => gross(0x35),
        ',' => einfach(0x36), ';' => gross(0x36),
        '.' => einfach(0x37), ':' => gross(0x37),
        '-' => einfach(0x38), '_' => gross(0x38),
        '<' => einfach(0x64), '>' => gross(0x64), '|' => altgr(0x64),
        '@' => altgr(0x14), '€' => altgr(0x08), 'µ' => altgr(0x10),
        ' ' => einfach(0x2C), '\n' => einfach(0x28), '\t' => einfach(0x2B),
        _ => None,
    }
}

/// Zerlegt einen Text in Tastenereignisse: (gedrückt, HID-Usage)
pub fn text_ereignisse(text: &str) -> Result<Vec<(bool, u8)>> {
    let mut e = Vec::new();
    for c in text.chars() {
        let Some((modifier, u)) = zeichen(c) else {
            bail!("Zeichen „{c}“ kann nicht getippt werden (nicht unterstützt: ^ ´ ` und Sonderzeichen außerhalb des deutschen Layouts)");
        };
        if let Some(m) = modifier {
            e.push((true, m));
        }
        e.push((true, u));
        e.push((false, u));
        if let Some(m) = modifier {
            e.push((false, m));
        }
    }
    Ok(e)
}

/// Wandelt einen Funktionstext aus der Konfiguration um.
pub fn kodieren(text: &str) -> Result<Funktion> {
    // Bei text: Groß-/Kleinschreibung und Leerzeichen erhalten
    let ohne_einzug = text.trim_start();
    if ohne_einzug.get(..5).is_some_and(|p| p.eq_ignore_ascii_case("text:")) {
        let inhalt = &ohne_einzug[5..];
        if inhalt.is_empty() {
            bail!("text: ist leer");
        }
        if inhalt.chars().count() > TEXT_MAX {
            bail!("text: höchstens {TEXT_MAX} Zeichen");
        }
        text_ereignisse(inhalt)?;
        return Ok(Funktion::Text(inhalt.to_string()));
    }
    let n = text.trim().to_lowercase();
    if let Some(name) = n.strip_prefix("taste:") {
        return Ok(Funktion::Taste(taste_lesen(name)?));
    }
    if let Some(liste) = n.strip_prefix("kombi:") {
        let teile: Vec<&str> = liste.split('+').map(str::trim).collect();
        if teile.len() < 2 || teile.len() > 4 {
            bail!("kombi: braucht 2–4 Tasten, z. B. kombi:strg+c (Plus-Taste heißt \"plus\")");
        }
        let mut k = [0u8; 4];
        for (i, t) in teile.iter().enumerate() {
            k[i] = taste_lesen(t)?;
        }
        return Ok(Funktion::Kombi(k, teile.len() as u8));
    }
    if let Some(name) = n.strip_prefix("medien:") {
        if let Some((_, u, _)) = MEDIEN.iter().find(|(k, _, _)| *k == name) {
            return Ok(Funktion::Medien(*u));
        }
        if name.starts_with("0x") {
            let v = zahl_lesen(name)?;
            if v == 0 || v > 0xFFFF {
                bail!("Consumer-Code {name} muss 0x0001–0xFFFF sein");
            }
            return Ok(Funktion::Medien(v as u16));
        }
        bail!("unbekannte Medienfunktion \"{name}\"");
    }
    if let Some(roh) = n.strip_prefix("roh:") {
        let v = zahl_lesen(roh)?;
        if v > 0x1F || v == CODE_TASTE as u32 || v == CODE_MEDIEN as u32 {
            bail!("Rohcode {roh} ungültig (max. 0x1F, 0x0E/0x0F über taste:/medien:)");
        }
        return Ok(Funktion::Einfach(v as u8));
    }
    match EINFACH.iter().find(|(k, _, _)| *k == n) {
        Some((_, c, _)) => Ok(Funktion::Einfach(*c)),
        None => bail!("unbekannte Funktion \"{text}\" – siehe `x806 funktionen`"),
    }
}

/// Rückübersetzung für `auslesen`
pub fn text_von(f: Funktion) -> String {
    match f {
        Funktion::Text(t) => format!("text:{t}"),
        Funktion::Einfach(c) => EINFACH
            .iter()
            .find(|e| e.1 == c)
            .map(|e| e.0.to_string())
            .unwrap_or_else(|| format!("roh:0x{c:02X}")),
        Funktion::Taste(u) => match TASTEN.iter().find(|t| t.1 == u) {
            Some((k, _)) => format!("taste:{k}"),
            None => format!("taste:0x{u:02X}"),
        },
        Funktion::Kombi(k, n) => {
            let teile: Vec<String> = k[..n as usize]
                .iter()
                .map(|u| match TASTEN.iter().find(|t| t.1 == *u) {
                    // "+" ist das Trennzeichen – die Plus-Taste heißt hier "plus"
                    Some((name, _)) if *name == "+" => "plus".to_string(),
                    Some((name, _)) => name.to_string(),
                    None => format!("0x{u:02X}"),
                })
                .collect();
            format!("kombi:{}", teile.join("+"))
        }
        Funktion::Medien(u) => match MEDIEN.iter().find(|m| m.1 == u) {
            Some((k, _, _)) => format!("medien:{k}"),
            None => format!("medien:0x{u:03X}"),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kombi_mit_plus_bleibt_lesbar() {
        let f = kodieren("kombi:strg+plus").unwrap();
        assert_eq!(text_von(f.clone()), "kombi:strg+plus");
        assert_eq!(kodieren(&text_von(f.clone())).unwrap(), f);
    }

    #[test]
    fn text_deutsches_layout() {
        let f = kodieren("text:Ab@z.de").unwrap();
        assert_eq!(text_von(f.clone()), "text:Ab@z.de");
        let e = text_ereignisse("A@z").unwrap();
        assert_eq!(
            e,
            vec![
                (true, 0xE1), (true, 0x04), (false, 0x04), (false, 0xE1), // A
                (true, 0xE6), (true, 0x14), (false, 0x14), (false, 0xE6), // @ = AltGr+Q
                (true, 0x1C), (false, 0x1C),                              // z
            ]
        );
        assert!(kodieren("text:a^b").is_err());
    }
}
