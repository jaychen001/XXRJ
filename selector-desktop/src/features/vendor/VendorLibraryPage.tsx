import { Database, Trash2 } from "lucide-react";
import { useEffect, useState } from "react";
import type { VendorLibraryRecord, VendorModelRecord } from "../../domain/vendor";
import {
  deleteVendorLibrary,
  listVendorLibraries,
  listVendorModels,
  setVendorLibraryEnabled,
} from "../../shared/api/vendor";
import { VendorImportWizard } from "./VendorImportWizard";
import "./vendor-library-page.css";

export function VendorLibraryPage() {
  const [libraries, setLibraries] = useState<VendorLibraryRecord[]>([]);
  const [models, setModels] = useState<VendorModelRecord[]>([]);
  const [selectedLibraryId, setSelectedLibraryId] = useState("");
  const [status, setStatus] = useState("读取厂家样本库中");

  useEffect(() => {
    void loadLibraries();
  }, []);

  useEffect(() => {
    void loadModels(selectedLibraryId);
  }, [selectedLibraryId]);

  async function loadLibraries() {
    try {
      const records = await listVendorLibraries();
      setLibraries(records);
      setSelectedLibraryId((current) => current || records[0]?.id || "");
      setStatus(records.length > 0 ? `样本库 ${records.length} 个` : "暂无厂家样本库");
    } catch (error: unknown) {
      setStatus(toErrorMessage(error));
    }
  }

  async function loadModels(libraryId: string) {
    try {
      setModels(await listVendorModels(libraryId || undefined));
    } catch (error: unknown) {
      setStatus(toErrorMessage(error));
    }
  }

  async function handleToggle(record: VendorLibraryRecord) {
    await setVendorLibraryEnabled(record.id, !record.enabled);
    await loadLibraries();
  }

  async function handleDelete(record: VendorLibraryRecord) {
    if (!window.confirm(`确认删除样本库 ${record.name}？`)) {
      return;
    }
    await deleteVendorLibrary(record.id);
    setSelectedLibraryId("");
    await loadLibraries();
  }

  return (
    <section className="vendor-page" aria-labelledby="vendor-title">
      <div className="vendor-page__header">
        <div>
          <h1 className="page-title" id="vendor-title">
            厂家样本库
          </h1>
          <p className="page-subtitle">导入 PDF、Excel、CSV 型号参数，确认后参与计算结果推荐。</p>
        </div>
        <span className="vendor-status" role="status">
          {status}
        </span>
      </div>

      <VendorImportWizard
        onImported={async () => {
          await loadLibraries();
          await loadModels(selectedLibraryId);
        }}
      />

      <div className="vendor-layout">
        <aside className="vendor-libraries" aria-label="厂家样本库列表">
          <header>
            <h2>样本库</h2>
            <span>{libraries.length}</span>
          </header>
          {libraries.map((library) => (
            <button
              className={`vendor-library-card${
                library.id === selectedLibraryId ? " vendor-library-card--active" : ""
              }`}
              key={library.id}
              type="button"
              onClick={() => setSelectedLibraryId(library.id)}
            >
              <Database size={16} aria-hidden="true" />
              <span>
                <strong>{library.name}</strong>
                <small>
                  {library.componentType} / {library.modelCount} 型号
                </small>
              </span>
              <em>{library.enabled ? "启用" : "停用"}</em>
            </button>
          ))}
        </aside>

        <section className="vendor-models" aria-label="厂家型号列表">
          <header>
            <h2>型号参数</h2>
            <div>
              {selectedLibrary(libraries, selectedLibraryId) ? (
                <>
                  <button
                    className="secondary-button"
                    type="button"
                    onClick={() => {
                      const record = selectedLibrary(libraries, selectedLibraryId);
                      if (record) {
                        void handleToggle(record);
                      }
                    }}
                  >
                    {selectedLibrary(libraries, selectedLibraryId)?.enabled ? "停用" : "启用"}
                  </button>
                  <button
                    className="danger-button"
                    type="button"
                    onClick={() => {
                      const record = selectedLibrary(libraries, selectedLibraryId);
                      if (record) {
                        void handleDelete(record);
                      }
                    }}
                  >
                    <Trash2 size={16} aria-hidden="true" />
                    删除
                  </button>
                </>
              ) : null}
            </div>
          </header>
          <div className="vendor-model-table-wrap">
            <table aria-label="厂家型号参数表">
              <thead>
                <tr>
                  <th>型号</th>
                  <th>品牌/系列</th>
                  <th>参数</th>
                  <th>来源</th>
                </tr>
              </thead>
              <tbody>
                {models.map((model) => (
                  <tr key={model.id}>
                    <td>{model.modelName}</td>
                    <td>{[model.brand, model.series].filter(Boolean).join(" / ") || "-"}</td>
                    <td>
                      {model.parameters.map((parameter) => (
                        <span key={parameter.sourceField}>
                          {parameter.label} {parameter.value} {parameter.unit}
                        </span>
                      ))}
                    </td>
                    <td>{model.sourcePage ?? model.libraryName}</td>
                  </tr>
                ))}
                {models.length === 0 ? (
                  <tr>
                    <td colSpan={4}>暂无型号，请先抽取预览并确认导入。</td>
                  </tr>
                ) : null}
              </tbody>
            </table>
          </div>
        </section>
      </div>
    </section>
  );
}

function selectedLibrary(records: VendorLibraryRecord[], id: string) {
  return records.find((record) => record.id === id) ?? null;
}

function toErrorMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}
