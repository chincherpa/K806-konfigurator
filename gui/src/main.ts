import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { ask, open, save } from "@tauri-apps/plugin-dialog";
import { ANZEIGE_EN, KATALOG_EN, seiteUebersetzen, sprache, spracheSetzen, t, type Sprache } from "./i18n";

// ---------------------------------------------------------------------------
// Typen (spiegeln die Rust-Strukturen in src/konfig.rs)
// ---------------------------------------------------------------------------

type LedModus = "zyklus_atmen" | "dauerlicht" | "tastenreaktion" | "aus" | "einzel_atmen";

interface Taste {
  name: string;
  steckplatz: number;
}

interface Profil {
  name: string;
  belegung: Record<string, string>;
  led_reaktion: Record<string, { aktiv: boolean; typ: number }>;
  bezeichnung?: Record<string, string>;
}

interface Konfiguration {
  geraet: { vendor_id: number; product_id: number; bericht_laenge: number };
  allgemein: { aktives_profil: number; abfragerate_hz: number; dauerfeuer_ms: number; tippverzoegerung: number };
  beleuchtung: { modus: LedModus; helligkeit: number; geschwindigkeit: number; [x: string]: unknown };
  taste: Taste[];
  profil: Profil[];
  erweitert: unknown;
}

interface Eintrag {
  name: string;
  code: number;
  text: string;
}

interface Katalog {
  tasten: Eintrag[];
  medien: Eintrag[];
  einfach: Eintrag[];
}

interface GeraeteStatus {
  profil: number | null;
}

type Art = "kombi" | "taste" | "text" | "medien" | "sonstige" | "keine";

// ---------------------------------------------------------------------------
// Zustand
// ---------------------------------------------------------------------------

let k: Konfiguration;
let gespeichert = ""; // Stand nach Laden / letztem Speichern (für „Abbrechen“)
let katalog: Katalog;
let profilIndex = 0;
let gewaehlt = 0; // Index in k.taste
let editorArt: Art = "kombi";
let beschaeftigt = false;
let liveProfil: number | null = null;
let verbunden: boolean | null = null; // null = noch unbekannt

const $ = <T extends HTMLElement = HTMLElement>(id: string) => document.getElementById(id) as T;
const profil = () => k.profil[profilIndex];
const tasteName = () => k.taste[gewaehlt]?.name;

// ---------------------------------------------------------------------------
// Symbole
// ---------------------------------------------------------------------------

const SYMBOL = {
  stumm: '<svg viewBox="0 0 24 24"><rect x="9" y="3" width="6" height="11" rx="3"/><path d="M5.5 11a6.5 6.5 0 0 0 13 0M12 17.5V21"/></svg>',
  text: '<svg viewBox="0 0 24 24"><path d="M5 6V4.5h14V6M12 4.5v15M9 19.5h6"/></svg>',
  globus: '<svg viewBox="0 0 24 24"><circle cx="12" cy="12" r="9"/><path d="M3 12h18M12 3a14 14 0 0 1 0 18M12 3a14 14 0 0 0 0 18"/></svg>',
  medien: '<svg viewBox="0 0 24 24"><path d="M9 18V5l11-2v13"/><circle cx="6.5" cy="18" r="2.5"/><circle cx="17.5" cy="16" r="2.5"/></svg>',
  laut: '<svg viewBox="0 0 24 24"><path d="M4 9.5h3.5L12 5.5v13l-4.5-4H4z"/><path d="M15.5 9a4 4 0 0 1 0 6M18 6.5a7.5 7.5 0 0 1 0 11"/></svg>',
  zahnrad:
    '<svg viewBox="0 0 24 24"><circle cx="12" cy="12" r="3"/><path d="M19.4 15a1.7 1.7 0 0 0 .3 1.8l.1.1a2 2 0 1 1-2.8 2.8l-.1-.1a1.7 1.7 0 0 0-1.8-.3 1.7 1.7 0 0 0-1 1.5V21a2 2 0 1 1-4 0v-.1a1.7 1.7 0 0 0-1.1-1.5 1.7 1.7 0 0 0-1.8.3l-.1.1a2 2 0 1 1-2.8-2.8l.1-.1a1.7 1.7 0 0 0 .3-1.8 1.7 1.7 0 0 0-1.5-1H3a2 2 0 1 1 0-4h.1a1.7 1.7 0 0 0 1.5-1.1 1.7 1.7 0 0 0-.3-1.8l-.1-.1a2 2 0 1 1 2.8-2.8l.1.1a1.7 1.7 0 0 0 1.8.3H9a1.7 1.7 0 0 0 1-1.5V3a2 2 0 1 1 4 0v.1a1.7 1.7 0 0 0 1 1.5 1.7 1.7 0 0 0 1.8-.3l.1-.1a2 2 0 1 1 2.8 2.8l-.1.1a1.7 1.7 0 0 0-.3 1.8V9a1.7 1.7 0 0 0 1.5 1H21a2 2 0 1 1 0 4h-.1a1.7 1.7 0 0 0-1.5 1z"/></svg>',
};

