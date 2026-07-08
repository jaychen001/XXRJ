import { useEffect, useMemo, useRef, useState } from "react";
import { ArrowRight, DatabaseZap } from "lucide-react";
import type { CoverageItem, CoverageStatus } from "../../domain/coverage";
import { DriveModulePage } from "../modules/DriveModulePage";
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
  items: CoverageItem[];
  searchQuery: string;
  selectedItemId: string;
  lastRefreshedAt: Date;
  onOpenItem: (itemId: string) => void;
  onIngestRootPdf: () => Promise<unknown>;
}

export function CoverageMatrixPage({
  items,
  searchQuery,
  selectedItemId,
  lastRefreshedAt,
  onOpenItem,
  onIngestRootPdf,
}: CoverageMatrixPageProps) {
  const rowRefs = useRef(new Map<string, HTMLTableRowElement>());
  const [isIngesting, setIsIngesting] = useState(false);
  const total = items.length;
  const completed = items.filter((item) => item.status === "done").length;
  const filteredItems = useMemo(() => {
    const normalizedQuery = searchQuery.trim().toLowerCase();
    if (!normalizedQuery) {
      return items;
    }

    return items.filter((item) =>
      `${item.chapter} ${item.requirement} ${item.shape} ${item.source}`
        .toLowerCase()
        .includes(normalizedQuery),
    );
  }, [items, searchQuery]);
  const selectedItem = items.find((item) => item.id === selectedItemId) ?? items[0];

  useEffect(() => {
    if (selectedItem) {
      rowRefs.current.get(selectedItem.id)?.scrollIntoView({ block: "nearest" });
    }
  }, [selectedItem]);

  async function handleIngestRootPdf() {
    setIsIngesting(true);
    try {
      await onIngestRootPdf();
    } finally {
      setIsIngesting(false);
    }
  }

  return (
    <section className="coverage-page" aria-labelledby="coverage-title">
      <div className="coverage-page__header">
        <div>
          <h1 className="page-title" id="coverage-title">
            工作台首页 / PDF 覆盖矩阵
          </h1>
          <p className="page-subtitle">
            第一版必须覆盖根目录 PDF 的 23 个章节。当前显示 PDF 索引后的目录、知识引用和来源页码。
          </p>
        </div>
        <button
          className="secondary-button"
          type="button"
          disabled={isIngesting}
          onClick={() => void handleIngestRootPdf()}
        >
          <DatabaseZap size={16} aria-hidden="true" />
          导入/刷新 PDF 索引
        </button>
      </div>

      <div className="coverage-summary" aria-label="覆盖摘要">
        <SummaryMetric label="章节总数" value={String(total)} />
        <SummaryMetric label="已实现" value={String(completed)} />
        <SummaryMetric label="当前阶段" value="Phase 4" />
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

      {selectedItem ? (
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
              <dt>匹配页码</dt>
              <dd>{selectedItem.source}</dd>
            </div>
            <div>
              <dt>目录页</dt>
              <dd>{selectedItem.catalogPage ?? "待索引"}</dd>
            </div>
            <div>
              <dt>知识条目</dt>
              <dd>{selectedItem.entryCount} 条</dd>
            </div>
            <div>
              <dt>当前状态</dt>
              <dd>{STATUS_LABELS[selectedItem.status]}</dd>
            </div>
          </dl>
          <p className="chapter-detail__note">
            {selectedItem.catalogExcerpt || "尚未建立 PDF 索引，刷新覆盖矩阵只会显示规划状态。"}
          </p>
          <DriveModulePage item={selectedItem} />
        </section>
      ) : null}
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
