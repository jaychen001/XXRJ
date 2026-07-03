import { useEffect, useState } from "react";
import { DatabaseZap, Search } from "lucide-react";
import type { KnowledgeSearchRecord, RootPdfIngestSummary } from "../../domain/knowledge";
import { listRecentKnowledgeEntries, searchKnowledgeEntries } from "../../shared/api/knowledge";
import "./knowledge-search-page.css";

interface KnowledgeSearchPageProps {
  ingestSummary: RootPdfIngestSummary | null;
  onIngestRootPdf: () => Promise<RootPdfIngestSummary>;
}

export function KnowledgeSearchPage({
  ingestSummary,
  onIngestRootPdf,
}: KnowledgeSearchPageProps) {
  const [query, setQuery] = useState("惯量比");
  const [results, setResults] = useState<KnowledgeSearchRecord[]>([]);
  const [recentEntries, setRecentEntries] = useState<KnowledgeSearchRecord[]>([]);
  const [status, setStatus] = useState("尚未检索");
  const [isBusy, setIsBusy] = useState(false);
  const [hasIndex, setHasIndex] = useState(Boolean(ingestSummary));

  useEffect(() => {
    void loadRecentEntries();
  }, []);

  async function loadRecentEntries(indexExists = Boolean(ingestSummary)) {
    try {
      const entries = await listRecentKnowledgeEntries();
      setRecentEntries(entries);
      setHasIndex(entries.length > 0 || indexExists);
      if (entries.length > 0) {
        setStatus(`最近引用 ${entries.length} 条`);
      }
    } catch (error: unknown) {
      setStatus(toErrorMessage(error));
    }
  }

  async function handleIngest() {
    setIsBusy(true);
    setStatus("正在读取根目录 PDF");
    try {
      const summary = await onIngestRootPdf();
      setHasIndex(summary.knowledgeEntryCount > 0);
      await loadRecentEntries(summary.knowledgeEntryCount > 0);
      setStatus(
        `索引完成：${summary.pageCount} 页，${summary.catalogCount} 条目录，${summary.knowledgeEntryCount} 条知识`,
      );
    } catch (error: unknown) {
      setStatus(toErrorMessage(error));
    } finally {
      setIsBusy(false);
    }
  }

  async function handleSearch() {
    const trimmed = query.trim();
    if (!hasIndex) {
      setStatus("索引未建立，请先建立/刷新索引");
      return;
    }
    if (!trimmed) {
      setStatus("搜索词不能为空");
      return;
    }

    setIsBusy(true);
    setStatus("检索中");
    try {
      const nextResults = await searchKnowledgeEntries(trimmed);
      setResults(nextResults);
      setStatus(nextResults.length > 0 ? `命中 ${nextResults.length} 条` : "无结果");
    } catch (error: unknown) {
      setStatus(toErrorMessage(error));
    } finally {
      setIsBusy(false);
    }
  }

  const canSearch = hasIndex && !isBusy;
  const visibleEntries = results.length > 0 ? results : recentEntries;

  return (
    <section className="knowledge-page" aria-labelledby="knowledge-title">
      <div className="knowledge-page__header">
        <div>
          <h1 className="page-title" id="knowledge-title">
            本地知识检索
          </h1>
          <p className="page-subtitle">
            搜索根目录 PDF 的抽取条目，结果只作为方法依据，不替代计算结果。
          </p>
        </div>
        <button className="secondary-button" type="button" disabled={isBusy} onClick={handleIngest}>
          <DatabaseZap size={16} aria-hidden="true" />
          建立/刷新索引
        </button>
      </div>

      <div className="knowledge-toolbar">
        <label className="knowledge-toolbar__search">
          <Search size={16} aria-hidden="true" />
          <span className="sr-only">知识检索词</span>
          <input
            value={query}
            maxLength={100}
            onChange={(event) => setQuery(event.target.value)}
            aria-disabled={!hasIndex}
            onKeyDown={(event) => {
              if (event.key === "Enter" && canSearch) {
                void handleSearch();
              }
            }}
          />
        </label>
        <button className="secondary-button" type="button" disabled={!canSearch} onClick={handleSearch}>
          搜索
        </button>
      </div>

      <div className="knowledge-status" role="status">
        <span>{status}</span>
        {ingestSummary ? <span>{ingestSummary.pdfPath}</span> : <span>索引未建立时搜索禁用</span>}
      </div>

      <div className="knowledge-results" aria-label="知识检索结果">
        {visibleEntries.map((result) => (
          <article className="knowledge-result" key={result.id}>
            <header>
              <h2>{result.title}</h2>
              <span>{result.page ?? "页码待确认"}</span>
            </header>
            <p>{result.content}</p>
            <footer>
              <span>{result.sourceTitle}</span>
              <span>{result.tags.join(" / ")}</span>
            </footer>
          </article>
        ))}
        {visibleEntries.length === 0 ? (
          <p className="knowledge-results__empty">先建立索引；验收词包括惯量比、摩擦系数、负载率。</p>
        ) : null}
      </div>
    </section>
  );
}

function toErrorMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}
