import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { COVERAGE_ITEMS } from "../domain/coverage";
import { CoverageMatrixPage } from "../features/coverage/CoverageMatrixPage";
import { AppShell } from "../shared/ui/AppShell";
import type { DatabaseHealth } from "../shared/ui/TracePanel";
import type { AppRouteId } from "./routes";
import { getAppRoute } from "./routes";

export function App() {
  const [activeRoute, setActiveRoute] = useState<AppRouteId>("coverage");
  const [searchQuery, setSearchQuery] = useState("");
  const [selectedCoverageId, setSelectedCoverageId] = useState(COVERAGE_ITEMS[0].id);
  const [coverageRefreshedAt, setCoverageRefreshedAt] = useState(new Date());
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

    return () => {
      isMounted = false;
    };
  }, []);

  return (
    <div className="app-root">
      <AppShell
        activeRoute={activeRoute}
        onRouteChange={setActiveRoute}
        searchQuery={searchQuery}
        onSearchQueryChange={setSearchQuery}
        chapterItems={COVERAGE_ITEMS}
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
            searchQuery={searchQuery}
            selectedItemId={selectedCoverageId}
            lastRefreshedAt={coverageRefreshedAt}
            onOpenItem={setSelectedCoverageId}
            onRefresh={() => setCoverageRefreshedAt(new Date())}
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
        {route.description}。该页面入口已纳入 Phase 1 导航骨架，具体业务实现按 DEV-PLAN 后续阶段推进。
      </p>
    </section>
  );
}
