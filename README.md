# x806 – Konfigurator für das K806/X806-Tastenfeld (8 Tasten)

Neu geschriebener Ersatz für `X806.exe`: grafische Oberfläche mit Tauri in [`gui/`](gui/).

## Aufbau

* [`gui/`](gui/) – die App (Tauri v2 + TypeScript), Entwickeln/Bauen siehe [`gui/README.md`](gui/README.md)
* [`gui/src-tauri/src/geraet/`](gui/src-tauri/src/geraet/) – Gerätelogik (HID-Protokoll, Speicherkarte, TOML-Format)
* [`gui/src-tauri/src/geraet/konfiguration.toml`](gui/src-tauri/src/geraet/konfiguration.toml) – Standardkonfiguration, wird ins Programm eingebettet

## Schnellstart

```bash
cd gui
pnpm install
pnpm tauri dev
```

Tests der Gerätelogik: `cargo test` in `gui/src-tauri`.

Das Original-Programm muss geschlossen sein, sonst blockiert es die Schnittstelle.

## Tastenfunktionen

* `taste:a`, `taste:f13`, `taste:eingabe` … – eine Taste (deutsches Layout)
* `kombi:strg+c`, `kombi:strg+umschalt+esc` – 2–4 Tasten gleichzeitig (Plus-Taste: `plus`)
* `text:name@beispiel.de` – tippt einen Text (max. 120 Zeichen, deutsches Layout, Zeilenumbruch = Enter; nicht möglich: Akzent-Tasten ^ ´ und Gravis) – Tempo über `[allgemein] tippverzoegerung` (1–127, Standard 15)
* `medien:lauter`, `medien:abspielen` … – Multimedia
* `profil_1` … `profil_3`, `led_zyklus`, Mausfunktionen

## Konfigurationsformat

Siehe [`konfiguration.toml`](gui/src-tauri/src/geraet/konfiguration.toml).

* `[allgemein]` – aktives Profil, Abfragerate
* `[beleuchtung]` – Modus, Helligkeit, Tempo, Effekte
* `[[taste]]` – frei benannte Tasten → Firmware-Steckplatz
* `[[profil]]` – bis zu 3 Profile
* `[erweitert]` – unbekannte Felder und direkte Speicherbytes

Die Firmware stammt aus einer Gaming-Maus. Deren DPI-Bereich im Speicher wird fest mit den
Standardwerten des Originals beschrieben und ist nicht konfigurierbar.

## Nicht unterstützt

Makros mit Verzögerungen oder Wiederholung. Bestehende Makros des Originals werden
beim Auslesen als „aus“ übernommen.

Protokoll und Speicherkarte: [`doku/PROTOKOLL.md`](doku/PROTOKOLL.md).
