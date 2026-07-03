import { useEffect, useRef } from "react";
import type { ModuleDefinition } from "../../domain/calculation";

interface CalculationFormPageProps {
  module: ModuleDefinition | null;
  values: Record<string, string>;
  units: Record<string, string>;
  safetyFactor: string;
  safetyFactorConfirmed: boolean;
  fieldError: { fieldId: string; message: string } | null;
  isBusy: boolean;
  onValueChange: (fieldId: string, value: string) => void;
  onUnitChange: (fieldId: string, unit: string) => void;
  onSafetyFactorChange: (value: string) => void;
  onSafetyFactorConfirmedChange: (confirmed: boolean) => void;
  onCalculate: () => void;
}

export function CalculationFormPage({
  module,
  values,
  units,
  safetyFactor,
  safetyFactorConfirmed,
  fieldError,
  isBusy,
  onValueChange,
  onUnitChange,
  onSafetyFactorChange,
  onSafetyFactorConfirmedChange,
  onCalculate,
}: CalculationFormPageProps) {
  const safetyFactorInputRef = useRef<HTMLInputElement>(null);
  const fieldInputRefs = useRef<Record<string, HTMLInputElement | null>>({});

  useEffect(() => {
    if (fieldError?.fieldId === "safetyFactor") {
      safetyFactorInputRef.current?.focus();
      return;
    }
    if (fieldError?.fieldId) {
      fieldInputRefs.current[fieldError.fieldId]?.focus();
    }
  }, [fieldError]);

  if (!module) {
    return <section className="calculation-form">请选择一个计算模块。</section>;
  }

  const hasFormula = module.fields.length > 0;

  return (
    <section className="calculation-form" aria-labelledby="calculation-form-title">
      <div>
        <h2 id="calculation-form-title">{module.name}</h2>
        <p>{module.description}</p>
      </div>

      {hasFormula ? (
        <div className="field-grid">
          {module.fields.map((field) => (
            <label className="unit-field" key={field.id}>
              <span>
                {field.label}
                <small>{field.source}</small>
              </span>
              <div>
                <input
                  aria-label={field.label}
                  aria-invalid={fieldError?.fieldId === field.id}
                  type="number"
                  step="any"
                  ref={(element) => {
                    fieldInputRefs.current[field.id] = element;
                  }}
                  value={values[field.id] ?? ""}
                  onChange={(event) => onValueChange(field.id, event.target.value)}
                />
                <select
                  aria-label={`${field.label}单位`}
                  value={units[field.id] ?? field.unit}
                  onChange={(event) => onUnitChange(field.id, event.target.value)}
                >
                  {field.unitOptions.map((unit) => (
                    <option key={unit} value={unit}>
                      {unit}
                    </option>
                  ))}
                </select>
              </div>
              <em>{field.helper}</em>
              {fieldError?.fieldId === field.id ? <b>{fieldError.message}</b> : null}
            </label>
          ))}
          <label className="unit-field unit-field--safety">
            <span>
              安全系数
              <small>必须手动输入或确认</small>
            </span>
            <div>
              <input
                aria-label="安全系数"
                aria-invalid={fieldError?.fieldId === "safetyFactor"}
                ref={safetyFactorInputRef}
                type="number"
                step="any"
                value={safetyFactor}
                onChange={(event) => onSafetyFactorChange(event.target.value)}
              />
              <strong>K</strong>
            </div>
            <label className="confirm-line">
              <input
                type="checkbox"
                checked={safetyFactorConfirmed}
                onChange={(event) => onSafetyFactorConfirmedChange(event.target.checked)}
              />
              <span>我已确认安全系数</span>
            </label>
            {fieldError?.fieldId === "safetyFactor" ? <b>{fieldError.message}</b> : null}
          </label>
        </div>
      ) : (
        <p className="calculation-form__empty">该模块已列入规划，公式在对应章节包中实现。</p>
      )}

      <button
        className="secondary-button"
        type="button"
        disabled={!hasFormula || isBusy}
        onClick={onCalculate}
      >
        {isBusy ? "计算中" : "计算"}
      </button>
      {!safetyFactorConfirmed && hasFormula ? (
        <p className="calculation-form__warning">确认安全系数后才能计算。</p>
      ) : null}
    </section>
  );
}
