import {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useMemo,
  useState,
  type ReactNode,
} from "react";

import { setPreferredAcceptLanguage } from "../api";
import { type AppLocale, interpolate, MESSAGES, type MessageKey } from "./strings";

export type { AppLocale, MessageKey };

function localeFromNavigator(): AppLocale {
  if (typeof navigator === "undefined") {
    return "en";
  }
  for (const raw of navigator.languages ?? [navigator.language]) {
    const tag = raw?.trim().toLowerCase() ?? "";
    if (tag.startsWith("zh")) {
      return "zh-Hans";
    }
    if (tag.startsWith("en")) {
      return "en";
    }
  }
  return "en";
}

export function normalizeApiLocale(tag: string): AppLocale {
  const t = tag.trim().toLowerCase();
  if (t === "zh-hans" || t.startsWith("zh")) {
    return "zh-Hans";
  }
  return "en";
}

function acceptLanguageHeader(loc: AppLocale): string {
  return loc === "zh-Hans" ? "zh-CN,zh;q=0.9,en;q=0.8" : "en-US,en;q=0.9";
}

type I18nContextValue = {
  locale: AppLocale;
  setLocale: (loc: AppLocale) => void;
  t: (key: MessageKey) => string;
  tf: (key: MessageKey, vars: Record<string, string | number>) => string;
};

const I18nContext = createContext<I18nContextValue | null>(null);

export function I18nProvider({ children }: { children: ReactNode }) {
  const [locale, setLocaleState] = useState<AppLocale>(() => localeFromNavigator());

  const setLocale = useCallback((loc: AppLocale) => {
    setLocaleState(loc);
  }, []);

  useEffect(() => {
    setPreferredAcceptLanguage(acceptLanguageHeader(locale));
  }, [locale]);

  const value = useMemo((): I18nContextValue => {
    const table = MESSAGES[locale];
    return {
      locale,
      setLocale,
      t: (key) => table[key],
      tf: (key, vars) => interpolate(table[key], vars),
    };
  }, [locale, setLocale]);

  return <I18nContext.Provider value={value}>{children}</I18nContext.Provider>;
}

export function useI18n(): I18nContextValue {
  const ctx = useContext(I18nContext);
  if (!ctx) {
    throw new Error("useI18n must be used within I18nProvider");
  }
  return ctx;
}