function symbolFuer(f: string | undefined): string {
  if (!f) return "";
  if (f === "medien:stumm") return SYMBOL.stumm;
  if (f === "medien:lauter" || f === "medien:leiser") return SYMBOL.laut;
  if (f === "medien:browser") return SYMBOL.globus;
  if (f.startsWith("medien:")) return SYMBOL.medien;
  if (f.startsWith("text:")) return SYMBOL.text;
  if (!f.startsWith("taste:") && !f.startsWith("kombi:") && f !== "aus") return SYMBOL.zahnrad;
  return "";
}

// ---------------------------------------------------------------------------
// Tastennamen
// ---------------------------------------------------------------------------

/** KeyboardEvent.code (physische Position) -> HID-Usage */
const CODE_ZU_HID: Record<string, number> = (() => {
  const m: Record<string, number> = {};
  "ABCDEFGHIJKLMNOPQRSTUVWXYZ".split("").forEach((b, i) => (m[`Key${b}`] = 0x04 + i));
  "1234567890".split("").forEach((z, i) => (m[`Digit${z}`] = 0x1e + i));
  for (let i = 1; i <= 12; i++) m[`F${i}`] = 0x39 + i;
  for (let i = 13; i <= 24; i++) m[`F${i}`] = 0x68 + i - 13;
  for (let i = 1; i <= 9; i++) m[`Numpad${i}`] = 0x58 + i;
  Object.assign(m, {
    Enter: 0x28, Escape: 0x29, Backspace: 0x2a, Tab: 0x2b, Space: 0x2c,
    Minus: 0x2d, Equal: 0x2e, BracketLeft: 0x2f, BracketRight: 0x30, Backslash: 0x32,
    Semicolon: 0x33, Quote: 0x34, Backquote: 0x35, Comma: 0x36, Period: 0x37, Slash: 0x38,
    CapsLock: 0x39, PrintScreen: 0x46, ScrollLock: 0x47, Pause: 0x48, Insert: 0x49,
    Home: 0x4a, PageUp: 0x4b, Delete: 0x4c, End: 0x4d, PageDown: 0x4e,
    ArrowRight: 0x4f, ArrowLeft: 0x50, ArrowDown: 0x51, ArrowUp: 0x52,
    NumLock: 0x53, NumpadDivide: 0x54, NumpadMultiply: 0x55, NumpadSubtract: 0x56,
    NumpadAdd: 0x57, NumpadEnter: 0x58, Numpad0: 0x62, NumpadDecimal: 0x63,
    IntlBackslash: 0x64, ContextMenu: 0x65,
    ControlLeft: 0xe0, ShiftLeft: 0xe1, AltLeft: 0xe2, MetaLeft: 0xe3,
    ControlRight: 0xe4, ShiftRight: 0xe5, AltRight: 0xe6, MetaRight: 0xe7,
  });
  return m;
})();

const ANZEIGE_DE: Record<string, string> = {
  eingabe: "Enter", esc: "Esc", ruecktaste: "Rücktaste", tab: "Tab", leertaste: "Leertaste",
  feststell: "Feststell", druck: "Druck", rollen: "Rollen", pause: "Pause", einfg: "Einfg",
  pos1: "Pos1", bild_auf: "Bild↑", entf: "Entf", ende: "Ende", bild_ab: "Bild↓",
  pfeil_rechts: "→", pfeil_links: "←", pfeil_runter: "↓", pfeil_hoch: "↑",
  num: "Num", menue: "Menü", strg: "Strg", umschalt: "Umschalt", alt: "Alt", win: "Win",
  strg_rechts: "Strg-R", umschalt_rechts: "Umschalt-R", altgr: "AltGr", win_rechts: "Win-R", plus: "+",
};

const anzeigeTabelle = () => (sprache === "en" ? ANZEIGE_EN : ANZEIGE_DE);

function anzeige(name: string): string {
  const a = anzeigeTabelle()[name];
  if (a) return a;
  if (name.startsWith("num_")) return "Num" + name.slice(4);
  if (/^f\d+$/.test(name)) return name.toUpperCase();
  return name.length === 1 ? name.toUpperCase() : name;
}

/** Anzeigetext -> interner Name (für das Eingabefeld) */
function ausAnzeige(text: string, inKombi: boolean): string {
  const t = text.trim().toLowerCase();
  // Beide Sprachen akzeptieren, die aktuelle zuerst
  for (const tabelle of [anzeigeTabelle(), ANZEIGE_DE, ANZEIGE_EN])
    for (const [name, a] of Object.entries(tabelle)) if (a.toLowerCase() === t) return inKombi && name === "+" ? "plus" : name;
  if (/^num\d$/.test(t)) return "num_" + t.slice(3);
  if (inKombi && t === "+") return "plus";
  return t;
}

