import { useEffect, useState } from "react";
import { AlertTriangle, CheckCircle2, FileText } from "lucide-react";
import type { CalculationRequest, CalculationResult } from "../../domain/calculation";
import type { RecommendationCandidate } from "../../domain/vendor";
import { ReportExportDialog } from "../reports/ReportExportDialog";
import { RecommendationPanel } from "./RecommendationPanel";

interface CalculationResultPanelProps {
  result: CalculationResult | null;
  request: CalculationRequest | null;
  caseName: string;
  notes: string;
}

export function CalculationResultPanel({
  result,
  request,
  caseName,
  notes,
}: CalculationResultPanelProps) {
  const [candidates, setCandidates] = useState<RecommendationCandidate[]>([]);
  const [selectedModelName, setSelectedModelName] = useState<string | null>(null);
  const resultKey = result ? `${result.moduleId}:${result.formulaVersion}:${result.summary}` : "";

  useEffect(() => {
    setCandidates([]);
    setSelectedModelName(null);
  }, [resultKey]);

  if (!result) {
    return (
      <section className="result-panel result-panel--empty">
        <div className="result-panel__empty-icon">
          <FileText size={22} aria-hidden="true" />
        </div>
        <h2>结果与过程</h2>
        <p>完成计算后，这里会显示结论、需求参数、公式代入、风险提示和报告导出。</p>
      </section>
    );
  }

  return (
    <section className="result-panel" aria-label="计算结果与过程分析">
      <header>
        <div>
          <h2>{result.moduleName}</h2>
          <p>计算结果</p>
        </div>
        <span className="result-panel__state">
          <CheckCircle2 size={16} aria-hidden="true" />
          已完成
        </span>
      </header>

      <strong className="result-panel__summary">{result.summary}</strong>
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
        result={result}
        candidates={candidates}
        selectedModelName={selectedModelName}
        onCandidatesChange={setCandidates}
        onSelectedModelNameChange={setSelectedModelName}
      />

      <ReportExportDialog
        caseName={caseName}
        notes={notes}
        request={request}
        result={result}
        candidates={candidates}
        finalModelName={selectedModelName}
      />

      <div className="result-panel__split">
        <div className="formula-steps">
          <div className="result-panel__section-title">
            <FileText size={15} aria-hidden="true" />
            <h3>公式过程</h3>
          </div>
          {result.steps.map((step) => (
            <article key={step.label}>
              <h3>{step.label}</h3>
              <code>{step.formula}</code>
              <p>{step.substitution}</p>
              <strong>
                {step.result} {step.unit}
              </strong>
            </article>
          ))}
        </div>
        <aside className="risk-list">
          <div className="result-panel__section-title">
            <AlertTriangle size={15} aria-hidden="true" />
            <h3>风险与判断</h3>
          </div>
          {result.rules.map((rule) => (
            <div className={`rule-item rule-item--${rule.risk}`} key={rule.id}>
              <strong>{rule.label}</strong>
              <p>{rule.recommendation}</p>
              <span>{rule.basis}</span>
            </div>
          ))}
          {result.risks.map((risk) => (
            <div className={`risk-item risk-item--${risk.level}`} key={risk.message}>
              <strong>{riskLevelText(risk.level)}</strong>
              <p>{risk.message}</p>
            </div>
          ))}
        </aside>
      </div>
    </section>
  );
}

function riskLevelText(level: string): string {
  switch (level) {
    case "success":
      return "通过";
    case "warning":
      return "注意";
    case "danger":
      return "风险";
    default:
      return "提示";
  }
}
