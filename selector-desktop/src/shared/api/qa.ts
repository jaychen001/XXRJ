import { invoke } from "@tauri-apps/api/core";
import type { QaCoverageAudit, QaRegressionReport } from "../../domain/qa";

export function getQaCoverageAudit(): Promise<QaCoverageAudit> {
  return invoke<QaCoverageAudit>("get_qa_coverage_audit");
}

export function runQaRegression(): Promise<QaRegressionReport> {
  return invoke<QaRegressionReport>("run_qa_regression");
}
