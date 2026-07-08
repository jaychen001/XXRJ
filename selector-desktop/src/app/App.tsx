import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { CalculationCaseDraft } from "../domain/calculation";
import { CalculationPage } from "../features/calculation/CalculationPage";
import { AppShell } from "../shared/ui/AppShell";
import type { DatabaseHealth } from "../shared/ui/TracePanel";
import type { AppRouteId } from "./routes";

export function App() {
  const [activeRoute, setActiveRoute] = useState<AppRouteId>("calculation");
  const [health, setHealth] = useState<DatabaseHealth | null>(null);
  const [isHealthLoading, setIsHealthLoading] = useState(true);
  const [calculationDraft] = useState<CalculationCaseDraft | null>(null);

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
        health={health}
        isHealthLoading={isHealthLoading}
        globalErrorMessage={health?.status === "error" ? health.message : null}
      >
        <CalculationPage draft={calculationDraft} />
      </AppShell>
    </div>
  );
}

function toHealthErrorMessage(error: unknown): string {
  const message = error instanceof Error ? error.message : String(error);

  if (message.toLowerCase().includes("invoke")) {
    return "浏览器预览模式不能检查本地数据库，请在桌面应用中运行。";
  }

  return message || "当前不是桌面运行环境。";
}
