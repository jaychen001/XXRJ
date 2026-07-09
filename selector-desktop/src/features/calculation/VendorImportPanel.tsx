import { useEffect, useState } from "react";
import { CheckCircle2, FileSpreadsheet, Upload } from "lucide-react";
import type { CalculationResult } from "../../domain/calculation";
import type { FieldMapping, VendorImportPreview } from "../../domain/vendor";
import { confirmVendorImport, previewVendorImport } from "../../shared/api/vendor";

const FIELD_OPTIONS = [
  ["modelName", "型号"],
  ["brand", "品牌"],
  ["series", "系列"],
  ["outputTorque", "扭矩/力矩"],
  ["requiredSpeed", "转速"],
  ["power", "功率"],
  ["load", "推力/载荷"],
  ["bore", "缸径/直径"],
  ["stroke", "行程"],
  ["flowRate", "流量"],
  ["vacuumPressure", "真空压力"],
  ["dynamicLoadRating", "动额定载荷"],
  ["staticLoadRating", "静额定载荷"],
  ["allowableMoment", "允许力矩"],
  ["kineticEnergy", "允许动能"],
  ["ratedLife", "寿命"],
] as const;

interface VendorImportPanelProps {
  result: CalculationResult;
  onImported: () => void;
}

export function VendorImportPanel({ result, onImported }: VendorImportPanelProps) {
  const [sourceFile, setSourceFile] = useState("");
  const [sourceFormat, setSourceFormat] = useState("");
  const [libraryName, setLibraryName] = useState(`${result.moduleName}样本库`);
  const [componentType, setComponentType] = useState(inferComponentType(result));
  const [versionName, setVersionName] = useState("v1");
  const [preview, setPreview] = useState<VendorImportPreview | null>(null);
  const [mappings, setMappings] = useState<FieldMapping[]>([]);
  const [confirmed, setConfirmed] = useState(false);
  const [isBusy, setIsBusy] = useState(false);
  const [status, setStatus] = useState("输入本地样本路径，读取预览后确认入库。");

  useEffect(() => {
    setLibraryName(`${result.moduleName}样本库`);
    setComponentType(inferComponentType(result));
    setPreview(null);
    setMappings([]);
    setConfirmed(false);
    setStatus("输入本地样本路径，读取预览后确认入库。");
  }, [result.moduleId, result.moduleName]);

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

function replaceMapping(
  mappings: FieldMapping[],
  index: number,
  key: keyof FieldMapping,
  value: string,
): FieldMapping[] {
  return mappings.map((mapping, currentIndex) =>
    currentIndex === index ? { ...mapping, [key]: value } : mapping,
  );
}

function optionsFor(value: string): Array<[string, string]> {
  const options = FIELD_OPTIONS.map(([optionValue, label]) => [optionValue, label] as [string, string]);
  return options.some(([option]) => option === value) ? options : [[value, value], ...options];
}

function inferComponentType(result: CalculationResult): string {
  if (result.moduleId.includes("timing-belt")) return "同步轮同步带";
  if (result.moduleId.includes("v-belt")) return "V 带";
  if (result.moduleId.includes("chain")) return "链条";
  if (result.moduleId.includes("gear")) return "齿轮";
  if (result.moduleId.includes("reducer")) return "减速机";
  if (result.moduleId.includes("linear-module")) return "直线模组";
  if (result.moduleId.includes("ball-screw")) return "滚珠丝杠";
  if (result.moduleId.includes("linear-bearing")) return "直线轴承";
  if (result.moduleId.includes("linear-guide")) return "直线导轨";
  if (result.moduleId.includes("rolling-bearing")) return "滚动轴承";
  if (result.moduleId.includes("coupling")) return "联轴器";
  if (result.moduleId.includes("brake-clutch")) return "制动器/离合器";
  if (result.moduleId.includes("indexer")) return "分割器";
  if (result.moduleId.includes("rotary-actuator")) return "旋转气缸";
  if (result.moduleId.includes("slide-table")) return "滑台气缸";
  if (result.moduleId.includes("gripper")) return "手指气缸";
  if (result.moduleId.includes("cylinder")) return "气缸";
  if (result.moduleId.includes("vacuum")) return "真空";
  if (result.moduleId.includes("flow-control")) return "电磁阀";
  if (result.moduleId.includes("servo") || result.moduleId.includes("stepper")) return "伺服/步进电机";
  if (result.moduleId.includes("motor")) return "普通电机";
  return result.moduleName;
}
