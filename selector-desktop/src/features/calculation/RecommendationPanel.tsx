import { useEffect, useState } from "react";
import { AlertTriangle, CheckCircle2, RefreshCw, SlidersHorizontal } from "lucide-react";
import type { CalculationResult } from "../../domain/calculation";
import type { MatchRuleResult, RecommendationCandidate } from "../../domain/vendor";
import { recommendVendorModels } from "../../shared/api/vendor";
import { VendorImportPanel } from "./VendorImportPanel";
import { VendorLibraryPanel } from "./VendorLibraryPanel";
import "./recommendation-panel.css";

type MatchStatus = "idle" | "loading" | "success" | "empty" | "error";

interface RecommendationPanelProps {
  result: CalculationResult;
  candidates: RecommendationCandidate[];
  selectedModelName: string | null;
  onCandidatesChange: (candidates: RecommendationCandidate[]) => void;
  onSelectedModelNameChange: (modelName: string | null) => void;
}

export function RecommendationPanel({
  result,
  candidates,
  selectedModelName,
  onCandidatesChange,
  onSelectedModelNameChange,
}: RecommendationPanelProps) {
  const [status, setStatus] = useState<MatchStatus>("idle");
  const [message, setMessage] = useState("使用已启用样本库按计算结果匹配候选型号。");
  const [libraryRefreshKey, setLibraryRefreshKey] = useState(0);

  useEffect(() => {
    setStatus("idle");
    setMessage("使用已启用样本库按计算结果匹配候选型号。");
  }, [result.moduleId, result.formulaVersion, result.summary]);

  function resetRecommendations() {
    onCandidatesChange([]);
    onSelectedModelNameChange(null);
    setStatus("idle");
    setMessage("样本库已更新，请重新匹配型号。");
  }

  function handleImported() {
    setLibraryRefreshKey((current) => current + 1);
    resetRecommendations();
  }

  async function handleRecommend() {
    setStatus("loading");
    setMessage("正在匹配型号");
    try {
      const nextCandidates = await recommendVendorModels({
        moduleId: result.moduleId,
        componentType: null,
        requirements: result.requirements,
        limit: 5,
      });
      onCandidatesChange(nextCandidates);
      const firstSelectable =
        nextCandidates.find((candidate) => candidate.failedRules.length === 0)?.model.modelName ??
        null;
      onSelectedModelNameChange(firstSelectable);
      if (nextCandidates.length === 0) {
        setStatus("empty");
        setMessage("没有找到可用于匹配的型号，请先导入并启用样本库。");
        return;
      }
      setStatus("success");
      setMessage(`已返回 ${nextCandidates.length} 个型号，请核对满足条件和不足项。`);
    } catch (error: unknown) {
      onCandidatesChange([]);
      onSelectedModelNameChange(null);
      setStatus("error");
      setMessage(error instanceof Error ? error.message : String(error));
    }
  }

  return (
    <section className="recommendation-panel" aria-label="型号推荐">
      <header>
        <div>
          <h3>型号推荐</h3>
          <p>按本次计算需求参数筛选已启用样本库。</p>
        </div>
        <button
          className="primary-button recommendation-panel__action"
          type="button"
          disabled={status === "loading"}
          onClick={() => void handleRecommend()}
        >
          {status === "loading" ? (
            <RefreshCw size={16} aria-hidden="true" />
          ) : (
            <SlidersHorizontal size={16} aria-hidden="true" />
          )}
          {candidates.length > 0 ? "重新匹配" : "匹配型号"}
        </button>
      </header>

      <span className={`recommendation-panel__status recommendation-panel__status--${status}`} role="status">
        {message}
      </span>

      <VendorImportPanel result={result} onImported={handleImported} />
      <VendorLibraryPanel refreshKey={libraryRefreshKey} onChanged={resetRecommendations} />

      {candidates.length > 0 ? (
        <div className="recommendation-panel__list">
          {candidates.map((candidate) => (
            <article className="recommendation-card" key={candidate.model.id}>
              <div className="recommendation-card__top">
                <div>
                  <strong>{candidate.model.modelName}</strong>
                  <span>
                    {candidate.model.libraryName} · 匹配 {Math.round(candidate.score * 100)}%
                  </span>
                </div>
                <button
                  className="recommendation-card__select"
                  type="button"
                  aria-pressed={selectedModelName === candidate.model.modelName}
                  onClick={() => onSelectedModelNameChange(candidate.model.modelName)}
                >
                  <CheckCircle2 size={15} aria-hidden="true" />
                  {selectedModelName === candidate.model.modelName ? "已选" : "设为最终型号"}
                </button>
              </div>
              <RuleList title="满足条件" rules={candidate.matchedRules} tone="ok" />
              <RuleList title="不足或缺失" rules={candidate.failedRules} tone="warn" />
            </article>
          ))}
        </div>
      ) : null}
    </section>
  );
}

function RuleList({
  title,
  rules,
  tone,
}: {
  title: string;
  rules: MatchRuleResult[];
  tone: "ok" | "warn";
}) {
  if (rules.length === 0) {
    return null;
  }
  return (
    <div className={`recommendation-card__rules recommendation-card__rules--${tone}`}>
      <span>
        {tone === "warn" ? (
          <AlertTriangle size={14} aria-hidden="true" />
        ) : (
          <CheckCircle2 size={14} aria-hidden="true" />
        )}
        {title}
      </span>
      {rules.slice(0, 4).map((rule) => (
        <p key={`${rule.requirementId}-${rule.message}`}>{rule.message}</p>
      ))}
    </div>
  );
}
