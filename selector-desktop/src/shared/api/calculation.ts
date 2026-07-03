import { invoke } from "@tauri-apps/api/core";
import type {
  CalculationRequest,
  CalculationResult,
  CaseDetailRecord,
  CaseFilter,
  CaseRecord,
  CaseRunRecord,
  FieldError,
  ModuleDefinition,
  SaveCaseRequest,
  UpdateCaseRequest,
} from "../../domain/calculation";

export function listCalculationModules(): Promise<ModuleDefinition[]> {
  return invoke<ModuleDefinition[]>("list_calculation_modules");
}

export function runCalculation(request: CalculationRequest): Promise<CalculationResult> {
  return invoke<CalculationResult>("run_calculation", { request });
}

export function saveCalculationCase(payload: SaveCaseRequest): Promise<CaseRunRecord> {
  return invoke<CaseRunRecord>("save_calculation_case", { payload });
}

export function listCalculationCases(filter?: CaseFilter): Promise<CaseRecord[]> {
  return invoke<CaseRecord[]>("list_calculation_cases", { filter: filter ?? null });
}

export function updateCalculationCase(payload: UpdateCaseRequest): Promise<CaseRecord> {
  return invoke<CaseRecord>("update_calculation_case", { payload });
}

export function duplicateCalculationCase(id: string): Promise<CaseRecord> {
  return invoke<CaseRecord>("duplicate_calculation_case", { id });
}

export function getCalculationCaseDetail(id: string): Promise<CaseDetailRecord> {
  return invoke<CaseDetailRecord>("get_calculation_case_detail", { id });
}

export function rerunCalculationCase(id: string): Promise<CaseRunRecord> {
  return invoke<CaseRunRecord>("rerun_calculation_case", { id });
}

export function rerunCalculationCaseWithRequest(
  id: string,
  request: CalculationRequest,
): Promise<CaseRunRecord> {
  return invoke<CaseRunRecord>("rerun_calculation_case_with_request", { id, request });
}

export function deleteCalculationCase(id: string): Promise<boolean> {
  return invoke<boolean>("delete_calculation_case", { id });
}

export function isFieldError(error: unknown): error is FieldError {
  return (
    typeof error === "object" &&
    error !== null &&
    "fieldId" in error &&
    "message" in error
  );
}
