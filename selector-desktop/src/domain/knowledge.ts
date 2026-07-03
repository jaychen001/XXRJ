import type { CoverageItem, CoverageStatus, ImplementationShape } from "./coverage";

export interface RootPdfIngestSummary {
  pdfPath: string;
  pageCount: number;
  catalogCount: number;
  coverageCount: number;
  knowledgeEntryCount: number;
  parameterCandidateCount: number;
}

export interface PdfCoverageRecord {
  id: string;
  chapter: string;
  implementationShape: ImplementationShape;
  status: CoverageStatus;
  sourcePageRange: string | null;
  catalogPage: string | null;
  catalogExcerpt: string;
  knowledgeEntryCount: number;
  notes: string;
}

export interface KnowledgeSearchRecord {
  id: string;
  title: string;
  content: string;
  page: string | null;
  tags: string[];
  sourceTitle: string;
}

export interface ParameterCandidateRecord {
  id: string;
  name: string;
  value: string;
  unit: string | null;
  scenario: string;
  sourcePage: string | null;
  status: "pending" | "confirmed" | "ignored";
}

export function mapCoverageRecord(record: PdfCoverageRecord): CoverageItem {
  return {
    id: record.id,
    chapter: record.chapter,
    shape: record.implementationShape,
    requirement: record.notes,
    status: record.status,
    source: record.sourcePageRange ?? record.catalogPage ?? "根目录 PDF 待索引",
    entryCount: record.knowledgeEntryCount,
    catalogPage: record.catalogPage,
    catalogExcerpt: record.catalogExcerpt,
    notes: record.notes,
  };
}
