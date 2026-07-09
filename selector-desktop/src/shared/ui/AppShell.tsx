import type { ReactNode } from "react";
import { AlertTriangle, Calculator, HardDrive, ShieldCheck } from "lucide-react";
import type { AppRoute, AppRouteId } from "../../app/routes";
import { APP_ROUTES } from "../../app/routes";
import { RiskBadge } from "./RiskBadge";
import type { DatabaseHealth } from "./TracePanel";
import "./app-shell.css";
import "./screen-reader.css";

interface AppShellProps {
  activeRoute: AppRouteId;
  onRouteChange: (routeId: AppRouteId) => void;
  health: DatabaseHealth | null;
  isHealthLoading: boolean;
  globalErrorMessage: string | null;
  children: ReactNode;
}

export function AppShell({
  activeRoute,
  onRouteChange,
  health,
  isHealthLoading,
  globalErrorMessage,
  children,
}: AppShellProps) {
  return (
    <div className="app-shell">
      <aside className="app-shell__sidebar" aria-label="主导航">
        <div className="app-shell__brand">
          <Calculator size={22} aria-hidden="true" />
          <div>
            <strong>非标选型计算</strong>
            <span>机械设计离线工具</span>
          </div>
        </div>

        <nav className="app-shell__nav-list" aria-label="页面导航">
          {APP_ROUTES.map((route) => (
            <RouteButton
              key={route.id}
              route={route}
              isActive={route.id === activeRoute}
              onClick={() => onRouteChange(route.id)}
            />
          ))}
        </nav>

        <div className="app-shell__sidebar-note" aria-label="应用状态说明">
          <span>
            <HardDrive size={15} aria-hidden="true" />
            本地计算
          </span>
          <span>
            <ShieldCheck size={15} aria-hidden="true" />
            手动确认安全系数
          </span>
        </div>
      </aside>

      <div className="app-shell__body">
        <header className="app-shell__topbar">
          <div>
            <strong>工程选型计算器</strong>
            <span>按已知工况输入，直接得到公式过程、风险判断和报告导出。</span>
          </div>
          <div className="app-shell__status">
            {globalErrorMessage ? (
              <div className="app-shell__alert" role="alert" title={globalErrorMessage}>
                <AlertTriangle size={14} aria-hidden="true" />
                <span>本地数据检查失败：{globalErrorMessage}</span>
              </div>
            ) : null}
            <RiskBadge tone={health?.status === "ok" ? "success" : "warning"}>
              {isHealthLoading
                ? "本地数据检查中"
                : health?.status === "ok"
                  ? "本地数据正常"
                  : "本地数据待检查"}
            </RiskBadge>
          </div>
        </header>

        <main className="app-shell__content">{children}</main>
      </div>
    </div>
  );
}

interface RouteButtonProps {
  route: AppRoute;
  isActive: boolean;
  onClick: () => void;
}

function RouteButton({ route, isActive, onClick }: RouteButtonProps) {
  const Icon = route.icon;

  return (
    <button
      className={`route-button${isActive ? " route-button--active" : ""}`}
      type="button"
      aria-current={isActive ? "page" : undefined}
      onClick={onClick}
      title={`${route.label}：${route.description}`}
    >
      <Icon size={18} aria-hidden="true" />
      <span>{route.label}</span>
    </button>
  );
}
