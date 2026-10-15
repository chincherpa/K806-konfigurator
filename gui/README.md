# K806 Konfigurator (GUI)

Grafische Oberfläche für das K806/X806-Tastenfeld. Tauri v2 + TypeScript (ohne Framework),
die Gerätelogik liegt in `src-tauri/src/geraet/`.

## Entwickeln

```bash
pnpm install
pnpm tauri dev
```

## Bauen

```bash
pnpm tauri build --no-bundle
```

Ergebnis: `src-tauri/target/release/k806-konfigurator.exe`.
Mit Installer (NSIS, lädt beim ersten Mal NSIS nach): `pnpm tauri build`.

## Bedienung

* **Profile** links: Name ändern, Startprofil wählen, „Aktivieren“ wechselt sofort am Gerät.
* **Taste anklicken** → rechts belegen:
  * *Taste*: „Taste drücken zum Aufnehmen“ oder aus der Liste (F13–F24 nur über die Liste)
  * *Kombination*: z. B. Strg + Umschalt + S gleichzeitig drücken, oder `strg+umschalt+s` tippen
  * *Medien*: Lautstärke, Wiedergabe, Rechner …
  * *Sonstige*: Mausfunktionen, Profilwechsel, LED-Zyklus
* **Beleuchtung / Abfragerate** unten.
* **Auf Gerät übertragen** schreibt alles ins Gerät (bleibt nach Abstecken erhalten).
* **Vom Gerät lesen** holt die aktuelle Belegung aus dem Gerät.

Änderungen werden automatisch gespeichert unter
`%APPDATA%\de.x806.konfigurator\konfiguration.toml`.
Das Original-Programm `X806.exe` muss geschlossen sein.
