export interface ModuleDefinition {
  id: string;
  name: string;
  category: string;
  description: string;
  sourceChapter: string;
  sourcePage: string;
  fields: ModuleField[];
}

export interface ModuleField {
  id: string;
  label: string;
  unit: string;
  unitOptions: string[];
  required: boolean;
  min: number | null;
  defaultValue: number | null;
  helper: string;
  source: string;
}

export interface CalculationRequest {
  moduleId: string;
  fields: FieldInput[];
  safetyFactor: number | null;
  safetyFactorConfirmed: boolean;
}

export interface FieldInput {
  id: string;
  value: number;
  unit: string;
}

export interface CalculationResult {
  moduleId: string;
  moduleName: string;
  formulaVersion: string;
  summary: string;
  conclusion: string;
  steps: FormulaStep[];
  rules: RuleDecision[];
  risks: RiskItem[];
  requirements: RequirementParameter[];
  sourcePages: string[];
  inputSnapshot: unknown;
  defaultsSnapshot: unknown;
}

export interface FormulaStep {
  label: string;
  formula: string;
  substitution: string;
  result: string;
  unit: string;
  source: string;
}

export interface RuleDecision {
  id: string;
  label: string;
  recommendation: string;
  basis: string;
  risk: string;
  source: string;
}

export interface RiskItem {
  level: "success" | "info" | "warning" | "danger";
  message: string;
  fieldId: string | null;
  source: string;
}

export interface RequirementParameter {
  id: string;
  label: string;
  value: number;
  unit: string;
}

export interface FieldError {
  fieldId: string;
  message: string;
}

export interface SaveCaseRequest {
  name: string;
  notes: string;
  request: CalculationRequest;
}

export interface CaseFilter {
  query?: string;
  moduleId?: string;
  createdFrom?: string;
  createdTo?: string;
}

export interface UpdateCaseRequest {
  id: string;
  name: string;
  notes: string;
}

export interface CaseRecord {
  id: string;
  name: string;
  moduleId: string;
  moduleName: string;
  notes: string;
  resultSummary: string;
  riskCount: number;
  createdAt: string;
  updatedAt: string;
}

export interface CaseRunRecord {
  caseRecord: CaseRecord;
  result: CalculationResult;
}

export interface CaseDetailRecord {
  caseRecord: CaseRecord;
  request: CalculationRequest;
  result: CalculationResult;
}

export interface CalculationCaseDraft {
  caseId: string;
  name: string;
  notes: string;
  request: CalculationRequest;
}
