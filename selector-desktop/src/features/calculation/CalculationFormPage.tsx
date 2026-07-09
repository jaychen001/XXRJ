import { useEffect, useRef } from "react";
import { Calculator, ShieldCheck } from "lucide-react";
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
    return <section className="calculation-form">请选择一个计算对象。</section>;
  }

  const hasFormula = module.fields.length > 0;

  return (
    <section className="calculation-form" aria-labelledby="calculation-form-title">
      <div className="calculation-form__heading">
        <div>
          <span>{module.category}</span>
          <h2 id="calculation-form-title">{module.name}</h2>
          <p>{module.description}</p>
        </div>
        <strong>{module.fields.length} 项参数</strong>
      </div>

      {fieldError ? (
        <div className="calculation-form__error" role="alert">
          {fieldError.message}
        </div>
      ) : null}

      {hasFormula ? (
        <div className="field-grid">
          {module.fields.map((field) => (
            <label className="unit-field" key={field.id}>
              <span>{field.label}</span>
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
          <div className="unit-field unit-field--safety">
            <span>安全系数</span>
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
              <span>我已确认本次计算使用的安全系数</span>
            </label>
            {fieldError?.fieldId === "safetyFactor" ? <b>{fieldError.message}</b> : null}
          </div>
        </div>
      ) : (
        <p className="calculation-form__empty">这个对象还没有公式，后续会补齐。</p>
      )}

      <div className="calculation-form__actions">
        <span className={safetyFactorConfirmed ? "confirm-state confirm-state--ok" : "confirm-state"}>
          <ShieldCheck size={15} aria-hidden="true" />
          {safetyFactorConfirmed ? "安全系数已确认" : "安全系数待确认"}
        </span>
        <button
          className="primary-button"
          type="button"
          disabled={!hasFormula || isBusy}
          onClick={onCalculate}
        >
          <Calculator size={16} aria-hidden="true" />
          {isBusy ? "计算中" : "计算"}
        </button>
      </div>
      {!safetyFactorConfirmed && hasFormula ? (
        <p className="calculation-form__warning">确认安全系数后才能计算。</p>
      ) : null}
    </section>
  );
}
