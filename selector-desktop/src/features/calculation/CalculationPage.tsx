import { useEffect, useMemo, useState } from "react";
import type {
  CalculationCaseDraft,
  CalculationRequest,
  CalculationResult,
  FieldError,
  ModuleDefinition,
} from "../../domain/calculation";
import {
  isFieldError,
  listCalculationModules,
  rerunCalculationCaseWithRequest,
  runCalculation,
  saveCalculationCase,
  updateCalculationCase,
} from "../../shared/api/calculation";
import { CalculationFormPage } from "./CalculationFormPage";
import { CalculationResultPanel } from "./CalculationResultPanel";
import { ModuleListPage } from "./ModuleListPage";
import "./calculation-page.css";

interface CalculationPageProps {
  draft: CalculationCaseDraft | null;
}

export function CalculationPage({ draft }: CalculationPageProps) {
  const [modules, setModules] = useState<ModuleDefinition[]>([]);
  const [selectedModuleId, setSelectedModuleId] = useState("");
  const [moduleSearch, setModuleSearch] = useState("");
  const [values, setValues] = useState<Record<string, string>>({});
  const [units, setUnits] = useState<Record<string, string>>({});
  const [safetyFactor, setSafetyFactor] = useState("1.5");
  const [safetyFactorConfirmed, setSafetyFactorConfirmed] = useState(false);
  const [fieldError, setFieldError] = useState<FieldError | null>(null);
  const [result, setResult] = useState<CalculationResult | null>(null);
  const [lastRequest, setLastRequest] = useState<CalculationRequest | null>(null);
  const [activeCaseId, setActiveCaseId] = useState("");
  const [loadedDraftId, setLoadedDraftId] = useState("");
  const [caseName, setCaseName] = useState("同步带计算案例");
  const [notes, setNotes] = useState("");
  const [status, setStatus] = useState("读取模块中");
  const [saveStatus, setSaveStatus] = useState("未保存");
  const [isBusy, setIsBusy] = useState(false);
  const selectedModule = useMemo(
    () => modules.find((module) => module.id === selectedModuleId) ?? null,
    [modules, selectedModuleId],
  );

  useEffect(() => {
    void loadModules();
  }, []);

  useEffect(() => {
    if (!draft || draft.caseId === loadedDraftId) {
      return;
    }
    const module = modules.find((item) => item.id === draft.request.moduleId);
    if (!module) {
      return;
    }
    setSelectedModuleId(module.id);
    setValues(valuesFromRequest(module, draft.request));
    setUnits(unitsFromRequest(module, draft.request));
    setSafetyFactor(String(draft.request.safetyFactor ?? ""));
    setSafetyFactorConfirmed(draft.request.safetyFactorConfirmed);
    setCaseName(draft.name);
    setNotes(draft.notes);
    setActiveCaseId(draft.caseId);
    setLastRequest(null);
    setResult(null);
    setFieldError(null);
    setSaveStatus("副本参数已载入");
    setStatus(`正在修改：${draft.name}`);
    setLoadedDraftId(draft.caseId);
  }, [draft, loadedDraftId, modules]);

  async function loadModules() {
    try {
      const records = await listCalculationModules();
      setModules(records);
      const firstRunnable = records.find((module) => module.fields.length > 0) ?? records[0];
      if (firstRunnable) {
        setSelectedModuleId(firstRunnable.id);
        setValues(defaultValues(firstRunnable));
        setUnits(defaultUnits(firstRunnable));
      }
      setStatus(`模块 ${records.length} 个`);
    } catch (error: unknown) {
      setStatus(toErrorMessage(error));
    }
  }

  function handleSelectModule(moduleId: string) {
    const nextModule = modules.find((module) => module.id === moduleId);
    setSelectedModuleId(moduleId);
    setValues(nextModule ? defaultValues(nextModule) : {});
    setUnits(nextModule ? defaultUnits(nextModule) : {});
    setActiveCaseId("");
    setFieldError(null);
    setResult(null);
  }

  async function handleCalculate() {
    if (!selectedModule) {
      return;
    }
    const requiredError = validateRequiredFields(selectedModule, values);
    if (requiredError) {
      setFieldError(requiredError);
      setStatus(requiredError.message);
      return;
    }
    if (!safetyFactorConfirmed) {
      const nextError = {
        fieldId: "safetyFactor",
        message: "安全系数未确认，请输入或确认本次计算使用值",
      };
      setFieldError(nextError);
      setStatus(nextError.message);
      return;
    }
    const request = buildRequest(
      selectedModule,
      values,
      units,
      safetyFactor,
      safetyFactorConfirmed,
    );
    setIsBusy(true);
    setFieldError(null);
    setStatus("计算中");
    try {
      const nextResult = await runCalculation(request);
      setLastRequest(request);
      setResult(nextResult);
      setStatus("计算完成");
      setSaveStatus("未保存");
    } catch (error: unknown) {
      const nextError = isFieldError(error)
        ? error
        : { fieldId: "form", message: toErrorMessage(error) };
      setFieldError(nextError);
      setStatus(nextError.message);
    } finally {
      setIsBusy(false);
    }
  }

  async function handleSaveCase() {
    if (!lastRequest) {
      return;
    }
    setSaveStatus("保存中");
    try {
      if (activeCaseId) {
        await updateCalculationCase({ id: activeCaseId, name: caseName, notes });
        await rerunCalculationCaseWithRequest(activeCaseId, lastRequest);
        setSaveStatus("当前案例已更新");
        return;
      }
      await saveCalculationCase({ name: caseName, notes, request: lastRequest });
      setSaveStatus("案例已保存");
    } catch (error: unknown) {
      setSaveStatus(toErrorMessage(error));
    }
  }

  return (
    <section className="calculation-page" aria-labelledby="calculation-title">
      <div className="calculation-page__header">
        <div>
          <h1 className="page-title" id="calculation-title">
            选型计算
          </h1>
          <p className="page-subtitle">
            通用表单、公式步骤、风险提示和安全系数手动确认。
          </p>
        </div>
        <span className="calculation-status" role="status">
          {status}
        </span>
      </div>
      <div className="calculation-workbench">
        <ModuleListPage
          modules={modules}
          selectedModuleId={selectedModuleId}
          searchQuery={moduleSearch}
          onSearchQueryChange={setModuleSearch}
          onSelectModule={handleSelectModule}
        />
        <CalculationFormPage
          module={selectedModule}
          values={values}
          units={units}
          safetyFactor={safetyFactor}
          safetyFactorConfirmed={safetyFactorConfirmed}
          fieldError={fieldError}
          isBusy={isBusy}
          onValueChange={(fieldId, value) =>
            setValues((current) => ({ ...current, [fieldId]: value }))
          }
          onUnitChange={(fieldId, unit) =>
            setUnits((current) => ({ ...current, [fieldId]: unit }))
          }
          onSafetyFactorChange={setSafetyFactor}
          onSafetyFactorConfirmedChange={setSafetyFactorConfirmed}
          onCalculate={() => void handleCalculate()}
        />
      </div>
      <CalculationResultPanel
        result={result}
        request={lastRequest}
        caseName={caseName}
        notes={notes}
        saveStatus={saveStatus}
        onCaseNameChange={setCaseName}
        onNotesChange={setNotes}
        onSaveCase={() => void handleSaveCase()}
      />
    </section>
  );
}

