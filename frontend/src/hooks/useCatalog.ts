import { useCallback, useEffect, useState } from 'react';

import { fetchCatalog } from '../api';
import type { CatalogResponse } from '../types';

export function useCatalog(catalogUrlPath: string) {
  const [catalog, setCatalog] = useState<CatalogResponse | null>(null);
  const [catalogLoading, setCatalogLoading] = useState(false);

  const loadCatalog = useCallback(async () => {
    setCatalogLoading(true);
    try {
      const c = await fetchCatalog(catalogUrlPath);
      setCatalog(c);
    } catch {
      setCatalog(null);
    } finally {
      setCatalogLoading(false);
    }
  }, [catalogUrlPath]);

  useEffect(() => {
    void loadCatalog();
  }, [loadCatalog]);

  return { catalog, catalogLoading, reloadCatalog: loadCatalog };
}
