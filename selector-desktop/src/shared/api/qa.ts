import { invoke } from "@tauri-apps/api/core";
import type { QaCoverageAudit } from "../../domain/qa";

export function getQaCoverageAudit(): Promise<QaCoverageAudit> {
  return invoke<QaCoverageAudit>("get_qa_coverage_audit");
}
