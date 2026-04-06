import type { MessageKey } from './locales/en';

export type AppLocale = 'en' | 'zh-Hans';

export type { MessageKey };

/** Loads one locale bundle (separate Vite chunk per language). */
export async function loadLocaleMessages(
  locale: AppLocale,
): Promise<Record<MessageKey, string>> {
  switch (locale) {
    case 'en':
      return (await import('./locales/en')).messages;
    case 'zh-Hans':
      return (await import('./locales/zh-Hans')).messages;
  }
}

export function interpolate(
  template: string,
  vars: Record<string, string | number>,
): string {
  return template.replace(/\{(\w+)\}/g, (_, key: string) =>
    Object.prototype.hasOwnProperty.call(vars, key)
      ? String(vars[key])
      : `{${key}}`,
  );
}
