import type { LangTag } from "./types";

export function detectBrowserLang(): LangTag {
  if (typeof navigator === "undefined") {
    return "zh";
  }

  const preferred = Array.isArray(navigator.languages)
    ? [...navigator.languages, navigator.language]
    : [navigator.language];

  for (const tag of preferred) {
    const normalized = tag.trim().toLowerCase();
    if (normalized.startsWith("zh")) {
      return "zh";
    }
    if (normalized.startsWith("en")) {
      return "en";
    }
  }

  return "zh";
}

export function translateByLang(lang: LangTag, zh: string, en: string): string {
  return lang === "zh" ? zh : en;
}