function tastenname(usage: number, inKombi: boolean): string | undefined {
  const e = katalog.tasten.find((t) => t.code === usage);
  if (!e) return undefined;
  return inKombi && e.name === "+" ? "plus" : e.name;
}

function istModifier(usage: number) {
  return usage >= 0xe0 && usage <= 0xe7;
}

function teile(f: string): [string, string] {
  const i = f.indexOf(":");
  return i < 0 ? ["", f] : [f.slice(0, i), f.slice(i + 1)];
}

function artVon(f: string | undefined): Art {
  if (!f || f === "aus") return "keine";
  const [p] = teile(f);
  if (p === "taste" || p === "kombi" || p === "medien" || p === "text") return p;
  return "sonstige";
}

/** Kurzbeschreibung einer Funktion */
function beschreibung(f: string | undefined): string {
  if (!f || f === "aus") return t("funktion.keine");
  const [p, rest] = teile(f);
  if (p === "taste") return t("funktion.taste", { name: anzeige(rest) });
  if (p === "kombi") return rest.split("+").map(anzeige).join("+");
  if (p === "text") return t("funktion.text", { text: rest.replace(/\n/g, "⏎") });
  if (p === "medien") {
    const e = katalog.medien.find((m) => m.name === rest);
    return e ? katalogText("medien", e) : rest;
  }
  const e = katalog.einfach.find((e) => e.name === f);
  return e ? katalogText("einfach", e) : f;
}

/** Katalogtext in der aktuellen Sprache (Deutsch liefert das Backend) */
function katalogText(liste: "medien" | "einfach", e: Eintrag): string {
  return (sprache === "en" && KATALOG_EN[liste][e.name]) || e.text;
}

const artTitel = (art: Art) => t(`titel.${art}`);

// ---------------------------------------------------------------------------
// Status / Meldungen
// ---------------------------------------------------------------------------

function melden(text: string, art: "info" | "fehler" | "erfolg" = "info") {
  $("meldung").textContent = text;
  document.querySelector<HTMLElement>(".status")!.dataset.art = art;
}

function fortschritt(prozent: number | null) {
  $("fortschritt").hidden = prozent === null;
  $("fortschritt-balken").style.width = `${prozent ?? 0}%`;
}

function sperren(an: boolean) {
  beschaeftigt = an;
  document.querySelectorAll<HTMLButtonElement>(".fuss .knopf, .menue-liste button[data-aktion]").forEach((b) => (b.disabled = an));
}

const schnappschuss = () => JSON.stringify(k);

/** `mitMeldung = false` nach Geräteaktionen: deren Erfolgs-/Fehlermeldung bleibt stehen. */
function geaendert(mitMeldung = true) {
  const schmutzig = schnappschuss() !== gespeichert;
  $<HTMLButtonElement>("knopf-abbrechen").disabled = !schmutzig || beschaeftigt;
  if (!mitMeldung) return;
  if (schmutzig) melden(t("meldung.ungespeichert"));
  else if ($("meldung").textContent === t("meldung.ungespeichert")) melden("");
}

// ---------------------------------------------------------------------------
// Bühne: Tastenfeld, Beschriftungen, Linien
// ---------------------------------------------------------------------------

const BUEHNE = { b: 820, h: 470 };
const KOERPER = { x: 214, y: 128, b: 422, h: 226, rand: 13 };
const FELD = { innen: 7, abstand: 6 };

/** Mittelpunkt und Größe der Kappe i (0–7), in Bühnenkoordinaten */
function kappenGeometrie(i: number) {
  const x0 = KOERPER.x + KOERPER.rand + FELD.innen;
  const y0 = KOERPER.y + KOERPER.rand + FELD.innen;
  const breite = (KOERPER.b - 2 * KOERPER.rand - 2 * FELD.innen - 3 * FELD.abstand) / 4;
  const hoehe = (KOERPER.h - 2 * KOERPER.rand - 2 * FELD.innen - FELD.abstand) / 2;
  const s = i % 4;
  const z = Math.floor(i / 4);
  const x = x0 + s * (breite + FELD.abstand);
  const y = y0 + z * (hoehe + FELD.abstand);
  return { x, y, breite, hoehe, mx: x + breite / 2, my: y + hoehe / 2 };
}

/** Seitlicher Versatz der oberen/unteren Beschriftungen, damit sie sich nicht überlappen */
const VERSATZ = [0, -66, 0, 66];

