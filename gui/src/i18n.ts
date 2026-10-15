// Zweisprachige Oberfläche (Deutsch/Englisch).
// Statische Texte in index.html tragen data-i18n*-Attribute, dynamische holen sich ihre Texte über t().

export type Sprache = "de" | "en";

const DE = {
  "app.titel": "K806 Konfigurator",
  "kopf.profil": "Profil",
  "kopf.geraetestatus": "Gerätestatus",
  "kopf.suche": "Suche Gerät …",
  "kopf.verbunden": "Verbunden",
  "kopf.getrennt": "Nicht verbunden",
  "kopf.menue": "Weitere Aktionen",
  "menue.auslesen": "Vom Gerät lesen",
  "menue.aktivieren": "Profil am Gerät aktivieren",
  "menue.oeffnen": "Konfiguration öffnen …",
  "menue.speichernUnter": "Konfiguration speichern unter …",
  "menue.sprache": "Sprache",
  "buehne.titel": "Tastenfeld",

  "editor.titel": "Taste {n} anpassen",
  "editor.aktionstyp": "Aktionstyp",
  "art.kombi": "Tastenkombination",
  "art.taste": "Einzelne Taste",
  "art.text": "Text tippen",
  "art.medien": "Medien",
  "art.sonstige": "Sonstige Funktion",
  "art.keine": "Keine",
  "editor.kombi": "Tasten-Kombi",
  "editor.taste": "Taste",
  "editor.ausListe": "Aus Liste (z. B. F13–F24)",
  "editor.text": "Text",
  "editor.textBeispiel": "name@beispiel.de",
  "editor.textHinweis": "Wird wie getippt eingegeben (deutsches Tastaturlayout). Zeilenumbruch = Enter. Nicht möglich: ^ ´ `",
  "editor.medien": "Medienfunktion",
  "editor.sonstige": "Funktion",
  "editor.name": "Namen",
  "editor.optional": "(optional)",
  "editor.waehlen": "– wählen –",

  "aufnahme.start": "Aufnehmen",
  "aufnahme.stopp": "Stopp",
  "aufnahme.hoert": "Ich höre zu …",
  "aufnahme.hinweisKombi": "„Aufnehmen“ und Kombination gleichzeitig drücken.",
  "aufnahme.hinweisTaste": "„Aufnehmen“ und gewünschte Taste drücken.",
  "aufnahme.nichtUnterstuetzt": "„{code}“ wird nicht unterstützt.",
  "aufnahme.ohneModifier": "Für eine Kombination bitte z. B. Strg dazu halten – oder Aktionstyp „Einzelne Taste“.",

  "titel.kombi": "Tastenkombination",
  "titel.taste": "Taste",
  "titel.text": "Text",
  "titel.medien": "Medien",
  "titel.sonstige": "Funktion",
  "titel.keine": "Funktion wählen",

  "funktion.keine": "Keine Funktion",
  "funktion.taste": "Taste {name}",
  "funktion.text": "„{text}“",
  "kappe.aria": "Taste {n}: {text}",
  "profil.tab": "Profil {n}",
  "profil.live": " – gerade am Gerät aktiv",

  "einst.titel": "Beleuchtung & Gerät",
  "einst.lichtmodus": "Lichtmodus",
  "led.dauerlicht": "Dauerlicht",
  "led.zyklus_atmen": "Zyklus-Atmen",
  "led.einzel_atmen": "Einzel-Atmen",
  "led.tastenreaktion": "Tastenreaktion",
  "led.aus": "Aus",
  "einst.helligkeit": "Helligkeit",
  "einst.tempo": "Tempo",
  "einst.abfragerate": "Abfragerate",
  "einst.tippverzoegerung": "Tippverzögerung",
  "einst.tippverzoegerungHinweis":
    "Pause je Tastenereignis beim Tippen von Text. Kleiner = schneller; zu schnell gehen Zeichen verloren. Original: 15",

  "fuss.werk": "Werkseinstellungen",
  "fuss.abbrechen": "Abbrechen",
  "fuss.anwenden": "Speichern & Anwenden",

  "meldung.ungespeichert": "Ungespeicherte Änderungen",
  "meldung.speichere": "Speichere und übertrage …",
  "meldung.gespeichertAktiv": "Gespeichert und übertragen – Profil {n} ist aktiv.",
  "meldung.fehler": "Fehler: {fehler}",
  "meldung.verworfen": "Änderungen verworfen.",
  "meldung.lese": "Lese Gerätespeicher …",
  "meldung.gelesen": "Aus dem Gerät gelesen. Makros des Original-Programms erscheinen als „Keine Funktion“.",
  "meldung.lesenFehler": "Lesen fehlgeschlagen: {fehler}",
  "meldung.aktiviert": "Profil {n} am Gerät aktiviert.",
  "meldung.aktivierenFehler": "Profilwechsel fehlgeschlagen: {fehler}",
  "meldung.geoeffnet": "Geöffnet: {pfad} – mit „Speichern & Anwenden“ übernehmen.",
  "meldung.gespeichert": "Gespeichert: {pfad}",
  "meldung.konfigFehlerhaft": "Gespeicherte Konfiguration fehlerhaft, Beispiel geladen: {fehler}",

  "dialog.werkTitel": "Werkseinstellungen",
  "dialog.werkText":
    "Alle drei Profile auf die Werksbelegung zurücksetzen?\n\nProfil 1: Ziffern 1–8 · Profil 2: Medien · Profil 3: F13–F20\n\nDas Gerät ändert sich erst mit „Speichern & Anwenden“.",
  "dialog.lesenTitel": "Vom Gerät lesen",
  "dialog.lesenText": "Die Belegung im Fenster wird durch die Daten aus dem Gerät ersetzt. Fortfahren?",
  "datei.filter": "Konfiguration",
};