function defaultValues(module: ModuleDefinition): Record<string, string> {
  return Object.fromEntries(
    module.fields.map((field) => [field.id, String(field.defaultValue ?? "")]),
  );
}

function defaultUnits(module: ModuleDefinition): Record<string, string> {
  return Object.fromEntries(module.fields.map((field) => [field.id, field.unit]));
}

function valuesFromRequest(
  module: ModuleDefinition,
  request: CalculationRequest,
): Record<string, string> {
  const requestValues = new Map(request.fields.map((field) => [field.id, String(field.value)]));
  return Object.fromEntries(
    module.fields.map((field) => [field.id, requestValues.get(field.id) ?? ""]),
  );
}

function unitsFromRequest(module: ModuleDefinition, request: CalculationRequest): Record<string, string> {
  const requestUnits = new Map(request.fields.map((field) => [field.id, field.unit]));
  return Object.fromEntries(
    module.fields.map((field) => [field.id, requestUnits.get(field.id) ?? field.unit]),
  );
}

function validateRequiredFields(
  module: ModuleDefinition,
  values: Record<string, string>,
): FieldError | null {
  const emptyField = module.fields.find((field) => field.required && !values[field.id]?.trim());
  if (!emptyField) {
    return null;
  }
  return {
    fieldId: emptyField.id,
    message: `${emptyField.label}不能为空，请输入数值后再计算`,
  };
}

function buildRequest(
  module: ModuleDefinition,
  values: Record<string, string>,
  units: Record<string, string>,
  safetyFactor: string,
  safetyFactorConfirmed: boolean,
): CalculationRequest {
  return {
    moduleId: module.id,
    fields: module.fields.map((field) => ({
      id: field.id,
      value: Number(values[field.id] ?? 0),
      unit: units[field.id] ?? field.unit,
    })),
    safetyFactor: safetyFactor.trim() ? Number(safetyFactor) : null,
    safetyFactorConfirmed,
  };
}

function toErrorMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}