/** Wo die Beschriftung einer Taste sitzt: 1 und 5 links, 2–4 oben, 6–8 unten */
function seiteVon(i: number): "links" | "oben" | "unten" {
  if (i === 0 || i === 4) return "links";
  return i < 4 ? "oben" : "unten";
}

function zeichneTastenfeld() {
  const feld = $("tastenfeld");
  feld.replaceChildren();
  const beschr = $("beschriftungen");
  beschr.replaceChildren();
  const svg = $("linien");
  svg.replaceChildren();
  const p = profil();

  k.taste.slice(0, 8).forEach((taste, i) => {
    const f = p.belegung[taste.name];
    const art = artVon(f);
    const leer = art === "keine";
    const waehlen = () => {
      gewaehlt = i;
      editorArt = art === "keine" ? "kombi" : art;
      zeichneTastenfeld();
      zeichneEditor();
    };

    // Kappe
    const kappe = document.createElement("button");
    kappe.className = "kappe " + (leer ? "leer" : "belegt") + (symbolFuer(f) ? " mit-symbol" : "");
    kappe.setAttribute("aria-pressed", String(i === gewaehlt));
    kappe.setAttribute("aria-label", t("kappe.aria", { n: i + 1, text: beschreibung(f) }));
    const sym = symbolFuer(f);
    if (sym) {
      const s = document.createElement("span");
      s.className = "symbol";
      s.innerHTML = sym; // feste, eigene SVG-Zeichenkette
      kappe.append(s);
    }
    const ziffer = document.createElement("span");
    ziffer.className = "ziffer";
    ziffer.textContent = String(i + 1);
    kappe.append(ziffer);
    kappe.addEventListener("click", waehlen);
    feld.append(kappe);

    // Beschriftung
    const g = kappenGeometrie(i);
    const seite = seiteVon(i);
    const name = p.bezeichnung?.[taste.name];
    const text = leer ? t("funktion.keine") : name ? `${name} (${beschreibung(f)})` : beschreibung(f);

    const el = document.createElement("div");
    el.className = `beschriftung ${seite}` + (leer ? " leer" : "") + (i === gewaehlt ? " aktiv" : "");
    el.tabIndex = 0;
    el.title = text;
    const titel = document.createElement("span");
    titel.className = "titel";
    titel.textContent = artTitel(art);
    const kasten = document.createElement("span");
    kasten.className = "kasten";
    if (sym) {
      const s = document.createElement("span");
      s.innerHTML = sym;
      s.style.display = "contents";
      kasten.append(s);
    }
    kasten.append(document.createTextNode(text));
    if (seite === "unten") el.append(kasten, titel);
    else el.append(titel, kasten);
    el.addEventListener("click", waehlen);
    el.addEventListener("keydown", (e) => (e.key === "Enter" || e.key === " ") && waehlen());
    beschr.append(el);

    // Position: Mittelpunkt des Kastens liegt auf der Linie
    const KASTEN_MITTE = 16 + 5 + 16; // Titelhöhe + Abstand + halbe Kastenhöhe
    const bx = g.mx + VERSATZ[i % 4];
    let punkte: [number, number][];
    if (seite === "links") {
      el.style.right = `${BUEHNE.b - (KOERPER.x - 26)}px`;
      el.style.top = `${g.my - KASTEN_MITTE}px`;
      punkte = [[KOERPER.x - 26, g.my], [g.x, g.my]];
    } else if (seite === "oben") {
      const yKasten = KOERPER.y - 38;
      el.style.left = `${bx}px`;
      el.style.bottom = `${BUEHNE.h - yKasten}px`;
      el.style.transform = "translateX(-50%)";
      const knick = KOERPER.y - 19;
      punkte = [[bx, yKasten], [bx, knick], [g.mx, knick], [g.mx, g.y]];
    } else {
      const yKasten = KOERPER.y + KOERPER.h + 38;
      el.style.left = `${bx}px`;
      el.style.top = `${yKasten}px`;
      el.style.transform = "translateX(-50%)";
      const knick = KOERPER.y + KOERPER.h + 19;
      punkte = [[bx, yKasten], [bx, knick], [g.mx, knick], [g.mx, g.y + g.hoehe]];
    }
    const linie = document.createElementNS("http://www.w3.org/2000/svg", "polyline");
    linie.setAttribute("points", punkte.map((p) => p.join(",")).join(" "));
    svg.append(linie);
  });
}

function skalieren() {
  const rahmen = document.querySelector<HTMLElement>(".buehne-rahmen")!.getBoundingClientRect();
  const s = Math.min(rahmen.width / BUEHNE.b, rahmen.height / BUEHNE.h, 1.3);
  $("buehne").style.transform = `translate(-50%, -50%) scale(${s})`;
}

// ---------------------------------------------------------------------------
// Profile
// ---------------------------------------------------------------------------

