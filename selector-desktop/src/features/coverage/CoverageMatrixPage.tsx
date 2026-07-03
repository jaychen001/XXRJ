import { useEffect, useMemo, useRef } from "react";
import { ArrowRight, RefreshCw } from "lucide-react";
import { COVERAGE_ITEMS, type CoverageStatus } from "../../domain/coverage";
import { RiskBadge, type RiskTone } from "../../shared/ui/RiskBadge";
import "./coverage-matrix-page.css";

const STATUS_LABELS = {
  planned: "待实现",
  partial: "部分实现",
  done: "已实现",
  blocked: "阻塞",
} satisfies Record<CoverageStatus, string>;

const STATUS_TONES = {
  planned: "neutral",
  partial: "warning",
  done: "success",
  blocked: "danger",
} satisfies Record<CoverageStatus, RiskTone>;

interface CoverageMatrixPageProps {
  searchQuery: string;
  selectedItemId: string;
  lastRefreshedAt: Date;
  onOpenItem: (itemId: string) => void;
  onRefresh: () => void;
}

export function CoverageMatrixPage({
  searchQuery,
  selectedItemId,
  lastRefreshedAt,
  onOpenItem,
  onRefresh,
}: CoverageMatrixPageProps) {
  const rowRefs = useRef(new Map<string, HTMLTableRowElement>());
  const total = COVERAGE_ITEMS.length;
  const completed = COVERAGE_ITEMS.filter((item) => item.status === "done").length;
  const filteredItems = useMemo(() => {
    const normalizedQuery = searchQuery.trim().toLowerCase();
    if (!normalizedQuery) {
      return COVERAGE_ITEMS;
    }

    return COVERAGE_ITEMS.filter((item) =>
      `${item.chapter} ${item.requirement} ${item.shape} ${item.source}`
        .toLowerCase()
        .includes(normalizedQuery),
    );
  }, [searchQuery]);
  const selectedItem =
    COVERAGE_ITEMS.find((item) => item.id === selectedItemId) ?? COVERAGE_ITEMS[0];

  useEffect(() => {
    rowRefs.current.get(selectedItem.id)?.scrollIntoView({ block: "nearest" });
  }, [selectedItem.id]);

  return (
    <section className="coverage-page" aria-labelledby="coverage-title">
      <div className="coverage-page__header">
        <div>
          <h1 className="page-title" id="coverage-title">
            工作台首页 / PDF 覆盖矩阵
          </h1>
          <p className="page-subtitle">
            第一版必须覆盖根目录 PDF 的 23 个章节。当前为 Phase 1 骨架，后续阶段逐章落计算和规则。
          </p>
        </div>
        <button className="secondary-button" type="button" onClick={onRefresh}>
          <RefreshCw size={16} aria-hidden="true" />
          刷新覆盖矩阵
        </button>
      </div>

      <div className="coverage-summary" aria-label="覆盖摘要">
        <SummaryMetric label="章节总数" value={String(total)} />
        <SummaryMetric label="已实现" value={String(completed)} />
        <SummaryMetric label="当前阶段" value="Phase 1" />
        <SummaryMetric label="最近刷新" value={formatRefreshTime(lastRefreshedAt)} />
      </div>

      <div className="coverage-table-wrap">
        <table className="coverage-table" aria-label="PDF 覆盖矩阵">
          <thead>
            <tr>
              <th>PDF 章节</th>
              <th>实现形态</th>
              <th>来源</th>
              <th>入口</th>
              <th>状态</th>
              <th aria-label="操作" />
            </tr>
          </thead>
          <tbody>
            {filteredItems.map((item) => (
              <tr
                className={item.id === selectedItem.id ? "coverage-table__row--active" : ""}
                key={item.id}
                ref={(node) => {
                  if (node) {
                    rowRefs.current.set(item.id, node);
                  } else {
                    rowRefs.current.delete(item.id);
                  }
                }}
              >
                <td>
                  <strong>{item.chapter}</strong>
                  <span>{item.requirement}</span>
                </td>
                <td>{item.shape}</td>
                <td>{item.source}</td>
                <td>{item.entryCount}</td>
                <td>
                  <RiskBadge tone={STATUS_TONES[item.status]}>
                    {STATUS_LABELS[item.status]}
                  </RiskBadge>
                </td>
                <td>
                  <button
                    className="icon-button"
                    type="button"
                    aria-label={`打开${item.chapter}章节入口`}
                    title={`打开${item.chapter}`}
                    onClick={() => onOpenItem(item.id)}
                  >
                    <ArrowRight size={16} aria-hidden="true" />
                  </button>
                </td>
              </tr>
            ))}
            {filteredItems.length === 0 ? (
              <tr>
                <td colSpan={6}>没有匹配的 PDF 章节</td>
              </tr>
            ) : null}
          </tbody>
        </table>
      </div>

      <section className="chapter-detail" aria-labelledby="chapter-detail-title">
        <div>
          <h2 id="chapter-detail-title">{selectedItem.chapter} · 章节入口</h2>
          <p>{selectedItem.requirement}</p>
        </div>
        <dl>
          <div>
            <dt>实现形态</dt>
            <dd>{selectedItem.shape}</dd>
          </div>
          <div>
            <dt>来源</dt>
            <dd>{selectedItem.source}</dd>
          </div>
          <div>
            <dt>当前状态</dt>
            <dd>{STATUS_LABELS[selectedItem.status]}</dd>
          </div>
        </dl>
        <p className="chapter-detail__note">
          Phase 1 提供章节入口和覆盖追踪；公式、规则、来源页码和回归样例按后续阶段逐项落库。
        </p>
      </section>
    </section>
  );
}

interface SummaryMetricProps {
  label: string;
  value: string;
}

function SummaryMetric({ label, value }: SummaryMetricProps) {
  return (
    <div className="summary-metric">
      <span>{label}</span>
      <strong>{value}</strong>
    </div>
  );
}

function formatRefreshTime(value: Date): string {
  return value.toLocaleTimeString("zh-CN", {
    hour: "2-digit",
    minute: "2-digit",
    second: "2-digit",
  });
}
