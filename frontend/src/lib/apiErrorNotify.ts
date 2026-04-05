import type { MessageKey } from "../i18n/locales/en";

export type ApiAnnouncedError =
  | { source: "text"; text: string }
  | { source: "i18n"; key: MessageKey };

type Listener = (detail: ApiAnnouncedError) => void;

let listener: Listener | null = null;

export function setApiErrorListener(fn: Listener | null): void {
  listener = fn;
}

export function announceApiError(detail: ApiAnnouncedError): void {
  listener?.(detail);
}