function profilAuffuellen() {
  while (k.profil.length < 3) {
    k.profil.push({ name: `Profil ${k.profil.length + 1}`, belegung: {}, led_reaktion: {} });
  }
}

function zeichneProfile() {
  const leiste = $("profilwahl");
  leiste.replaceChildren();
  k.profil.forEach((p, i) => {
    const b = document.createElement("button");
    b.setAttribute("role", "tab");
    b.setAttribute("aria-selected", String(i === profilIndex));
    b.title = p.name + (liveProfil === i + 1 ? t("profil.live") : "");
    b.textContent = t("profil.tab", { n: i + 1 });
    if (liveProfil === i + 1) {
      const punkt = document.createElement("span");
      punkt.className = "live";
      b.append(punkt);
    }
    b.addEventListener("click", () => {
      profilIndex = i;
      zeichneAlles();
    });
    leiste.append(b);
  });
}

// ---------------------------------------------------------------------------
// Editor
// ---------------------------------------------------------------------------

function fuelleAuswahl(sel: HTMLSelectElement, eintraege: { wert: string; text: string }[]) {
  sel.replaceChildren();
  for (const e of eintraege) {
    const o = document.createElement("option");
    o.value = e.wert;
    o.textContent = e.text;
    sel.append(o);
  }
}

function zeichneEditor() {
  const name = tasteName();
  const f = name ? profil().belegung[name] : undefined;
  $("editor-titel").textContent = t("editor.titel", { n: gewaehlt + 1 });
  $<HTMLSelectElement>("aktionstyp").value = editorArt;

  document.querySelectorAll<HTMLElement>("#editor .feld[data-fuer]").forEach((el) => {
    el.hidden = !el.dataset.fuer!.split(" ").includes(editorArt);
  });

  const [p, rest] = f ? teile(f) : ["", ""];
  $("aufnahme-titel").textContent = t(editorArt === "kombi" ? "editor.kombi" : "editor.taste");
  const eingabe = $<HTMLInputElement>("eingabe-tasten");
  eingabe.placeholder = editorArt === "kombi" ? `${anzeige("strg")} + C` : "A";
  eingabe.value =
    p === "kombi" && editorArt === "kombi"
      ? rest.split("+").map(anzeige).join(" + ")
      : p === "taste" && editorArt === "taste"
        ? anzeige(rest)
        : "";
  $<HTMLSelectElement>("wahl-taste").value = p === "taste" ? rest : "";
  $<HTMLSelectElement>("wahl-medien").value = p === "medien" ? rest : "";
  const textfeld = $<HTMLTextAreaElement>("eingabe-text");
  if (document.activeElement !== textfeld) textfeld.value = p === "text" ? rest : "";
  $<HTMLSelectElement>("wahl-sonstige").value = artVon(f) === "sonstige" ? f! : "";
  $<HTMLInputElement>("eingabe-name").value = (name && profil().bezeichnung?.[name]) || "";
  $("knopf-aufnahme").textContent = t(hoert ? "aufnahme.stopp" : "aufnahme.start");
  if (!hoert) {
    $("aufnahme-hinweis").textContent = t(editorArt === "kombi" ? "aufnahme.hinweisKombi" : "aufnahme.hinweisTaste");
  }
}

function zeichneEinstellungen() {
  const b = k.beleuchtung;
  $<HTMLSelectElement>("led-modus").value = b.modus;
  $<HTMLInputElement>("led-hell").value = String(b.helligkeit);
  $("led-hell-wert").textContent = String(b.helligkeit);
  $<HTMLInputElement>("led-tempo").value = String(b.geschwindigkeit);
  $("led-tempo-wert").textContent = String(b.geschwindigkeit);
  $<HTMLSelectElement>("abfragerate").value = String(k.allgemein.abfragerate_hz);
  $<HTMLInputElement>("tippverzoegerung").value = String(k.allgemein.tippverzoegerung);
  $("tippverzoegerung-wert").textContent = String(k.allgemein.tippverzoegerung);
}

function zeichneAlles() {
  zeichneProfile();
  zeichneTastenfeld();
  zeichneEditor();
  zeichneEinstellungen();
  geaendert();
}

async function setzeFunktion(text: string) {
  const name = tasteName();
  if (!name) return;
  $("editor-fehler").textContent = "";
  try {
    const normal = await invoke<string>("funktion_pruefen", { text });
    const belegung = profil().belegung;
    if (normal === "aus") delete belegung[name];
    else belegung[name] = normal;
    const art = artVon(normal);
    if (art !== "keine") editorArt = art;
    zeichneTastenfeld();
    zeichneEditor();
    geaendert();
  } catch (e) {
    $("editor-fehler").textContent = String(e);
  }
}

// Aufnahme per Tastatur
let hoert = false;
const gedrueckt = new Set<number>();
let nurModifier = true;

