import type { RequirementParameter } from "./calculation";

export interface VendorImportPreviewRequest {
  sourceFile: string;
  sourceFormat: string;
}

export interface VendorImportPreview {
  jobId: string;
  sourceFile: string;
  sourceFormat: string;
  confidence: number;
  rows: VendorPreviewRow[];
  failedRows: ImportFailureRow[];
  suggestedMappings: FieldMapping[];
  warnings: string[];
}

export interface ImportFailureRow {
  rowIndex: number;
  reason: string;
  rawText: string;
}

export interface VendorPreviewRow {
  rowIndex: number;
  modelName: string;
  brand: string;
  series: string;
  sourcePage: string | null;
  confidence: number;
  rawText: string;
  parameters: VendorParameter[];
}

export interface VendorParameter {
  field: string;
  label: string;
  value: number;
  unit: string;
  sourceField: string;
}

export interface FieldMapping {
  sourceField: string;
  targetField: string;
  unit: string | null;
}

export interface ConfirmVendorImportRequest {
  libraryName: string;
  componentType: string;
  versionName: string;
  confirmed: boolean;
  preview: VendorImportPreview;
  mappings: FieldMapping[];
}

export interface VendorImportSummary {
  library: VendorLibraryRecord;
  importedModels: number;
  failedRows: number;
  jobId: string;
}

export interface VendorLibraryRecord {
  id: string;
  name: string;
  componentType: string;
  sourceFile: string;
  sourceFormat: string;
  versionName: string;
  importedAt: string;
  enabled: boolean;
  modelCount: number;
  createdAt: string;
  updatedAt: string;
}

export interface VendorModelRecord {
  id: string;
  libraryId: string;
  libraryName: string;
  componentType: string;
  modelName: string;
  brand: string;
  series: string;
  parameters: VendorParameter[];
  normalizedParameters: Record<string, NormalizedParameter>;
  sourcePage: string | null;
  enabled: boolean;
}

export interface NormalizedParameter {
  label: string;
  value: number;
  unit: string;
  sourceField: string;
}

export interface RecommendationRequest {
  moduleId: string;
  componentType: string | null;
  requirements: RequirementParameter[];
  limit?: number;
}

export interface RecommendationCandidate {
  model: VendorModelRecord;
  score: number;
  matchedRules: MatchRuleResult[];
  failedRules: MatchRuleResult[];
}

export interface MatchRuleResult {
  requirementId: string;
  label: string;
  message: string;
  requiredValue: number;
  candidateValue: number | null;
  unit: string;
}
