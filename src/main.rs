//! x806 – Konfigurationswerkzeug für das K806/X806-Tastenfeld (8 Tasten).

use std::io::Write;
use std::path::PathBuf;

use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand};

use x806_konfig::aktionen::{self, BEISPIEL};
use x806_konfig::funktionen;
use x806_konfig::konfig::{zahl_lesen, Geraet, Konfiguration};
use x806_konfig::protokoll::{self, hex};
use x806_konfig::speicher;

#[derive(Parser)]
#[command(name = "x806", version, about = "Konfiguriert das K806/X806-Tastenfeld über eine TOML-Datei")]
struct Cli {
    /// Jeden gesendeten HID-Bericht anzeigen
    #[arg(short, long, global = true)]
    ausfuehrlich: bool,
    #[command(subcommand)]
    befehl: Befehl,
}

#[derive(Subcommand)]
enum Befehl {
    /// Angeschlossene Schnittstellen auflisten
    Geraete,
    /// Aktives Profil und Rohstatus anzeigen
    Status,
    /// Konfigurationsdatei auf das Gerät übertragen
    Anwenden {
        #[arg(default_value = "konfiguration.toml")]
        datei: PathBuf,
        /// Nichts senden, nur die Berichte anzeigen
        #[arg(long)]
        trocken: bool,
    },
    /// Nur das aktive Profil wechseln (1–3)
    Profil { nummer: u8 },
    /// Gerätespeicher auslesen und als Konfiguration speichern
    Auslesen {
        #[arg(short = 'o', long, default_value = "ausgelesen.toml")]
        ausgabe: PathBuf,
        /// Zusätzlich Hexdump ausgeben
        #[arg(long)]
        roh: bool,
    },
    /// Bytes ab einer Adresse lesen (Hexdump)
    Lesen {
        /// z. B. 0x400
        adresse: String,
        #[arg(default_value = "16")]
        anzahl: u16,
    },
    /// Rohblock (1–8 Byte) an eine Adresse schreiben und speichern
    Schreiben {
        /// z. B. 0x18
        adresse: String,
        /// Bytes, z. B. 0x18 0 0x06 0
        bytes: Vec<String>,
    },
    /// Verfügbare Tastenfunktionen anzeigen
    Funktionen,
    /// Beispielkonfiguration erzeugen
    Beispiel {
        #[arg(default_value = "konfiguration.toml")]
        datei: PathBuf,
    },
}

fn main() {
    if let Err(e) = ausfuehren() {
        eprintln!("Fehler: {e:#}");
        std::process::exit(1);
    }
}

fn hexdump(start: usize, daten: &[u8]) {
    for (z, zeile) in daten.chunks(16).enumerate() {
        println!("{:04X}: {}", start + z * 16, hex(zeile));
    }
}

fn ausfuehren() -> Result<()> {
    let cli = Cli::parse();
    let g = Geraet::default();

    match &cli.befehl {
        Befehl::Geraete => {
            let liste = protokoll::auflisten(g.vendor_id, g.product_id)?;
            if liste.is_empty() {
                println!("Kein K806/X806 gefunden.");
            }
            for z in liste {
                println!("{z}");
            }
        }
        Befehl::Status => {
            let s = aktionen::status(&g)?;
            match s.profil {
                Some(p) => println!("Aktives Profil : {p}"),
                None => println!("Aktives Profil : verborgenes Hardware-Profil {}", s.hardware_profil),
            }
            println!("Makrospeicher  : bis 0x{:03X}", s.makro_ende);
            println!("Rohdaten       : {}", s.roh);
        }
        Befehl::Anwenden { datei, trocken } => {
            let k = Konfiguration::laden(datei)?;
            aktionen::anwenden(&k, *trocken, cli.ausfuehrlich, &mut |t| println!("{t}"))?;
        }
        Befehl::Profil { nummer } => {
            aktionen::profil(&g, *nummer)?;
            println!("Profil {nummer} aktiviert.");
        }
        Befehl::Auslesen { ausgabe, roh } => {
            let a = aktionen::auslesen(&g, &aktionen::beispiel().taste, cli.ausfuehrlich, &mut |n, gesamt| {
                if n % 64 == 0 || n == gesamt {
                    print!("\rLese … {:3} %", n * 100 / gesamt);
                    let _ = std::io::stdout().flush();
                }
            })?;
            println!();
            if *roh {
                println!("Grundeinstellungen:");
                hexdump(0, &a.grundeinstellungen);
                println!("Tastentabelle:");
                hexdump(speicher::TABELLE_START as usize, &a.tabelle);
            }
            let text = format!(
                "# Aus dem Gerät ausgelesen – Tastennamen stammen aus der Beispielkonfiguration.\n{}",
                toml::to_string_pretty(&a.konfiguration)?
            );
            std::fs::write(ausgabe, text)?;
            println!("Gespeichert in {}", ausgabe.display());
        }
        Befehl::Lesen { adresse, anzahl } => {
            let start = u16::try_from(zahl_lesen(adresse)?).context("Adresse zu groß")?;
            hexdump(start as usize, &aktionen::lesen(&g, start, *anzahl)?);
        }
        Befehl::Schreiben { adresse, bytes } => {
            let adr = zahl_lesen(adresse)?;
            let daten = bytes
                .iter()
                .map(|b| zahl_lesen(b).and_then(|v| u8::try_from(v).context("Byte > 255")))
                .collect::<Result<Vec<u8>>>()?;
            aktionen::schreiben(&g, u16::try_from(adr).context("Adresse zu groß")?, &daten)?;
            println!("{} Byte an 0x{adr:04X} geschrieben.", daten.len());
        }
        Befehl::Funktionen => {
            println!("Tastaturtasten (deutsches Layout):  taste:<name>  oder  taste:0xNN (HID-Code)");
            println!("Tastenkombination:                  kombi:strg+c  (2–4 Tasten, Plus-Taste = plus)");
            println!("Text tippen:                        text:name@beispiel.de  (max. 120 Zeichen, deutsches Layout)");
            let namen: Vec<&str> = funktionen::TASTEN.iter().map(|t| t.0).collect();
            for zeile in namen.chunks(12) {
                println!("    {}", zeile.join("  "));
            }
            println!("\nMedientasten:  medien:<name>  oder  medien:0xNNN (Consumer-Code)");
            for (name, code, text) in funktionen::MEDIEN {
                println!("    {name:<12} 0x{code:03X}  {text}");
            }
            println!("\nSonstige:");
            for (name, code, text) in funktionen::EINFACH {
                println!("    {name:<18} 0x{code:02X}  {text}");
            }
            println!("    {:<18} {:4}  beliebiger 5-Bit-Firmwarecode", "roh:0xNN", "");
        }
        Befehl::Beispiel { datei } => {
            if datei.exists() {
                bail!("{} existiert bereits", datei.display());
            }
            std::fs::write(datei, BEISPIEL)?;
            println!("Beispielkonfiguration geschrieben: {}", datei.display());
        }
    }
    Ok(())
}
