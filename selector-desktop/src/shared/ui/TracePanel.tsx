import { Database, FileText, PanelRightClose, PanelRightOpen, ShieldAlert } from "lucide-react";
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
      aria-label="来源与状态追溯"
    >
      <div className="trace-panel__header">
        <h2>来源与状态</h2>
        <div className="trace-panel__header-actions">
          <RiskBadge tone={isLoading ? "neutral" : healthTone}>
            {isLoading ? "检查中" : health?.status === "ok" ? "数据库正常" : "待连接"}
          </RiskBadge>
          <button
            className="trace-panel__toggle"
            type="button"
            aria-label={isCollapsed ? "展开追溯区" : "折叠追溯区"}
            title={isCollapsed ? "展开追溯区" : "折叠追溯区"}
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
              <FileText size={16} aria-hidden="true" />
              <span>根目录 PDF</span>
            </div>
            <p>
              当前阶段只建立入口和追溯骨架；PDF 解析、页码索引和参数候选抽取在 Phase 2 落地。
            </p>
          </section>

          <section className="trace-section">
            <div className="trace-section__title">
              <ShieldAlert size={16} aria-hidden="true" />
              <span>Phase 1 边界</span>
            </div>
            <p>
              本阶段不实现计算公式，不推荐型号，不导入厂家样本；只保证桌面壳、视觉骨架和数据库底座可运行。
            </p>
          </section>
        </>
      )}
    </aside>
  );
}
