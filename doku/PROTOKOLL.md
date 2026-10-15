# X806 / K806 – Analyse des Originalprogramms

Gerät: Tastenfeld mit 8 Tasten. Die Firmware ist eine Gaming-Maus-Firmware
(Produktname „USB GAMING MOUSE“, Maus-, Tastatur- und Consumer-Collection).

Quelle: `C:\Program Files (x86)\K806\X806.exe` (32-Bit, MFC, 2,4 MB) samt `skins\`-Ordner.
Methode: Import-Tabelle, Zeichenketten, Disassemblierung der HID-relevanten Funktionen
(Python + pefile + capstone). Etwa 95 % des Programms sind MFC-Oberfläche und Skin-Code;
die eigentliche Geräte-Logik steckt in wenigen Funktionen:

| Adresse (VA) | Aufgabe |
|---|---|
| `0x40D6F0` | Geräte suchen (SetupDi…, HidD_GetAttributes, HidP_GetCaps) |
| `0x40D470` | Block in den Gerätespeicher schreiben |
| `0x413220` | Hintergrund-Thread: Profilwechsel, Feuer-Taste, Status pollen |
| `0x413A10` | „Übernehmen“: komplette Konfiguration kodieren und senden |
| `0x421xxx`/`0x42Bxxx` | `config.ini` laden/speichern |

## Gerät

* VID `0x30FA`, PID `0x1340`, Produktname „USB GAMING MOUSE“
* Interface 1, Collection mit **Usage-Page 0xFF01 / Usage 1** → alle Befehle (Feature-Reports)
* Usage-Page 0xFF00 wird vom Original geöffnet, aber nicht benutzt
* Feature-Report-Länge: **8 Byte** inkl. Report-ID (am Gerät bestätigt)

## Berichtsformat (Report-ID 7)

```
Byte 0  0x07           Report-ID
Byte 1  Befehl         0x18 Speicher, 0x11 Feuer-Steuerung, 0x20 Freigabe beim Beenden
Byte 2  Unterbefehl    (bei 0x18) 3 puffern · 9 übernehmen · 5 lesen · 0x10 Flash speichern · 0 Leerlauf
Byte 3  Index im Block
Byte 4  Adresse low
Byte 5  Adresse high
Byte 6  Datenbyte
Byte 7  Blocklänge − 1
```

**Block schreiben** (n ≤ 8 Byte an Adresse A):
`03 i A d[i] n-1` für jedes Byte → 4 ms → `09 0 A 0 n-1` → 2 ms → `00 0 A 0 0`

**Byte lesen**: `03 0 A 0 0` → `00 0 A` → `05 0 A` → GET_FEATURE, Antwort in Byte 1 → `00`

**Dauerhaft speichern**: `10 0 0 0 0` → 20 ms → `00`

**Status** (GET_FEATURE ohne Befehl): Byte 6 Bit 2–3 = aktives Profil, Bit 1 = großer Makrospeicher (bis 0xFF8 statt 0x7F8).

## Speicherkarte 0x00–0x7F

| Adresse | Inhalt |
|---|---|
| `0x01` | `Profil<<5 \| 3` |
| `0x08–0x0F` | Profil 1, Steckplatz 1–8 |
| `0x10` | Profil 1, Steckplatz 9 |
| `0x13` | Bit 4 der sechs DPI-Codes (Bit *i* = Stufe *i*) – DPI-Bereich fest `38 08 6C F2 02 18` (Maus, ohne Wirkung) |
| `0x14` | konstant `0x08` |
| `0x15–0x17` | DPI-Codes, je zwei Nibble (Stufe 1/2, 3/4, 5/6) |
| `0x18` | `0x18 \| (aktive DPI-Stufe − 1)` |
| `0x19` | 4 Bits: Farben im Farbzyklus |
| `0x1A–0x1B` | Dauerfeuer-Intervall / 8 (LE16) |
| `0x1C` | LED: `0x80 \| Modus<<4 \| Tastenreaktion<<3 \| Tempo` |
| `0x1D` | DPI-Stufe aktiv (Bits 0,2,4,6 für Stufe 1–4, fest `0x55`) \| Benutzerfarbe<<7 |
| `0x1E` | Abfragerate-Code (0–3) |
| `0x20–0x31` | DPI-Farben: 6 × (R,G,B), je 4 Bit (fest, Standardwerte des Originals) |
| `0x32–0x33` | konstant `07 30` |
| `0x3A` | LED-Effekte: Einzelfarbe<<7 \| Symmetrie<<6 \| Richtung<<4 \| unbekanntes Nibble |
| `0x3B–0x3C` | Profil 1, Steckplatz 10–11 |
| `0x3D` | Kopie Profil 2 / Steckplatz 1 |
| `0x3F` | Helligkeit<<4 |
| `0x43` | unbekannt, `0xFC` oder `0xFD` |
| `0x44–0x4E` | Profil 2, Steckplatz 1–11 |
| `0x50–0x5A` | Profil 3, Steckplatz 1–11 |
| `0x60–0x6A` | Profil 4, Steckplatz 1–11 |
| `0x6E–0x6F` | unbekannter 16-Bit-Wert |
| `0x70–0x75` | drei unbekannte 16-Bit-Werte |
| `0x76` | Geräteflags; Bit 0–1 = aktives Profil (wird gelesen, verändert, zurückgeschrieben) |
| `0xA0–0x2CF` | Tastentabelle: 10-Byte-Eintrag je (Hardware-Profil × 14 + Steckplatz − 1) |
| `0x400–0x7F7` | Tastendaten / Makros (je Taste auf 8 Byte aufgerundet) |

### Tastenbyte

`LED-Reaktionstyp<<6 | LED-Reaktion aktiv<<5 | Funktionscode (5 Bit)`

| Code | Funktion | Code | Funktion |
|---|---|---|---|
| 0x01 | Linksklick | 0x0B | DPI + |
| 0x02 | Mittelklick | 0x0C | DPI − |
| 0x03 | Rechtsklick | 0x0D | Tastenkürzel (braucht Makrodaten) |
| 0x04 | Zurück | 0x0E | Makro / Einzeltaste (braucht Makrodaten) |
| 0x05 | Vor | 0x0F | Multimedia (braucht Makrodaten) |
| 0x06 | DPI-Zyklus | 0x14 | Fn |
| 0x07 | Desktop anzeigen | 0x15 | LED-Zyklus |
| 0x08 | Doppelklick | 0x17/0x19/0x1A | Profil 1/2/3 |
| 0x09 | Dauerfeuer | 0x1E/0x1F | Makro-Wiederholmodi |

### Steckplätze (aus `config.ini`, `[KEYMAP]`)

| Taste | Steckplatz | Standard |
|---|---|---|
| Links | 7 | Linksklick |
| Rechts | 3 | Rechtsklick |
| Mitte | 2 | Mittelklick |
| Vor | 1 | Vor |
| Zurück | 9 | Zurück |
| DPI + | 6 | DPI + |
| DPI − | 5 | DPI − |
| Feuer | 4 | Dauerfeuer |

## Nicht in der Maus gespeichert

„Zeigergenauigkeit“ (`DPI_SPEED`) und „Mausrad-Geschwindigkeit“ (`WHEEL_SPEED`) setzt das Original
per `SystemParametersInfo(SPI_SETMOUSESPEED / SPI_SETWHEELSCROLLLINES)` in Windows – hier nicht umgesetzt.

## Unsicherheiten

* **Abfragerate**: Code 0–3 ist sicher, die Zuordnung 0 = 1000 Hz … 3 = 125 Hz ist abgeleitet, nicht gemessen.
* **LED-Modus**: Reihenfolge stammt aus der Auswahlliste des Originals
  (DPI breath, Cycle breath, Constantly, Button response, Mute, Single Breath).
* **Lesen**: Der Leseablauf ist aus dem Original übernommen, wurde aber nur für 0x76 beobachtet.
* Felder mit „unbekannt“ werden mit den Werten befüllt, die das Original im Normalfall sendet.

## Profile

Das Original zeigt drei Profile; sie liegen in den Hardware-Profilen **0, 2 und 3**.
Hardware-Profil 1 ist verborgen und bekommt keine Tabelleneinträge.
Profilwechsel-Codes: `0x17` → 0, `0x19` → 2, `0x1A` → 3.

## Tastentabelle (ab 0xA0, am Gerät verifiziert)

```
Byte 0  Abfrageraten-Code (Kopie von 0x1E)
Byte 1  Anzahl Ereignisse (Bit 7: Wiederholmodus)
Byte 2  HID-Tastencode bei Einzeltaste, sonst 0
Byte 3-4 HID-Consumer-Code (Medientaste), LE
Byte 5  Datenadresse low
Byte 6  Datenadresse high (Bit 0-5) | Wiederholflags (Bit 6-7)
Byte 7  Anzahl Ereignisse >> 7 | Flag
Byte 8  0x0A bei Makros, sonst 0
Byte 9  0
```

Funktionscode im Tastenbyte (Original `0x413E55`): `0x0E` nur Einzeltaste (Firmware liest nur Byte 2),
`0x0D` Tastenkürzel, `0x1E` Benutzermakro (`0x1F` = Wiederholmodus), `0x0F` Medien.
Mehrere Ereignisse unter `0x0E` werden von der Firmware ignoriert.

## Tastendaten (ab 0x400)

```
01 00                  Kopf
[P<<7 | Verzögerung] [HID-Code]   je Ereignis, P = 1 drücken / 0 loslassen
```

Einzeltaste: `01 00 80 <Code> 00 00 00 00` (Loslassen übernimmt die Firmware).

Multimedia-Codes des Originals: Browser 0x223, Player 0x183, Mail 0x18A, Lauter 0xE9,
Leiser 0xEA, Zurück 0xB6, Weiter 0xB5, Stumm 0xE2, Play/Pause 0xCD, Rechner 0x192, Stopp 0xB7.