function aufnahmeStarten() {
  hoert = true;
  gedrueckt.clear();
  nurModifier = true;
  const b = $("knopf-aufnahme");
  b.classList.add("hoert");
  b.textContent = t("aufnahme.stopp");
  $("aufnahme-hinweis").textContent = t("aufnahme.hoert");
}

function aufnahmeBeenden() {
  hoert = false;
  gedrueckt.clear();
  const b = $("knopf-aufnahme");
  b.classList.remove("hoert");
  b.textContent = t("aufnahme.start");
  zeichneEditor();
}

window.addEventListener(
  "keydown",
  (e) => {
    if (!hoert) return;
    e.preventDefault();
    e.stopPropagation();
    const usage = CODE_ZU_HID[e.code];
    if (usage === undefined) {
      $("aufnahme-hinweis").textContent = t("aufnahme.nichtUnterstuetzt", { code: e.code });
      return;
    }
    gedrueckt.add(usage);
    if (istModifier(usage)) {
      $("aufnahme-hinweis").textContent = [...gedrueckt].map((u) => anzeige(tastenname(u, true) ?? "?")).join(" + ") + " + …";
      return;
    }
    nurModifier = false;
    const mods = [...gedrueckt].filter(istModifier).slice(0, 3);
    if (mods.length === 0 && editorArt !== "kombi") {
      setzeFunktion(`taste:${tastenname(usage, false)}`);
    } else if (mods.length === 0) {
      $("aufnahme-hinweis").textContent = t("aufnahme.ohneModifier");
      gedrueckt.delete(usage);
      return;
    } else {
      setzeFunktion(`kombi:${[...mods, usage].map((u) => tastenname(u, true)).join("+")}`);
    }
    aufnahmeBeenden();
  },
  true,
);

window.addEventListener(
  "keyup",
  (e) => {
    if (!hoert) return;
    e.preventDefault();
    const usage = CODE_ZU_HID[e.code];
    if (usage === undefined) return;
    // Nur ein Modifier gedrückt und wieder losgelassen -> als einzelne Taste
    if (istModifier(usage) && nurModifier && gedrueckt.size === 1) {
      setzeFunktion(`taste:${tastenname(usage, false)}`);
      aufnahmeBeenden();
    } else {
      gedrueckt.delete(usage);
    }
  },
  true,
);

// ---------------------------------------------------------------------------
// Gerät
// ---------------------------------------------------------------------------

async function statusAbfragen(still = false) {
  if (beschaeftigt) return;
  const vorher = liveProfil;
  try {
    const s = await invoke<GeraeteStatus>("status");
    verbunden = true;
    liveProfil = s.profil;
  } catch (e) {
    verbunden = false;
    liveProfil = null;
    if (!still) melden(String(e), "fehler");
  }
  zeichneGeraet();
  if (vorher !== liveProfil) zeichneProfile();
}

function zeichneGeraet() {
  $("geraet").dataset.zustand = verbunden === null ? "unbekannt" : verbunden ? "verbunden" : "getrennt";
  $("geraet-text").textContent = t(verbunden === null ? "kopf.suche" : verbunden ? "kopf.verbunden" : "kopf.getrennt");
}

async function speichernUndAnwenden() {
  k.allgemein.aktives_profil = profilIndex + 1;
  sperren(true);
  melden(t("meldung.speichere"));
  try {
    await invoke("konfig_speichern", { konfiguration: k });
    gespeichert = schnappschuss();
    await invoke("anwenden", { konfiguration: k });
    melden(t("meldung.gespeichertAktiv", { n: profilIndex + 1 }), "erfolg");
  } catch (e) {
    melden(t("meldung.fehler", { fehler: String(e) }), "fehler");
  } finally {
    sperren(false);
    geaendert(false);
    statusAbfragen(true);
  }
}

function abbrechen() {
  k = JSON.parse(gespeichert);
  profilAuffuellen();
  zeichneAlles();
  melden(t("meldung.verworfen"));
}

async function werkseinstellungen() {
  const ok = await ask(t("dialog.werkText"), { title: t("dialog.werkTitel"), kind: "warning" });
  if (!ok) return;
  k = await invoke<Konfiguration>("beispiel");
  profilAuffuellen();
  zeichneAlles();
}

async function auslesen() {
  const ok = await ask(t("dialog.lesenText"), { title: t("dialog.lesenTitel"), kind: "warning" });
  if (!ok) return;
  sperren(true);
  fortschritt(0);
  melden(t("meldung.lese"));
  try {
    const neu = await invoke<Konfiguration>("auslesen", { tasten: k.taste });
    // Namen gibt es nur in der Datei
    neu.profil.forEach((p, i) => {
      p.name = k.profil[i]?.name ?? p.name;
      p.bezeichnung = k.profil[i]?.bezeichnung ?? {};
    });
    k = neu;
    profilAuffuellen();
    zeichneAlles();
    melden(t("meldung.gelesen"), "erfolg");
  } catch (e) {
    melden(t("meldung.lesenFehler", { fehler: String(e) }), "fehler");
  } finally {
    fortschritt(null);
    sperren(false);
    geaendert(false);
  }
}

