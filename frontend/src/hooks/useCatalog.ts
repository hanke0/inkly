import { useCallback, useEffect, useState } from "react";

import { fetchCatalog } from "../api";
import { useI18n } from "../i18n/context";
import { extractErrorMessage } from "../lib/errors";
import type { CatalogResponse } from "../types";

export function useCatalog(catalogUrlPath: string) {
  const { t } = useI18n();
  const [catalog, setCatalog] = useState<CatalogResponse | null>(null);
  const [catalogLoading, setCatalogLoading] = useState(false);
  const [catalogErr, setCatalogErr] = useState("");

  const loadCatalog = useCallback(async () => {
    setCatalogErr("");
    setCatalogLoading(true);
    try {
      const c = await fetchCatalog(catalogUrlPath);
      setCatalog(c);
    } catch (e) {
      setCatalog(null);
      setCatalogErr(extractErrorMessage(e, t("errors.catalogFailed")));
    } finally {
      setCatalogLoading(false);
    }
  }, [catalogUrlPath, t]);

  useEffect(() => {
    void loadCatalog();
  }, [loadCatalog]);

  return { catalog, catalogLoading, catalogErr, reloadCatalog: loadCatalog };
}
