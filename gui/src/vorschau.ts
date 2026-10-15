// Nur für `pnpm dev` im normalen Browser: ersetzt das Tauri-Backend durch Beispieldaten.
import { mockIPC, mockWindows } from "@tauri-apps/api/mocks";

const TASTEN = [
  ["a", 0x04], ["c", 0x06], ["s", 0x16], ["v", 0x19], ["z", 0x1c], ["1", 0x1e], ["2", 0x1f], ["3", 0x20],
  ["4", 0x21], ["5", 0x22], ["6", 0x23], ["7", 0x24], ["8", 0x25], ["eingabe", 0x28], ["esc", 0x29],
  ["+", 0x30], ["f13", 0x68], ["f14", 0x69], ["strg", 0xe0], ["umschalt", 0xe1], ["alt", 0xe2], ["win", 0xe3], ["plus", 0x30],
].map(([name, code]) => ({ name, code, text: "" }));

const MEDIEN = [
  ["abspielen", 0xcd, "Wiedergabe/Pause"],
  ["stumm", 0xe2, "Stummschalten"],
  ["lauter", 0xe9, "Lautstärke +"],
  ["leiser", 0xea, "Lautstärke −"],
  ["browser", 0x223, "Browser-Startseite"],
  ["rechner", 0x192, "Taschenrechner"],
].map(([name, code, text]) => ({ name, code, text }));

const EINFACH = [
  ["aus", 0, "Taste ohne Funktion"],
  ["desktop_anzeigen", 7, "Desktop anzeigen"],
  ["profil_1", 0x17, "Zu Profil 1 wechseln"],
  ["led_zyklus", 0x15, "Beleuchtungsmodus wechseln"],
].map(([name, code, text]) => ({ name, code, text }));

const tasten = [7, 3, 2, 1, 9, 6, 5, 4].map((s, i) => ({ name: `t${i + 1}`, steckplatz: s }));

const konfiguration = {
  geraet: { vendor_id: 0x30fa, product_id: 0x1340, bericht_laenge: 8 },
  allgemein: { aktives_profil: 1, abfragerate_hz: 1000, dauerfeuer_ms: 48 },
  beleuchtung: { modus: "dauerlicht", helligkeit: 4, geschwindigkeit: 3 },
  taste: tasten,
  profil: [
    {
      name: "Arbeit",
      belegung: {
        t1: "kombi:strg+c", t2: "kombi:strg+v", t3: "medien:stumm", t4: "medien:abspielen",
        t5: "text:name@beispiel.de", t6: "kombi:win+umschalt+s", t7: "desktop_anzeigen",
      },
      led_reaktion: {},
      bezeichnung: { t1: "Kopieren", t2: "Einfügen", t6: "Screenshot" },
    },
    { name: "Profil 2", belegung: { t1: "taste:1" }, led_reaktion: {} },
    { name: "Profil 3", belegung: {}, led_reaktion: {} },
  ],
  erweitert: {},
};

export function einrichten() {
  mockWindows("main");
  mockIPC(
    (cmd, args) => {
      const a = args as Record<string, unknown>;
      switch (cmd) {
        case "katalog":
          return { tasten: TASTEN, medien: MEDIEN, einfach: EINFACH };
        case "konfig_laden":
        case "beispiel":
          return structuredClone(konfiguration);
        case "funktion_pruefen":
          return String(a.text).startsWith("text:") ? String(a.text) : String(a.text).toLowerCase();
        case "status":
          return { profil: 1, hardware_profil: 0, makro_ende: 0xff8, roh: "" };
        case "plugin:dialog|ask":
          return true;
        default:
          return null;
      }
    },
    { shouldMockEvents: true },
  );
}
