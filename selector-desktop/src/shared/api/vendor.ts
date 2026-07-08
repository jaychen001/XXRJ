import { invoke } from "@tauri-apps/api/core";
import type {
  ConfirmVendorImportRequest,
  RecommendationCandidate,
  RecommendationRequest,
  VendorImportPreview,
  VendorImportPreviewRequest,
  VendorImportSummary,
  VendorLibraryRecord,
  VendorModelRecord,
} from "../../domain/vendor";

export function previewVendorImport(
  request: VendorImportPreviewRequest,
): Promise<VendorImportPreview> {
  return invoke<VendorImportPreview>("preview_vendor_import", { request });
}

export function confirmVendorImport(
  request: ConfirmVendorImportRequest,
): Promise<VendorImportSummary> {
  return invoke<VendorImportSummary>("confirm_vendor_import", { request });
}

export function listVendorLibraries(): Promise<VendorLibraryRecord[]> {
  return invoke<VendorLibraryRecord[]>("list_vendor_libraries");
}

export function listVendorModels(libraryId?: string): Promise<VendorModelRecord[]> {
  return invoke<VendorModelRecord[]>("list_vendor_models", { libraryId: libraryId ?? null });
}

export function setVendorLibraryEnabled(
  id: string,
  enabled: boolean,
): Promise<VendorLibraryRecord> {
  return invoke<VendorLibraryRecord>("set_vendor_library_enabled", { id, enabled });
}

export function deleteVendorLibrary(id: string): Promise<boolean> {
  return invoke<boolean>("delete_vendor_library", { id });
}

export function recommendVendorModels(
  request: RecommendationRequest,
): Promise<RecommendationCandidate[]> {
  return invoke<RecommendationCandidate[]>("recommend_vendor_models", { request });
}