async function profilAktivieren() {
  sperren(true);
  try {
    await invoke("profil_aktivieren", { nummer: profilIndex + 1 });
    melden(t("meldung.aktiviert", { n: profilIndex + 1 }), "erfolg");
  } catch (e) {
    melden(t("meldung.aktivierenFehler", { fehler: String(e) }), "fehler");
  } finally {
    sperren(false);
    geaendert(false);
    statusAbfragen(true);
  }
}

const filter = () => [{ name: t("datei.filter"), extensions: ["toml"] }];

async function dateiOeffnen() {
  const pfad = await open({ filters: filter(), multiple: false, directory: false });
  if (typeof pfad !== "string") return;
  try {
    k = await invoke<Konfiguration>("datei_oeffnen", { pfad });
    profilAuffuellen();
    gewaehlt = 0;
    zeichneAlles();
    melden(t("meldung.geoeffnet", { pfad }));
  } catch (e) {
    melden(String(e), "fehler");
  }
}

async function dateiSpeichern() {
  const pfad = await save({ filters: filter(), defaultPath: "konfiguration.toml" });
  if (!pfad) return;
  try {
    await invoke("datei_speichern", { pfad, konfiguration: k });
    melden(t("meldung.gespeichert", { pfad }), "erfolg");
  } catch (e) {
    melden(String(e), "fehler");
  }
}

// ---------------------------------------------------------------------------
// Verdrahtung
// ---------------------------------------------------------------------------

function menueSchalten(an?: boolean) {
  const liste = $("menue-liste");
  const offen = an ?? liste.hidden;
  liste.hidden = !offen;
  $("knopf-menue").setAttribute("aria-expanded", String(offen));
}

function verdrahten() {
  $("knopf-anwenden").addEventListener("click", speichernUndAnwenden);
  $("knopf-abbrechen").addEventListener("click", abbrechen);
  $("knopf-werk").addEventListener("click", werkseinstellungen);

  $("knopf-menue").addEventListener("click", (e) => {
    e.stopPropagation();
    menueSchalten();
  });
  document.addEventListener("click", () => menueSchalten(false));
  document.addEventListener("keydown", (e) => !hoert && e.key === "Escape" && menueSchalten(false));
  const aktionen: Record<string, () => void> = {
    auslesen,
    aktivieren: profilAktivieren,
    oeffnen: dateiOeffnen,
    "speichern-unter": dateiSpeichern,
  };
  document.querySelectorAll<HTMLButtonElement>(".menue-liste button[data-aktion]").forEach((b) =>
    b.addEventListener("click", () => {
      menueSchalten(false);
      aktionen[b.dataset.aktion!]();
    }),
  );
  document.querySelectorAll<HTMLButtonElement>("#sprachwahl button").forEach((b) =>
    b.addEventListener("click", (e) => {
      e.stopPropagation(); // Menü offen lassen
      spracheWechseln(b.dataset.sprache as Sprache);
    }),
  );

  $<HTMLSelectElement>("aktionstyp").addEventListener("change", (e) => {
    editorArt = (e.target as HTMLSelectElement).value as Art;
    if (editorArt === "keine") setzeFunktion("aus");
    else zeichneEditor();
  });
  $("knopf-aufnahme").addEventListener("click", () => (hoert ? aufnahmeBeenden() : aufnahmeStarten()));
  $<HTMLInputElement>("eingabe-tasten").addEventListener("change", (e) => {
    const roh = (e.target as HTMLInputElement).value.trim();
    if (!roh) return;
    const namen = roh.split(/\s*\+\s*(?=\S)/).map((t) => ausAnzeige(t, editorArt === "kombi"));
    setzeFunktion(namen.length === 1 && editorArt === "taste" ? `taste:${namen[0]}` : `kombi:${namen.join("+")}`);
  });
  $<HTMLTextAreaElement>("eingabe-text").addEventListener("change", (e) => {
    const v = (e.target as HTMLTextAreaElement).value;
    setzeFunktion(v ? `text:${v}` : "aus");
  });
  $<HTMLSelectElement>("wahl-taste").addEventListener("change", (e) => {
    const v = (e.target as HTMLSelectElement).value;
    if (v) setzeFunktion(`taste:${v}`);
  });
  $<HTMLSelectElement>("wahl-medien").addEventListener("change", (e) => {
    const v = (e.target as HTMLSelectElement).value;
    if (v) setzeFunktion(`medien:${v}`);
  });
  $<HTMLSelectElement>("wahl-sonstige").addEventListener("change", (e) => {
    const v = (e.target as HTMLSelectElement).value;
    if (v) setzeFunktion(v);
  });
  $<HTMLInputElement>("eingabe-name").addEventListener("input", (e) => {
    const name = tasteName();
    if (!name) return;
    const p = profil();
    p.bezeichnung ??= {};
    const v = (e.target as HTMLInputElement).value.trim();
    if (v) p.bezeichnung[name] = v;
    else delete p.bezeichnung[name];
    zeichneTastenfeld();
    geaendert();
  });

  $<HTMLSelectElement>("led-modus").addEventListener("change", (e) => {
    k.beleuchtung.modus = (e.target as HTMLSelectElement).value as LedModus;
    geaendert();
  });
  $<HTMLInputElement>("led-hell").addEventListener("input", (e) => {
    k.beleuchtung.helligkeit = Number((e.target as HTMLInputElement).value);
    $("led-hell-wert").textContent = String(k.beleuchtung.helligkeit);
    geaendert();
  });
  $<HTMLInputElement>("led-tempo").addEventListener("input", (e) => {
    k.beleuchtung.geschwindigkeit = Number((e.target as HTMLInputElement).value);
    $("led-tempo-wert").textContent = String(k.beleuchtung.geschwindigkeit);
    geaendert();
  });
  $<HTMLSelectElement>("abfragerate").addEventListener("change", (e) => {
    k.allgemein.abfragerate_hz = Number((e.target as HTMLSelectElement).value);
    geaendert();
  });
  $<HTMLInputElement>("tippverzoegerung").addEventListener("input", (e) => {
    k.allgemein.tippverzoegerung = Number((e.target as HTMLInputElement).value);
    $("tippverzoegerung-wert").textContent = String(k.allgemein.tippverzoegerung);
    geaendert();
  });

  new ResizeObserver(skalieren).observe(document.querySelector(".buehne-rahmen")!);
}

