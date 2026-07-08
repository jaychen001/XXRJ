import { SearchCheck } from "lucide-react";
import { useState } from "react";
import type { RequirementParameter } from "../../domain/calculation";
import type { RecommendationCandidate } from "../../domain/vendor";
import { recommendVendorModels } from "../../shared/api/vendor";
import "./vendor-library-page.css";

interface RecommendationPanelProps {
  moduleId: string;
  componentType: string | null;
  requirements: RequirementParameter[];
  onCandidatesChange?: (candidates: RecommendationCandidate[]) => void;
}

export function RecommendationPanel({
  moduleId,
  componentType,
  requirements,
  onCandidatesChange,
}: RecommendationPanelProps) {
  const [candidates, setCandidates] = useState<RecommendationCandidate[]>([]);
  const [status, setStatus] = useState("尚未匹配厂家型号");
  const [isBusy, setIsBusy] = useState(false);

  async function handleRecommend() {
    setIsBusy(true);
    setStatus("匹配样本库中");
    try {
      const records = await recommendVendorModels({
        moduleId,
        componentType,
        requirements,
        limit: 8,
      });
      setCandidates(records);
      onCandidatesChange?.(records);
      setStatus(records.length > 0 ? `匹配到 ${records.length} 个候选型号` : "没有匹配到候选型号");
    } catch (error: unknown) {
      setStatus(error instanceof Error ? error.message : String(error));
    } finally {
      setIsBusy(false);
    }
  }

  return (
    <section className="recommendation-panel" aria-label="厂家型号推荐">
      <header>
        <div>
          <h3>厂家型号推荐</h3>
          <p>{status}</p>
        </div>
        <button
          className="secondary-button"
          type="button"
          disabled={isBusy || requirements.length === 0}
          onClick={() => void handleRecommend()}
        >
          <SearchCheck size={16} aria-hidden="true" />
          匹配型号
        </button>
      </header>

      {candidates.length > 0 ? (
        <div className="recommendation-list">
          {candidates.map((candidate) => (
            <article key={candidate.model.id}>
              <div>
                <strong>{candidate.model.modelName}</strong>
                <span>
                  {candidate.model.libraryName} / {candidate.model.componentType}
                </span>
              </div>
              <b>{Math.round(candidate.score * 100)}%</b>
              <ul>
                {candidate.matchedRules.map((rule) => (
                  <li key={`match-${rule.requirementId}`}>{rule.message}</li>
                ))}
                {candidate.failedRules.slice(0, 2).map((rule) => (
                  <li key={`fail-${rule.requirementId}`}>{rule.message}</li>
                ))}
              </ul>
            </article>
          ))}
        </div>
      ) : null}
    </section>
  );
}
