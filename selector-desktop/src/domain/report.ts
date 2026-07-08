import type { CalculationRequest, CalculationResult } from "./calculation";
import type { RecommendationCandidate } from "./vendor";

export interface ExportReportRequest {
  format: string;
  outputPath: string;
  caseId: string | null;
  caseName: string;
  notes: string;
  request: CalculationRequest;
  result: CalculationResult;
  candidates: RecommendationCandidate[];
  finalModelName: string | null;
}

export interface ExportCaseReportRequest {
  caseId: string;
  format: string;
  outputPath: string;
}

export interface ReportExportRecord {
  id: string;
  caseId: string | null;
  runId: string | null;
  format: string;
  path: string;
  exportedAt: string;
}
