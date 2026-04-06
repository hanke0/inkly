import {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useMemo,
  useRef,
  useState,
  type ReactNode,
} from 'react';

import { setPreferredAcceptLanguage } from '../api';
import {
  interpolate,
  loadLocaleMessages,
  type AppLocale,
  type MessageKey,
} from './strings';

export type { AppLocale, MessageKey };

function localeFromNavigator(): AppLocale {
  if (typeof navigator === 'undefined') {
    return 'en';
  }
  for (const raw of navigator.languages ?? [navigator.language]) {
    const tag = raw?.trim().toLowerCase() ?? '';
    if (tag.startsWith('zh')) {
      return 'zh-Hans';
    }
    if (tag.startsWith('en')) {
      return 'en';
    }
  }
  return 'en';
}

export function normalizeApiLocale(tag: string): AppLocale {
  const t = tag.trim().toLowerCase();
  if (t === 'zh-hans' || t.startsWith('zh')) {
    return 'zh-Hans';
  }
  return 'en';
}

function acceptLanguageHeader(loc: AppLocale): string {
  return loc === 'zh-Hans' ? 'zh-CN,zh;q=0.9,en;q=0.8' : 'en-US,en;q=0.9';
}

type I18nContextValue = {
  locale: AppLocale;
  setLocale: (loc: AppLocale) => void;
  t: (key: MessageKey) => string;
  tf: (key: MessageKey, vars: Record<string, string | number>) => string;
};

const I18nContext = createContext<I18nContextValue | null>(null);

export function I18nProvider({ children }: { children: ReactNode }) {
  const [locale, setLocaleState] = useState<AppLocale>(() =>
    localeFromNavigator(),
  );
  const [bundle, setBundle] = useState<{
    loc: AppLocale;
    messages: Record<MessageKey, string>;
  } | null>(null);
  const localeRef = useRef(locale);
  localeRef.current = locale;

  const setLocale = useCallback((loc: AppLocale) => {
    setLocaleState(loc);
  }, []);

  useEffect(() => {
    setPreferredAcceptLanguage(acceptLanguageHeader(locale));
  }, [locale]);

  useEffect(() => {
    const requested = locale;
    void loadLocaleMessages(requested).then((table) => {
      if (localeRef.current !== requested) {
        return;
      }
      setBundle({ loc: requested, messages: table });
    });
  }, [locale]);

  const value = useMemo((): I18nContextValue | null => {
    if (!bundle || bundle.loc !== locale) {
      return null;
    }
    const { messages } = bundle;
    return {
      locale,
      setLocale,
      t: (key) => messages[key],
      tf: (key, vars) => interpolate(messages[key], vars),
    };
  }, [bundle, locale, setLocale]);

  if (!value) {
    return (
      <div className="flex min-h-screen items-center justify-center bg-neutral-50 text-neutral-500 dark:bg-neutral-950 dark:text-neutral-400">
        Loading…
      </div>
    );
  }

  return <I18nContext.Provider value={value}>{children}</I18nContext.Provider>;
}

export function useI18n(): I18nContextValue {
  const ctx = useContext(I18nContext);
  if (!ctx) {
    throw new Error('useI18n must be used within I18nProvider');
  }
  return ctx;
}
