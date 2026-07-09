import { useEffect, useState } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import { CheckCircle2, FileSpreadsheet, FolderOpen, Upload } from "lucide-react";
import type { CalculationResult } from "../../domain/calculation";
import type { FieldMapping, VendorImportPreview } from "../../domain/vendor";
import { confirmVendorImport, previewVendorImport } from "../../shared/api/vendor";
import {
  formatFromPath,
  inferVendorComponentType,
  optionsFor,
  replaceMapping,
} from "./vendor-import-options";

interface VendorImportPanelProps {
  result: CalculationResult;
  onImported: () => void;
}

export function VendorImportPanel({ result, onImported }: VendorImportPanelProps) {
  const [sourceFile, setSourceFile] = useState("");
  const [sourceFormat, setSourceFormat] = useState("");
  const [libraryName, setLibraryName] = useState(`${result.moduleName}样本库`);
  const [componentType, setComponentType] = useState(inferVendorComponentType(result));
  const [versionName, setVersionName] = useState("v1");
  const [preview, setPreview] = useState<VendorImportPreview | null>(null);
  const [mappings, setMappings] = useState<FieldMapping[]>([]);
  const [confirmed, setConfirmed] = useState(false);
  const [isBusy, setIsBusy] = useState(false);
  const [status, setStatus] = useState("输入本地样本路径，读取预览后确认入库。");

  useEffect(() => {
    setLibraryName(`${result.moduleName}样本库`);
    setComponentType(inferVendorComponentType(result));
    setPreview(null);
    setMappings([]);
    setConfirmed(false);
    setStatus("输入本地样本路径，读取预览后确认入库。");
  }, [result.moduleId, result.moduleName]);

  async function handleSelectFile() {
    setIsBusy(true);
    setStatus("正在打开文件选择窗口");
    try {
      const selected = await open({
        multiple: false,
        filters: [
          {
            name: "厂家样本",
            extensions: ["pdf", "csv", "tsv", "xlsx", "xls", "xlsm"],
          },
        ],
      });
      if (typeof selected !== "string") {
        setStatus("未选择样本文件。");
        return;
      }
      setSourceFile(selected);
      setSourceFormat(formatFromPath(selected));
      setStatus("已选择样本文件，可读取预览。");
    } catch (error: unknown) {
      setStatus(error instanceof Error ? error.message : String(error));
    } finally {
      setIsBusy(false);
    }
  }

  async function handlePreview() {
    if (!sourceFile.trim()) {
      setStatus("请先输入样本文件路径。");
      return;
    }
    setIsBusy(true);
    setStatus("正在读取样本预览");
    try {
      const nextPreview = await previewVendorImport({
        sourceFile: sourceFile.trim(),
        sourceFormat,
      });
      setPreview(nextPreview);
      setMappings(nextPreview.suggestedMappings);
      setConfirmed(false);
      setStatus(
        `识别到 ${nextPreview.rows.length} 个型号，失败 ${nextPreview.failedRows.length} 行。`,
      );
    } catch (error: unknown) {
      setPreview(null);
      setMappings([]);
      setStatus(error instanceof Error ? error.message : String(error));
    } finally {
      setIsBusy(false);
    }
  }

  async function handleConfirm() {
    if (!preview) {
      setStatus("请先读取样本预览。");
      return;
    }
    if (!confirmed) {
      setStatus("请先确认字段映射。");
      return;
    }
    setIsBusy(true);
    setStatus("正在写入样本库");
    try {
      const summary = await confirmVendorImport({
        libraryName,
        componentType,
        versionName,
        confirmed,
        preview,
        mappings,
      });
      setStatus(`已导入 ${summary.importedModels} 个型号，失败 ${summary.failedRows} 行。`);
      onImported();
    } catch (error: unknown) {
      setStatus(error instanceof Error ? error.message : String(error));
    } finally {
      setIsBusy(false);
    }
  }

  return (
    <section className="vendor-inline-panel" aria-label="导入厂家样本">
      <header>
        <div>
          <h4>导入样本</h4>
          <p>读取本地 PDF、Excel 或 CSV 样本，确认字段后参与型号匹配。</p>
        </div>
        <FileSpreadsheet size={17} aria-hidden="true" />
      </header>

      <div className="vendor-inline-grid">
        <label>
          <span>样本文件路径</span>
          <input
            aria-label="样本文件路径"
            placeholder="例如 D:\\samples\\servo.csv"
            value={sourceFile}
            onChange={(event) => setSourceFile(event.target.value)}
          />
        </label>
        <label>
          <span>格式</span>
          <select
            aria-label="样本文件格式"
            value={sourceFormat}
            onChange={(event) => setSourceFormat(event.target.value)}
          >
            <option value="">自动识别</option>
            <option value="pdf">PDF</option>
            <option value="csv">CSV</option>
            <option value="tsv">TSV</option>
            <option value="xlsx">Excel</option>
            <option value="xls">Excel 97-2003</option>
            <option value="xlsm">Excel 含宏</option>
          </select>
        </label>
        <label>
          <span>样本库名称</span>
          <input
            aria-label="样本库名称"
            value={libraryName}
            onChange={(event) => setLibraryName(event.target.value)}
          />
        </label>
        <label>
          <span>样本类型</span>
          <input
            aria-label="样本类型"
            value={componentType}
            onChange={(event) => setComponentType(event.target.value)}
          />
        </label>
      </div>

      <button
        className="vendor-inline-button"
        type="button"
        disabled={isBusy}
        onClick={() => void handleSelectFile()}
      >
        <FolderOpen size={15} aria-hidden="true" />
        选择文件
      </button>

      <button className="vendor-inline-button" type="button" disabled={isBusy} onClick={() => void handlePreview()}>
        <Upload size={15} aria-hidden="true" />
        读取预览
      </button>

      <span className="vendor-inline-status" role="status">
        {status}
      </span>

      {preview ? (
        <div className="vendor-preview">
          <strong>抽取置信度 {Math.round(preview.confidence * 100)}%</strong>
          <PreviewRows preview={preview} />
          <MappingEditor mappings={mappings} onMappingsChange={setMappings} />
          <label className="vendor-inline-confirm">
            <input
              type="checkbox"
              checked={confirmed}
              onChange={(event) => setConfirmed(event.target.checked)}
            />
            <span>我已核对字段映射</span>
          </label>
          <button
            className="vendor-inline-button vendor-inline-button--primary"
            type="button"
            disabled={isBusy || !confirmed}
            onClick={() => void handleConfirm()}
          >
            <CheckCircle2 size={15} aria-hidden="true" />
            确认导入
          </button>
        </div>
      ) : null}
    </section>
  );
}

