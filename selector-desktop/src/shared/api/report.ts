import { invoke } from "@tauri-apps/api/core";
import type {
  ExportCaseReportRequest,
  ExportReportRequest,
  ReportExportRecord,
} from "../../domain/report";

export function exportCalculationReport(
  request: ExportReportRequest,
): Promise<ReportExportRecord> {
  return invoke<ReportExportRecord>("export_calculation_report", { request });
}

export function exportCaseReport(
  request: ExportCaseReportRequest,
): Promise<ReportExportRecord> {
  return invoke<ReportExportRecord>("export_case_report", { request });
}

export function listReportExports(): Promise<ReportExportRecord[]> {
  return invoke<ReportExportRecord[]>("list_report_exports");
}