// ---------------------------------------------------------------------------
// Sprache
// ---------------------------------------------------------------------------

function fuelleListen() {
  const leer = { wert: "", text: t("editor.waehlen") };
  fuelleAuswahl($("wahl-taste"), [
    leer,
    ...katalog.tasten.filter((e) => e.name !== "plus").map((e) => ({ wert: e.name, text: anzeige(e.name) })),
  ]);
  fuelleAuswahl($("wahl-medien"), [leer, ...katalog.medien.map((m) => ({ wert: m.name, text: katalogText("medien", m) }))]);
  fuelleAuswahl($("wahl-sonstige"), [
    leer,
    ...katalog.einfach.filter((e) => e.name !== "aus").map((e) => ({ wert: e.name, text: katalogText("einfach", e) })),
  ]);
}

/** Statische Texte, Auswahllisten und Sprachumschalter in der aktuellen Sprache */
function spracheAnwenden() {
  seiteUebersetzen();
  document.querySelectorAll<HTMLButtonElement>("#sprachwahl button").forEach((b) =>
    b.setAttribute("aria-checked", String(b.dataset.sprache === sprache)),
  );
  fuelleListen();
  invoke("sprache_setzen", { englisch: sprache === "en" }).catch(() => {});
}

function spracheWechseln(s: Sprache) {
  if (s === sprache) return;
  spracheSetzen(s);
  spracheAnwenden();
  zeichneGeraet();
  zeichneAlles();
}

async function starten() {
  katalog = await invoke<Katalog>("katalog");
  spracheAnwenden();
  zeichneGeraet();

  try {
    k = await invoke<Konfiguration>("konfig_laden");
  } catch (e) {
    melden(t("meldung.konfigFehlerhaft", { fehler: String(e) }), "fehler");
    k = await invoke<Konfiguration>("beispiel");
  }
  profilAuffuellen();
  gespeichert = schnappschuss();
  profilIndex = Math.min(Math.max(k.allgemein.aktives_profil - 1, 0), 2);
  editorArt = artVon(profil().belegung[tasteName() ?? ""]);
  if (editorArt === "keine") editorArt = "kombi";

  verdrahten();
  zeichneAlles();
  skalieren();

  await listen<string>("meldung", (e) => melden(e.payload));
  await listen<number>("fortschritt", (e) => fortschritt(e.payload));

  await statusAbfragen(true);
  window.setInterval(() => statusAbfragen(true), 4000);
}

// Im normalen Browser (pnpm dev ohne Tauri) mit Beispieldaten anzeigen
if (import.meta.env.DEV && !("__TAURI_INTERNALS__" in window)) {
  import("./vorschau").then((v) => v.einrichten()).then(starten);
} else {
  starten();
}
