import { useState } from "react";

import { search } from "../api";
import { DEFAULT_SEARCH_LIMIT, type BrandHeaderSearchProps } from "../components/BrandHeader";
import { extractErrorMessage } from "../lib/errors";
import type { SearchQuery, SearchResponse } from "../types";

export function useSearch(catalogPath: string) {
  const [q, setQ] = useState("");
  const [limit, setLimit] = useState(DEFAULT_SEARCH_LIMIT);
  const [limitToFolder, setLimitToFolder] = useState(true);
  const [tagsFilter, setTagsFilter] = useState("");
  const [searchSummary, setSearchSummary] = useState<string | undefined>();
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState("");
  const [results, setResults] = useState<SearchResponse | null>(null);
  const [resultsOpen, setResultsOpen] = useState(false);

  async function runSearch() {
    setError("");
    setResults(null);
    setResultsOpen(false);
    setLoading(true);

    const trimmed = q.trim();
    const tagParts = tagsFilter
      .split(",")
      .map((t) => t.trim())
      .filter(Boolean);
    const usePath = limitToFolder && catalogPath !== "/";

    if (!trimmed && !usePath && tagParts.length === 0) {
      setLoading(false);
      setError("Enter keywords, tags (in settings), or limit to the current folder.");
      return;
    }

    const query: SearchQuery = {
      q: trimmed,
      limit: Math.max(1, Math.min(50, limit)),
    };
    if (usePath) {
      query.path = catalogPath;
    }
    if (tagParts.length > 0) {
      query.tags = tagParts.join(",");
    }

    const summaryParts: string[] = [];
    if (trimmed) {
      summaryParts.push(trimmed);
    }
    if (usePath) {
      summaryParts.push(`in ${catalogPath}`);
    }
    if (tagParts.length > 0) {
      summaryParts.push(`tags: ${tagParts.join(", ")}`);
    }
    setSearchSummary(summaryParts.length > 0 ? summaryParts.join(" · ") : undefined);

    try {
      const res = await search(query);
      setResults(res);
      setResultsOpen(true);
    } catch (err) {
      setError(extractErrorMessage(err, "Search request failed."));
    } finally {
      setLoading(false);
    }
  }

  const headerProps: BrandHeaderSearchProps = {
    q,
    onQChange: setQ,
    limit,
    onLimitChange: setLimit,
    onSearch: () => {
      void runSearch();
    },
    loading,
    catalogPath,
    limitToFolder,
    onLimitToFolderChange: setLimitToFolder,
    tagsFilter,
    onTagsFilterChange: setTagsFilter,
  };

  return {
    headerProps,
    error,
    results,
    resultsOpen,
    closeResults: () => setResultsOpen(false),
    searchSummary,
  };
}
