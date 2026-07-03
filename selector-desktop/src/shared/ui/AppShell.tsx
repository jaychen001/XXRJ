import { useEffect, useRef, useState, type ReactNode } from "react";
import {
  AlertTriangle,
  Database,
  PanelLeftClose,
  PanelLeftOpen,
  Search,
} from "lucide-react";
import type { AppRoute, AppRouteId } from "../../app/routes";
import { APP_ROUTES } from "../../app/routes";
import { RiskBadge } from "./RiskBadge";
import type { DatabaseHealth } from "./TracePanel";
import { TracePanel } from "./TracePanel";
import "./app-shell.css";
import "./screen-reader.css";

interface AppShellProps {
  activeRoute: AppRouteId;
  onRouteChange: (routeId: AppRouteId) => void;
  searchQuery: string;
  onSearchQueryChange: (query: string) => void;
  chapterItems: ChapterNavItem[];
  activeChapterId: string;
  onChapterOpen: (chapterId: string) => void;
  health: DatabaseHealth | null;
  isHealthLoading: boolean;
  globalErrorMessage: string | null;
  children: ReactNode;
}

interface ChapterNavItem {
  id: string;
  chapter: string;
  requirement: string;
}

export function AppShell({
  activeRoute,
  onRouteChange,
  searchQuery,
  onSearchQueryChange,
  chapterItems,
  activeChapterId,
  onChapterOpen,
  health,
  isHealthLoading,
  globalErrorMessage,
  children,
}: AppShellProps) {
  const searchInputRef = useRef<HTMLInputElement>(null);
  const [isSidebarCollapsed, setIsSidebarCollapsed] = useState(false);
  const [isTraceCollapsed, setIsTraceCollapsed] = useState(false);
  const normalizedQuery = searchQuery.trim().toLowerCase();
  const filteredChapters = normalizedQuery
    ? chapterItems.filter((chapter) =>
        `${chapter.chapter} ${chapter.requirement}`.toLowerCase().includes(normalizedQuery),
      )
    : chapterItems;

  useEffect(() => {
    function handleKeyDown(event: KeyboardEvent) {
      if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === "k") {
        event.preventDefault();
        searchInputRef.current?.focus();
      }
    }

    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, []);

  return (
    <div
      className={`app-shell${isSidebarCollapsed ? " app-shell--sidebar-collapsed" : ""}${
        isTraceCollapsed ? " app-shell--trace-collapsed" : ""
      }`}
    >
      <nav className="app-shell__sidebar" aria-label="主导航">
        <div className="app-shell__brand">
          <Database size={20} aria-hidden="true" />
          <div>
            <strong>非标选型</strong>
            <span>离线工作台</span>
          </div>
          <button
            className="sidebar-toggle"
            type="button"
            aria-label={isSidebarCollapsed ? "展开左侧导航" : "折叠左侧导航"}
            title={isSidebarCollapsed ? "展开左侧导航" : "折叠左侧导航"}
            onClick={() => setIsSidebarCollapsed((value) => !value)}
          >
            {isSidebarCollapsed ? (
              <PanelLeftOpen size={16} aria-hidden="true" />
            ) : (
              <PanelLeftClose size={16} aria-hidden="true" />
            )}
          </button>
        </div>

        <div className="app-shell__sidebar-scroll">
          <section className="sidebar-section" aria-label="页面导航">
            <div className="sidebar-section__header">页面</div>
            <div className="app-shell__nav-list">
              {APP_ROUTES.map((route) => (
                <RouteButton
                  key={route.id}
                  route={route}
                  isActive={route.id === activeRoute}
                  onClick={() => onRouteChange(route.id)}
                />
              ))}
            </div>
          </section>

          <nav className="sidebar-section" aria-label="PDF 章节导航">
            <div className="sidebar-section__header">
              <span>PDF 23 章</span>
              <small>{filteredChapters.length}/{chapterItems.length}</small>
            </div>
            <div className="chapter-nav-list">
              {filteredChapters.map((chapter, index) => (
                <button
                  className={`chapter-button${
                    chapter.id === activeChapterId ? " chapter-button--active" : ""
                  }`}
                  key={chapter.id}
                  type="button"
                  aria-current={chapter.id === activeChapterId ? "location" : undefined}
                  aria-label={`打开${chapter.chapter}章节入口`}
                  onClick={() => onChapterOpen(chapter.id)}
                  title={chapter.requirement}
                >
                  <span className="chapter-button__index">{index + 1}</span>
                  <span className="chapter-button__text">{chapter.chapter}</span>
                </button>
              ))}
              {filteredChapters.length === 0 ? (
                <p className="chapter-nav-list__empty">没有匹配章节</p>
              ) : null}
            </div>
          </nav>
        </div>
      </nav>

      <div className="app-shell__body">
        <header className="app-shell__topbar">
          <label className="search-box">
            <Search size={16} aria-hidden="true" />
            <span className="sr-only">全局搜索</span>
            <input
              aria-label="全局搜索"
              placeholder="搜索 PDF 章节"
              ref={searchInputRef}
              value={searchQuery}
              onChange={(event) => onSearchQueryChange(event.target.value)}
            />
          </label>
          <div className="app-shell__status">
            {globalErrorMessage ? (
              <div className="app-shell__alert" role="alert" title={globalErrorMessage}>
                <AlertTriangle size={14} aria-hidden="true" />
                数据库检查失败
              </div>
            ) : null}
            <RiskBadge tone={health?.status === "ok" ? "success" : "warning"}>
              {health?.status === "ok" ? "本地数据正常" : "本地数据待检查"}
            </RiskBadge>
          </div>
        </header>

        <main className="app-shell__content">{children}</main>
      </div>

      <TracePanel
        health={health}
        isCollapsed={isTraceCollapsed}
        isLoading={isHealthLoading}
        onToggleCollapsed={() => setIsTraceCollapsed((value) => !value)}
      />
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
      <small>{route.priority}</small>
    </button>
  );
}
