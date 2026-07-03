import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { COVERAGE_ITEMS, type CoverageItem } from "../domain/coverage";
import { mapCoverageRecord, type RootPdfIngestSummary } from "../domain/knowledge";
import { CoverageMatrixPage } from "../features/coverage/CoverageMatrixPage";
import { KnowledgeSearchPage } from "../features/knowledge/KnowledgeSearchPage";
import { ParameterCandidatePage } from "../features/parameters/ParameterCandidatePage";
import { getPdfCoverageItems, ingestRootPdfNote } from "../shared/api/knowledge";
import { AppShell } from "../shared/ui/AppShell";
import type { DatabaseHealth } from "../shared/ui/TracePanel";
import type { AppRouteId } from "./routes";
import { getAppRoute } from "./routes";

export function App() {
  const [activeRoute, setActiveRoute] = useState<AppRouteId>("coverage");
  const [searchQuery, setSearchQuery] = useState("");
  const [coverageItems, setCoverageItems] = useState<CoverageItem[]>(COVERAGE_ITEMS);
  const [selectedCoverageId, setSelectedCoverageId] = useState(COVERAGE_ITEMS[0].id);
  const [coverageRefreshedAt, setCoverageRefreshedAt] = useState(new Date());
  const [ingestSummary, setIngestSummary] = useState<RootPdfIngestSummary | null>(null);
  const [health, setHealth] = useState<DatabaseHealth | null>(null);
  const [isHealthLoading, setIsHealthLoading] = useState(true);

  useEffect(() => {
    let isMounted = true;

    async function loadHealth() {
      setIsHealthLoading(true);
      try {
        const result = await invoke<DatabaseHealth>("get_database_health");
        if (isMounted) {
          setHealth(result);
        }
      } catch (error: unknown) {
        if (isMounted) {
          setHealth({
            status: "error",
            databasePath: "",
            appliedMigrations: 0,
            tableCount: 0,
            message: toHealthErrorMessage(error),
          });
        }
      } finally {
        if (isMounted) {
          setIsHealthLoading(false);
        }
      }
    }

    void loadHealth();
    void loadCoverageItems();

    return () => {
      isMounted = false;
    };
  }, []);

  async function loadCoverageItems() {
    try {
      const records = await getPdfCoverageItems();
      const mappedItems = records.map(mapCoverageRecord);
      if (mappedItems.length > 0) {
        setCoverageItems(mappedItems);
        if (!mappedItems.some((item) => item.id === selectedCoverageId)) {
          setSelectedCoverageId(mappedItems[0].id);
        }
      }
    } catch {
      setCoverageItems(COVERAGE_ITEMS);
    } finally {
      setCoverageRefreshedAt(new Date());
    }
  }

  async function handleIngestRootPdf() {
    const summary = await ingestRootPdfNote();
    setIngestSummary(summary);
    await loadCoverageItems();
    return summary;
  }

  return (
    <div className="app-root">
      <AppShell
        activeRoute={activeRoute}
        onRouteChange={setActiveRoute}
        searchQuery={searchQuery}
        onSearchQueryChange={setSearchQuery}
        chapterItems={coverageItems}
        activeChapterId={selectedCoverageId}
        onChapterOpen={(chapterId) => {
          setActiveRoute("coverage");
          setSelectedCoverageId(chapterId);
        }}
        health={health}
        isHealthLoading={isHealthLoading}
        globalErrorMessage={health?.status === "error" ? health.message : null}
      >
        {activeRoute === "coverage" ? (
          <CoverageMatrixPage
            items={coverageItems}
            searchQuery={searchQuery}
            selectedItemId={selectedCoverageId}
            lastRefreshedAt={coverageRefreshedAt}
            onOpenItem={setSelectedCoverageId}
            onIngestRootPdf={handleIngestRootPdf}
          />
        ) : activeRoute === "knowledge" ? (
          <KnowledgeSearchPage
            ingestSummary={ingestSummary}
            onIngestRootPdf={handleIngestRootPdf}
          />
        ) : activeRoute === "parameters" ? (
          <ParameterCandidatePage
            ingestSummary={ingestSummary}
            onIngestRootPdf={handleIngestRootPdf}
          />
        ) : (
          <PlaceholderPage routeId={activeRoute} />
        )}
      </AppShell>
    </div>
  );
}

function toHealthErrorMessage(error: unknown): string {
  const message = error instanceof Error ? error.message : String(error);

  if (message.toLowerCase().includes("invoke")) {
    return "浏览器预览模式：数据库检查需在 Tauri 桌面壳内运行";
  }

  return message || "当前不是 Tauri 运行环境";
}

interface PlaceholderPageProps {
  routeId: AppRouteId;
}

function PlaceholderPage({ routeId }: PlaceholderPageProps) {
  const route = getAppRoute(routeId);

  return (
    <section aria-labelledby="placeholder-title">
      <h1 className="page-title" id="placeholder-title">
        {route.label}
      </h1>
      <p className="page-subtitle">
        {route.description}。该页面入口已纳入导航，具体业务实现按 DEV-PLAN 后续阶段推进。
      </p>
    </section>
  );
}