type Schluessel = keyof typeof DE;

const EN: Record<Schluessel, string> = {
  "app.titel": "K806 Configurator",
  "kopf.profil": "Profile",
  "kopf.geraetestatus": "Device status",
  "kopf.suche": "Searching for device …",
  "kopf.verbunden": "Connected",
  "kopf.getrennt": "Not connected",
  "kopf.menue": "More actions",
  "menue.auslesen": "Read from device",
  "menue.aktivieren": "Activate profile on device",
  "menue.oeffnen": "Open configuration …",
  "menue.speichernUnter": "Save configuration as …",
  "menue.sprache": "Language",
  "buehne.titel": "Keypad",

  "editor.titel": "Customise key {n}",
  "editor.aktionstyp": "Action type",
  "art.kombi": "Key combination",
  "art.taste": "Single key",
  "art.text": "Type text",
  "art.medien": "Media",
  "art.sonstige": "Other function",
  "art.keine": "None",
  "editor.kombi": "Key combo",
  "editor.taste": "Key",
  "editor.ausListe": "From list (e.g. F13–F24)",
  "editor.text": "Text",
  "editor.textBeispiel": "name@example.com",
  "editor.textHinweis": "Typed as on a German keyboard layout. Line break = Enter. Not possible: ^ ´ `",
  "editor.medien": "Media function",
  "editor.sonstige": "Function",
  "editor.name": "Name",
  "editor.optional": "(optional)",
  "editor.waehlen": "– choose –",

  "aufnahme.start": "Record",
  "aufnahme.stopp": "Stop",
  "aufnahme.hoert": "Listening …",
  "aufnahme.hinweisKombi": "Click “Record” and press the combination.",
  "aufnahme.hinweisTaste": "Click “Record” and press the desired key.",
  "aufnahme.nichtUnterstuetzt": "“{code}” is not supported.",
  "aufnahme.ohneModifier": "For a combination hold e.g. Ctrl as well – or use action type “Single key”.",

  "titel.kombi": "Key combination",
  "titel.taste": "Key",
  "titel.text": "Text",
  "titel.medien": "Media",
  "titel.sonstige": "Function",
  "titel.keine": "Choose function",

  "funktion.keine": "No function",
  "funktion.taste": "Key {name}",
  "funktion.text": "“{text}”",
  "kappe.aria": "Key {n}: {text}",
  "profil.tab": "Profile {n}",
  "profil.live": " – currently active on device",

  "einst.titel": "Lighting & device",
  "einst.lichtmodus": "Light mode",
  "led.dauerlicht": "Steady",
  "led.zyklus_atmen": "Breathing cycle",
  "led.einzel_atmen": "Single breathing",
  "led.tastenreaktion": "Key reaction",
  "led.aus": "Off",
  "einst.helligkeit": "Brightness",
  "einst.tempo": "Speed",
  "einst.abfragerate": "Polling rate",
  "einst.tippverzoegerung": "Typing delay",
  "einst.tippverzoegerungHinweis":
    "Pause per key event when typing text. Lower = faster; too fast and characters get lost. Original: 15",

  "fuss.werk": "Factory defaults",
  "fuss.abbrechen": "Cancel",
  "fuss.anwenden": "Save & apply",

  "meldung.ungespeichert": "Unsaved changes",
  "meldung.speichere": "Saving and transferring …",
  "meldung.gespeichertAktiv": "Saved and transferred – profile {n} is active.",
  "meldung.fehler": "Error: {fehler}",
  "meldung.verworfen": "Changes discarded.",
  "meldung.lese": "Reading device memory …",
  "meldung.gelesen": "Read from device. Macros from the original software appear as “No function”.",
  "meldung.lesenFehler": "Reading failed: {fehler}",
  "meldung.aktiviert": "Profile {n} activated on device.",
  "meldung.aktivierenFehler": "Profile switch failed: {fehler}",
  "meldung.geoeffnet": "Opened: {pfad} – apply with “Save & apply”.",
  "meldung.gespeichert": "Saved: {pfad}",
  "meldung.konfigFehlerhaft": "Saved configuration is invalid, example loaded: {fehler}",

  "dialog.werkTitel": "Factory defaults",
  "dialog.werkText":
    "Reset all three profiles to the factory assignments?\n\nProfile 1: digits 1–8 · Profile 2: media · Profile 3: F13–F20\n\nThe device only changes after “Save & apply”.",
  "dialog.lesenTitel": "Read from device",
  "dialog.lesenText": "The assignments in this window will be replaced by the data from the device. Continue?",
  "datei.filter": "Configuration",
};