function PreviewRows({ preview }: { preview: VendorImportPreview }) {
  return (
    <div className="vendor-preview__rows" aria-label="样本预览行">
      {preview.rows.slice(0, 3).map((row) => (
        <p key={row.rowIndex}>
          {row.modelName} · {row.parameters.length} 项参数
        </p>
      ))}
      {preview.warnings.map((warning) => (
        <p key={warning}>{warning}</p>
      ))}
    </div>
  );
}

function MappingEditor({
  mappings,
  onMappingsChange,
}: {
  mappings: FieldMapping[];
  onMappingsChange: (mappings: FieldMapping[]) => void;
}) {
  return (
    <div className="vendor-mapping" aria-label="字段映射">
      {mappings.map((mapping, index) => (
        <div className="vendor-mapping__row" key={`${mapping.sourceField}-${index}`}>
          <span>{mapping.sourceField}</span>
          <select
            aria-label={`${mapping.sourceField}目标字段`}
            value={mapping.targetField}
            onChange={(event) =>
              onMappingsChange(replaceMapping(mappings, index, "targetField", event.target.value))
            }
          >
            {optionsFor(mapping.targetField).map(([value, label]) => (
              <option key={value} value={value}>
                {label}
              </option>
            ))}
          </select>
          <input
            aria-label={`${mapping.sourceField}单位`}
            value={mapping.unit ?? ""}
            onChange={(event) =>
              onMappingsChange(replaceMapping(mappings, index, "unit", event.target.value))
            }
          />
        </div>
      ))}
    </div>
  );
}
