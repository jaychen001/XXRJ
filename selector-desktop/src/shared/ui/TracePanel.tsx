import { Calculator, Database, PanelRightClose, PanelRightOpen, ShieldAlert } from "lucide-react";
import { RiskBadge } from "./RiskBadge";
import "./trace-panel.css";

export interface DatabaseHealth {
  status: "ok" | "error";
  databasePath: string;
  appliedMigrations: number;
  tableCount: number;
  message: string;
}

interface TracePanelProps {
  health: DatabaseHealth | null;
  isCollapsed: boolean;
  isLoading: boolean;
  onToggleCollapsed: () => void;
}

export function TracePanel({
  health,
  isCollapsed,
  isLoading,
  onToggleCollapsed,
}: TracePanelProps) {
  const healthTone = health?.status === "ok" ? "success" : "warning";

  return (
    <aside
      className={`trace-panel${isCollapsed ? " trace-panel--collapsed" : ""}`}
      aria-label="数据与状态"
    >
      <div className="trace-panel__header">
        <h2>数据与状态</h2>
        <div className="trace-panel__header-actions">
          <RiskBadge tone={isLoading ? "neutral" : healthTone}>
            {isLoading ? "检查中" : health?.status === "ok" ? "数据库正常" : "待连接"}
          </RiskBadge>
          <button
            className="trace-panel__toggle"
            type="button"
            aria-label={isCollapsed ? "展开状态区" : "折叠状态区"}
            title={isCollapsed ? "展开状态区" : "折叠状态区"}
            onClick={onToggleCollapsed}
          >
            {isCollapsed ? (
              <PanelRightOpen size={16} aria-hidden="true" />
            ) : (
              <PanelRightClose size={16} aria-hidden="true" />
            )}
          </button>
        </div>
      </div>

      {isCollapsed ? null : (
        <>
          <section className="trace-section">
            <div className="trace-section__title">
              <Database size={16} aria-hidden="true" />
              <span>数据库</span>
            </div>
            <dl className="trace-list">
              <div>
                <dt>状态</dt>
                <dd>{health?.message ?? "等待 Tauri 环境返回健康状态"}</dd>
              </div>
              <div>
                <dt>迁移</dt>
                <dd>{health ? `${health.appliedMigrations} 条已记录` : "-"}</dd>
              </div>
              <div>
                <dt>表数量</dt>
                <dd>{health ? `${health.tableCount} 张` : "-"}</dd>
              </div>
            </dl>
            <p className="trace-path">{health?.databasePath ?? "浏览器预览模式下不创建本地库"}</p>
          </section>

          <section className="trace-section">
            <div className="trace-section__title">
              <Calculator size={16} aria-hidden="true" />
              <span>工程公式库</span>
            </div>
            <p>
              计算按工程公式、单位换算和安全系数执行，结果页只展示过程、结论和风险。
            </p>
          </section>

          <section className="trace-section">
            <div className="trace-section__title">
              <ShieldAlert size={16} aria-hidden="true" />
              <span>当前边界</span>
            </div>
            <p>
              计算结果用于工程初选；最终型号仍需结合厂家样本、安装空间和现场工况复核。
            </p>
          </section>
        </>
      )}
    </aside>
  );
}
