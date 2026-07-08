import type { CalculationRequest, CalculationResult } from "../../domain/calculation";
import { ReportExportDialog } from "../reports/ReportExportDialog";

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
  if (!result) {
    return (
      <section className="result-panel result-panel--empty">
        <h2>结果与过程</h2>
        <p>完成计算后，这里会显示公式、代入值、中间值、结论和风险提示。</p>
      </section>
    );
  }

  return (
    <section className="result-panel" aria-label="计算结果与过程分析">
      <header>
        <div>
          <h2>{result.moduleName}</h2>
          <p>计算过程</p>
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

      <ReportExportDialog
        caseName={caseName}
        notes={notes}
        request={request}
        result={result}
        candidates={[]}
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
            </article>
          ))}
        </div>
        <aside className="risk-list">
          {result.rules.map((rule) => (
            <div className={`rule-item rule-item--${rule.risk}`} key={rule.id}>
              <strong>{rule.label}</strong>
              <p>{rule.recommendation}</p>
              <span>{rule.basis}</span>
            </div>
          ))}
          {result.risks.map((risk) => (
            <div className={`risk-item risk-item--${risk.level}`} key={risk.message}>
              <strong>{risk.level}</strong>
              <p>{risk.message}</p>
            </div>
          ))}
        </aside>
      </div>
    </section>
  );
}
