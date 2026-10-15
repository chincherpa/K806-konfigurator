// Kein Konsolenfenster im Release-Build unter Windows
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod geraet;

use std::path::PathBuf;
use std::sync::Mutex;

use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};

use geraet::aktionen::{self, GeraeteStatus};
use geraet::funktionen;
use geraet::konfig::{Konfiguration, Taste};

/// Serialisiert alle Gerätezugriffe – das HID-Gerät verträgt keine parallelen Befehle.
struct Geraetesperre(Mutex<()>);

type Ergebnis<T> = Result<T, String>;

fn fehler(e: anyhow::Error) -> String {
    format!("{e:#}")
}

fn konfig_pfad(app: &AppHandle) -> Ergebnis<PathBuf> {
    let ordner = app.path().app_config_dir().map_err(|e| e.to_string())?;
    std::fs::create_dir_all(&ordner).map_err(|e| e.to_string())?;
    Ok(ordner.join("konfiguration.toml"))
}

fn als_toml(k: &Konfiguration) -> Ergebnis<String> {
    Ok(format!(
        "# K806 / X806 – gespeichert vom K806 Konfigurator\n{}",
        toml::to_string_pretty(k).map_err(|e| e.to_string())?
    ))
}

#[derive(Serialize)]
struct Eintrag {
    name: &'static str,
    code: u16,
    text: &'static str,
}

#[derive(Serialize)]
struct Katalog {
    tasten: Vec<Eintrag>,
    medien: Vec<Eintrag>,
    einfach: Vec<Eintrag>,
}

#[tauri::command]
fn katalog() -> Katalog {
    Katalog {
        tasten: funktionen::TASTEN.iter().map(|(n, c)| Eintrag { name: n, code: *c as u16, text: "" }).collect(),
        medien: funktionen::MEDIEN.iter().map(|(n, c, t)| Eintrag { name: n, code: *c, text: t }).collect(),
        einfach: funktionen::EINFACH.iter().map(|(n, c, t)| Eintrag { name: n, code: *c as u16, text: t }).collect(),
    }
}

/// Prüft einen Funktionstext und gibt ihn in Normalform zurück.
#[tauri::command]
fn funktion_pruefen(text: String) -> Ergebnis<String> {
    funktionen::kodieren(&text).map(funktionen::text_von).map_err(fehler)
}

#[tauri::command]
fn konfig_laden(app: AppHandle) -> Ergebnis<Konfiguration> {
    let pfad = konfig_pfad(&app)?;
    if pfad.exists() {
        Konfiguration::laden(&pfad).map_err(fehler)
    } else {
        Ok(aktionen::beispiel())
    }
}

#[tauri::command]
fn konfig_speichern(app: AppHandle, konfiguration: Konfiguration) -> Ergebnis<()> {
    konfiguration.pruefen().map_err(fehler)?;
    std::fs::write(konfig_pfad(&app)?, als_toml(&konfiguration)?).map_err(|e| e.to_string())
}

#[tauri::command]
fn sprache_setzen(englisch: bool) {
    geraet::sprache::setzen(englisch);
}

#[tauri::command]
fn beispiel() -> Konfiguration {
    aktionen::beispiel()
}

#[tauri::command]
fn datei_oeffnen(pfad: PathBuf) -> Ergebnis<Konfiguration> {
    Konfiguration::laden(&pfad).map_err(fehler)
}

#[tauri::command]
fn datei_speichern(pfad: PathBuf, konfiguration: Konfiguration) -> Ergebnis<()> {
    konfiguration.pruefen().map_err(fehler)?;
    std::fs::write(pfad, als_toml(&konfiguration)?).map_err(|e| e.to_string())
}

#[tauri::command]
async fn status(app: AppHandle) -> Ergebnis<GeraeteStatus> {
    tauri::async_runtime::spawn_blocking(move || {
        let sperre = app.state::<Geraetesperre>();
        let _g = sperre.0.lock().unwrap();
        aktionen::status(&Default::default()).map_err(fehler)
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn anwenden(app: AppHandle, konfiguration: Konfiguration) -> Ergebnis<()> {
    tauri::async_runtime::spawn_blocking(move || {
        let sperre = app.state::<Geraetesperre>();
        let _g = sperre.0.lock().unwrap();
        aktionen::anwenden(&konfiguration, false, false, &mut |t| {
            let _ = app.emit("meldung", t);
        })
        .map_err(fehler)
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn auslesen(app: AppHandle, tasten: Vec<Taste>) -> Ergebnis<Konfiguration> {
    tauri::async_runtime::spawn_blocking(move || {
        let sperre = app.state::<Geraetesperre>();
        let _g = sperre.0.lock().unwrap();
        let konfiguration = aktionen::auslesen(&Default::default(), &tasten, false, &mut |n, gesamt| {
            if n % 16 == 0 || n == gesamt {
                let _ = app.emit("fortschritt", (n * 100 / gesamt) as u32);
            }
        })
        .map_err(fehler)?;
        Ok(konfiguration)
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn profil_aktivieren(app: AppHandle, nummer: u8) -> Ergebnis<()> {
    tauri::async_runtime::spawn_blocking(move || {
        let sperre = app.state::<Geraetesperre>();
        let _g = sperre.0.lock().unwrap();
        aktionen::profil(&Default::default(), nummer).map_err(fehler)
    })
    .await
    .map_err(|e| e.to_string())?
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(Geraetesperre(Mutex::new(())))
        .invoke_handler(tauri::generate_handler![
            katalog,
            funktion_pruefen,
            konfig_laden,
            konfig_speichern,
            beispiel,
            datei_oeffnen,
            datei_speichern,
            status,
            anwenden,
            auslesen,
            profil_aktivieren,
            sprache_setzen,
        ])
        .run(tauri::generate_context!())
        .expect("K806 Konfigurator konnte nicht gestartet werden");
}
