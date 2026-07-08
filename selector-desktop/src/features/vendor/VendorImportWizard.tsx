import { FileText, Upload } from "lucide-react";
import { useState } from "react";
import type { FieldMapping, VendorImportPreview } from "../../domain/vendor";
import { confirmVendorImport, previewVendorImport } from "../../shared/api/vendor";
import { FieldMappingTable } from "./FieldMappingTable";

interface VendorImportWizardProps {
  onImported: () => void;
}

export function VendorImportWizard({ onImported }: VendorImportWizardProps) {
  const [libraryName, setLibraryName] = useState("厂家样本库");
  const [componentType, setComponentType] = useState("伺服/步进电机");
  const [versionName, setVersionName] = useState("v1");
  const [sourceFile, setSourceFile] = useState("");
  const [sourceFormat, setSourceFormat] = useState("pdf");
  const [preview, setPreview] = useState<VendorImportPreview | null>(null);
  const [mappings, setMappings] = useState<FieldMapping[]>([]);
  const [confirmed, setConfirmed] = useState(false);
  const [status, setStatus] = useState("等待选择 PDF、CSV 或 Excel 样本");
  const [isBusy, setIsBusy] = useState(false);

  async function handlePreview() {
    if (!sourceFile.trim()) {
      setStatus("请先填写本地样本文件路径");
      return;
    }
    setIsBusy(true);
    setStatus("抽取预览中");
    try {
      const nextPreview = await previewVendorImport({
        sourceFile: sourceFile.trim(),
        sourceFormat,
      });
      setPreview(nextPreview);
      setMappings(nextPreview.suggestedMappings);
      setConfirmed(false);
      setStatus(`抽取 ${nextPreview.rows.length} 个疑似型号，置信度 ${formatPercent(nextPreview.confidence)}`);
    } catch (error: unknown) {
      setStatus(toErrorMessage(error));
    } finally {
      setIsBusy(false);
    }
  }

  async function handleConfirm() {
    if (!preview) {
      setStatus("请先生成抽取预览");
      return;
    }
    setIsBusy(true);
    setStatus("写入厂家型号库中");
    try {
      const summary = await confirmVendorImport({
        libraryName,
        componentType,
        versionName,
        confirmed,
        preview,
        mappings,
      });
      setStatus(`已导入 ${summary.importedModels} 个型号，失败 ${summary.failedRows} 行`);
      setPreview(null);
      setConfirmed(false);
      await onImported();
    } catch (error: unknown) {
      setStatus(toErrorMessage(error));
    } finally {
      setIsBusy(false);
    }
  }

  return (
    <section className="vendor-import" aria-label="厂家样本导入">
      <header>
        <div>
          <h2>样本导入</h2>
          <p>PDF 支持文本抽取；Excel 支持 xls/xlsx，CSV/TSV 适合人工整理后的参数表。</p>
        </div>
        <span role="status">{status}</span>
      </header>

      <div className="vendor-import__grid">
        <label>
          样本库名称
          <input
            aria-label="样本库名称"
            value={libraryName}
            onChange={(event) => setLibraryName(event.currentTarget.value)}
          />
        </label>
        <label>
          部件类型
          <input
            aria-label="部件类型"
            value={componentType}
            onChange={(event) => setComponentType(event.currentTarget.value)}
          />
        </label>
        <label>
          版本
          <input
            aria-label="样本版本"
            value={versionName}
            onChange={(event) => setVersionName(event.currentTarget.value)}
          />
        </label>
        <label>
          文件格式
          <select
            aria-label="样本格式"
            value={sourceFormat}
            onChange={(event) => setSourceFormat(event.currentTarget.value)}
          >
            <option value="pdf">PDF</option>
            <option value="xlsx">Excel xlsx</option>
            <option value="xls">Excel xls</option>
            <option value="csv">CSV</option>
            <option value="tsv">TSV</option>
          </select>
        </label>
      </div>

      <label className="vendor-import__path">
        本地文件路径
        <input
          aria-label="厂家样本文件路径"
          placeholder="D:\\samples\\servo.csv"
          value={sourceFile}
          onChange={(event) => setSourceFile(event.currentTarget.value)}
        />
      </label>

      <div className="vendor-import__actions">
        <button className="secondary-button" type="button" disabled={isBusy} onClick={() => void handlePreview()}>
          <FileText size={16} aria-hidden="true" />
          抽取预览
        </button>
        <label className="confirm-line">
          <input
            aria-label="已确认字段映射"
            type="checkbox"
            checked={confirmed}
            onChange={(event) => setConfirmed(event.currentTarget.checked)}
          />
          已逐项确认字段映射
        </label>
        <button
          className="primary-button"
          type="button"
          disabled={isBusy || !preview || !confirmed}
          onClick={() => void handleConfirm()}
        >
          <Upload size={16} aria-hidden="true" />
          确认导入
        </button>
      </div>

      {preview ? (
        <div className="vendor-preview">
          <FieldMappingTable mappings={mappings} onMappingsChange={setMappings} />
          <div className="vendor-preview__table-wrap">
            <table aria-label="厂家样本抽取预览">
              <thead>
                <tr>
                  <th>型号</th>
                  <th>品牌/系列</th>
                  <th>参数</th>
                  <th>来源</th>
                  <th>置信度</th>
                </tr>
              </thead>
              <tbody>
                {preview.rows.slice(0, 12).map((row) => (
                  <tr key={`${row.rowIndex}-${row.modelName}`}>
                    <td>{row.modelName}</td>
                    <td>{[row.brand, row.series].filter(Boolean).join(" / ") || "-"}</td>
                    <td>
                      {row.parameters.map((parameter) => (
                        <span key={parameter.sourceField}>
                          {parameter.label} {parameter.value} {parameter.unit}
                        </span>
                      ))}
                    </td>
                    <td>{row.sourcePage ?? "表格行"}</td>
                    <td>{formatPercent(row.confidence)}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
          {preview.warnings.map((warning) => (
            <p className="vendor-warning" key={warning}>
              {warning}
            </p>
          ))}
          {preview.failedRows.length > 0 ? (
            <div className="vendor-failed-rows">
              <strong>失败行</strong>
              {preview.failedRows.map((row) => (
                <span key={`${row.rowIndex}-${row.reason}`}>
                  第 {row.rowIndex} 行：{row.reason}
                </span>
              ))}
            </div>
          ) : null}
        </div>
      ) : null}
    </section>
  );
}

function formatPercent(value: number): string {
  return `${Math.round(value * 100)}%`;
}

function toErrorMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}