/** Englische Anzeigenamen der Tasten (deutsche stehen in main.ts) */
export const ANZEIGE_EN: Record<string, string> = {
  eingabe: "Enter", esc: "Esc", ruecktaste: "Backspace", tab: "Tab", leertaste: "Space",
  feststell: "CapsLock", druck: "PrtSc", rollen: "ScrLk", pause: "Pause", einfg: "Ins",
  pos1: "Home", bild_auf: "PgUp", entf: "Del", ende: "End", bild_ab: "PgDn",
  pfeil_rechts: "→", pfeil_links: "←", pfeil_runter: "↓", pfeil_hoch: "↑",
  num: "NumLock", menue: "Menu", strg: "Ctrl", umschalt: "Shift", alt: "Alt", win: "Win",
  strg_rechts: "Ctrl-R", umschalt_rechts: "Shift-R", altgr: "AltGr", win_rechts: "Win-R", plus: "+",
};

/** Englische Texte zum Funktionskatalog des Backends (deutsche liefert das Backend selbst) */
export const KATALOG_EN: Record<"medien" | "einfach", Record<string, string>> = {
  einfach: {
    aus: "Key without function",
    linksklick: "Left mouse button",
    mittelklick: "Middle mouse button",
    rechtsklick: "Right mouse button",
    zurueck: "Mouse button 4 (browser back)",
    vor: "Mouse button 5 (browser forward)",
    desktop_anzeigen: "Show desktop",
    doppelklick: "Double click",
    dauerfeuer: "Left-click rapid fire",
    led_zyklus: "Cycle lighting mode",
    profil_1: "Switch to profile 1",
    profil_2: "Switch to profile 2",
    profil_3: "Switch to profile 3",
  },
  medien: {
    browser: "Browser home page",
    player: "Launch media player",
    mail: "E-mail client",
    lauter: "Volume +",
    leiser: "Volume −",
    zurueck: "Previous track",
    weiter: "Next track",
    stumm: "Mute",
    abspielen: "Play/Pause",
    rechner: "Calculator",
    stopp: "Stop playback",
  },
};

const SPEICHER_SCHLUESSEL = "sprache";

function anfangsSprache(): Sprache {
  try {
    const s = localStorage.getItem(SPEICHER_SCHLUESSEL);
    if (s === "de" || s === "en") return s;
  } catch {
    /* kein Speicher – Systemsprache nehmen */
  }
  return navigator.language.toLowerCase().startsWith("de") ? "de" : "en";
}

export let sprache: Sprache = anfangsSprache();

export function spracheSetzen(s: Sprache) {
  sprache = s;
  try {
    localStorage.setItem(SPEICHER_SCHLUESSEL, s);
  } catch {
    /* nur für diese Sitzung */
  }
}

export function t(schluessel: Schluessel, werte: Record<string, string | number> = {}): string {
  const text = (sprache === "en" ? EN : DE)[schluessel];
  return text.replace(/\{(\w+)\}/g, (_, n) => String(werte[n] ?? `{${n}}`));
}

/** Überträgt alle data-i18n*-Attribute im Dokument in die aktuelle Sprache. */
export function seiteUebersetzen() {
  document.documentElement.lang = sprache;
  document.title = t("app.titel");
  const attribute: [string, (el: HTMLElement, text: string) => void][] = [
    ["data-i18n", (el, text) => (el.textContent = text)],
    ["data-i18n-title", (el, text) => (el.title = text)],
    ["data-i18n-aria-label", (el, text) => el.setAttribute("aria-label", text)],
    ["data-i18n-placeholder", (el, text) => el.setAttribute("placeholder", text)],
  ];
  for (const [attr, setzen] of attribute) {
    document.querySelectorAll<HTMLElement>(`[${attr}]`).forEach((el) => setzen(el, t(el.getAttribute(attr) as Schluessel)));
  }
}
