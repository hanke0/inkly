import { useCallback, useEffect, useState } from "react";

import { fetchCatalog } from "../api";
import type { CatalogResponse } from "../types";

export function useCatalog(catalogUrlPath: string) {
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
      setCatalogErr(e instanceof Error ? e.message : "Catalog request failed.");
    } finally {
      setCatalogLoading(false);
    }
  }, [catalogUrlPath]);

  useEffect(() => {
    void loadCatalog();
  }, [loadCatalog]);

  return { catalog, catalogLoading, catalogErr, reloadCatalog: loadCatalog };
}
