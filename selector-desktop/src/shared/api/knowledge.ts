import { invoke } from "@tauri-apps/api/core";
import type {
  KnowledgeSearchRecord,
  ParameterCandidateRecord,
  PdfCoverageRecord,
  RootPdfIngestSummary,
} from "../../domain/knowledge";

export function getPdfCoverageItems(): Promise<PdfCoverageRecord[]> {
  return invoke<PdfCoverageRecord[]>("get_pdf_coverage_items");
}

export function ingestRootPdfNote(): Promise<RootPdfIngestSummary> {
  return invoke<RootPdfIngestSummary>("ingest_root_pdf_note");
}

export function searchKnowledgeEntries(query: string): Promise<KnowledgeSearchRecord[]> {
  return invoke<KnowledgeSearchRecord[]>("search_knowledge_entries", { query });
}

export function listRecentKnowledgeEntries(): Promise<KnowledgeSearchRecord[]> {
  return invoke<KnowledgeSearchRecord[]>("list_recent_knowledge_entries");
}

export function listParameterCandidates(): Promise<ParameterCandidateRecord[]> {
  return invoke<ParameterCandidateRecord[]>("list_parameter_candidates");
}

export function updateParameterCandidateStatus(
  id: string,
  status: "confirmed" | "ignored",
): Promise<ParameterCandidateRecord> {
  return invoke<ParameterCandidateRecord>("update_parameter_candidate_status", { id, status });
}
