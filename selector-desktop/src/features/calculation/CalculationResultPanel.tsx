import { useEffect, useState } from "react";
import type { CalculationRequest, CalculationResult } from "../../domain/calculation";
import type { RecommendationCandidate } from "../../domain/vendor";
import { ReportExportDialog } from "../reports/ReportExportDialog";
import { RecommendationPanel } from "../vendor/RecommendationPanel";

interface CalculationResultPanelProps {
  result: CalculationResult | null;
  request: CalculationRequest | null;
  caseName: string;
  notes: string;
  saveStatus: string;
  onCaseNameChange: (value: string) => void;
  onNotesChange: (value: string) => void;
  onSaveCase: () => void;
}

export function CalculationResultPanel({
  result,
  request,
  caseName,
  notes,
  saveStatus,
  onCaseNameChange,
  onNotesChange,
  onSaveCase,
}: CalculationResultPanelProps) {
  const [recommendationCandidates, setRecommendationCandidates] = useState<
    RecommendationCandidate[]
  >([]);

  useEffect(() => {
    setRecommendationCandidates([]);
  }, [result?.moduleId, result?.summary]);

  if (!result) {
    return (
      <section className="result-panel result-panel--empty">
        <h2>结果与过程</h2>
        <p>完成计算后显示公式、代入值、中间值、结论、风险和来源。</p>
      </section>
    );
  }

  return (
    <section className="result-panel" aria-label="计算结果与过程分析">
      <header>
        <div>
          <h2>{result.moduleName}</h2>
          <p>{result.formulaVersion}</p>
        </div>
        <strong>{result.summary}</strong>
      </header>
      <p className="result-panel__conclusion">{result.conclusion}</p>

      <div className="requirement-grid">
        {result.requirements.map((parameter) => (
          <div key={parameter.id}>
            <span>{parameter.label}</span>
            <strong>
              {parameter.value.toFixed(3)} {parameter.unit}
            </strong>
          </div>
        ))}
      </div>

      <RecommendationPanel
        moduleId={result.moduleId}
        componentType={null}
        requirements={result.requirements}
        onCandidatesChange={setRecommendationCandidates}
      />

      <ReportExportDialog
        caseName={caseName}
        notes={notes}
        request={request}
        result={result}
        candidates={recommendationCandidates}
      />

      <div className="result-panel__split">
        <div className="formula-steps">
          {result.steps.map((step) => (
            <article key={step.label}>
              <h3>{step.label}</h3>
              <code>{step.formula}</code>
              <p>{step.substitution}</p>
              <strong>
                {step.result} {step.unit}
              </strong>
              <span>{step.source}</span>
            </article>
          ))}
        </div>
        <aside className="risk-list">
          {result.rules.map((rule) => (
            <div className={`rule-item rule-item--${rule.risk}`} key={rule.id}>
              <strong>{rule.label}</strong>
              <p>{rule.recommendation}</p>
              <span>{rule.basis}</span>
              <small>{rule.source}</small>
            </div>
          ))}
          {result.risks.map((risk) => (
            <div className={`risk-item risk-item--${risk.level}`} key={risk.message}>
              <strong>{risk.level}</strong>
              <p>{risk.message}</p>
              <span>{risk.source}</span>
            </div>
          ))}
          <div className="source-list">
            <strong>来源</strong>
            {result.sourcePages.map((source) => (
              <span key={source}>{source}</span>
            ))}
          </div>
        </aside>
      </div>

      <div className="save-case-bar">
        <input
          aria-label="案例名称"
          value={caseName}
          onChange={(event) => onCaseNameChange(event.target.value)}
        />
        <input
          aria-label="案例备注"
          value={notes}
          onChange={(event) => onNotesChange(event.target.value)}
        />
        <button
          className="secondary-button"
          type="button"
          disabled={!request}
          onClick={onSaveCase}
        >
          保存案例
        </button>
        <span>{saveStatus}</span>
      </div>
    </section>
  );
}
