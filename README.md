# x806 – Konfigurator für das K806/X806-Tastenfeld (8 Tasten)

Neu geschriebener Ersatz für `X806.exe`, in Rust, auf Deutsch, gesteuert über **eine** TOML-Datei.

## Bauen

```bash
cargo build --release
```

Ergebnis: `target\release\x806.exe`

## Benutzung

```bash
x806 geraete                        # Tastenfeld gefunden?
x806 status                         # aktives Profil anzeigen
x806 beispiel meine.toml            # Vorlage erzeugen
x806 anwenden meine.toml --trocken  # nur anzeigen, was gesendet würde
x806 anwenden meine.toml            # aufs Tastenfeld übertragen
x806 profil 2                       # nur Profil wechseln (1–3)
x806 funktionen                     # alle Tastenfunktionen
x806 auslesen -o aktuell.toml       # Gerät auslesen und als TOML speichern
x806 lesen 0x400 64                 # Hexdump ab Adresse
x806 schreiben 0x38 0 0 0 0 0 0 0 0x40   # Experten: Rohblock schreiben
```

`-a` / `--ausfuehrlich` zeigt jeden HID-Bericht.

Das Original-Programm muss geschlossen sein, sonst blockiert es die Schnittstelle.

## Tastenfunktionen

* `taste:a`, `taste:f13`, `taste:eingabe` … – eine Taste (deutsches Layout, Liste: `x806 funktionen`)
* `kombi:strg+c`, `kombi:strg+umschalt+esc` – 2–4 Tasten gleichzeitig (Plus-Taste: `plus`)
* `text:name@beispiel.de` – tippt einen Text (max. 120 Zeichen, deutsches Layout, Zeilenumbruch = Enter; nicht möglich: Akzent-Tasten ^ ´ und Gravis) – Tempo über `[allgemein] tippverzoegerung` (1–127, Standard 15)
* `medien:lauter`, `medien:abspielen` … – Multimedia
* `profil_1` … `profil_3`, `led_zyklus`, Mausfunktionen

## Konfiguration

Siehe [`konfiguration.toml`](konfiguration.toml): drei Beispielprofile (Ziffern 1–8, Medien, F13–F20).

* `[allgemein]` – aktives Profil, Abfragerate
* `[beleuchtung]` – Modus, Helligkeit, Tempo, Effekte
* `[[taste]]` – frei benannte Tasten → Firmware-Steckplatz
* `[[profil]]` – bis zu 3 Profile
* `[erweitert]` – unbekannte Felder und direkte Speicherbytes

Die Firmware stammt aus einer Gaming-Maus; DPI-Einstellungen (`[dpi]`) werden deshalb
mitgesendet, haben aber keine Wirkung und können weggelassen werden.

## Nicht unterstützt

Makros mit Verzögerungen oder Wiederholung. Bestehende Makros des Originals werden
beim Auslesen als „aus“ übernommen.

Protokoll und Speicherkarte: [`doku/PROTOKOLL.md`](doku/PROTOKOLL.md).
