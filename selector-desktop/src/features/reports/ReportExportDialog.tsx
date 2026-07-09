import { useMemo, useState } from "react";
import { save } from "@tauri-apps/plugin-dialog";
import { Download, FolderOpen } from "lucide-react";
import type { CalculationRequest, CalculationResult } from "../../domain/calculation";
import type { RecommendationCandidate } from "../../domain/vendor";
import { exportCalculationReport } from "../../shared/api/report";

interface ReportExportDialogProps {
  caseName: string;
  notes: string;
  request: CalculationRequest | null;
  result: CalculationResult | null;
  candidates: RecommendationCandidate[];
  finalModelName?: string | null;
}

export function ReportExportDialog({
  caseName,
  notes,
  request,
  result,
  candidates,
  finalModelName = null,
}: ReportExportDialogProps) {
  const [format, setFormat] = useState("pdf");
  const [outputPath, setOutputPath] = useState("");
  const [status, setStatus] = useState("计算完成后可导出报告");
  const defaultName = useMemo(
    () => `${caseName.trim() || result?.moduleName || "选型计算报告"}`,
    [caseName, result?.moduleName],
  );
  const extension = extensionForFormat(format);

  async function handleSelectOutputPath() {
    if (!request || !result) {
      setStatus("请先完成一次计算。");
      return;
    }
    setStatus("正在选择保存位置");
    try {
      const selected = await save({
        defaultPath: `${defaultName}.${extension}`,
        filters: [
          {
            name: format === "pdf" ? "PDF 报告" : "Excel 报告",
            extensions: [extension],
          },
        ],
      });
      if (!selected) {
        setStatus("未选择保存位置。");
        return;
      }
      setOutputPath(selected);
      setStatus("已选择保存位置。");
    } catch (error: unknown) {
      setStatus(error instanceof Error ? error.message : String(error));
    }
  }

  async function handleExport() {
    if (!request || !result) {
      setStatus("请先完成一次计算。");
      return;
    }
    const path = outputPath.trim() || `${defaultName}.${extension}`;
    setStatus("正在导出报告");
    try {
      const record = await exportCalculationReport({
        format,
        outputPath: path,
        caseId: null,
        caseName,
        notes,
        request,
        result,
        candidates,
        finalModelName,
      });
      setStatus(`已导出：${record.path}`);
    } catch (error: unknown) {
      setStatus(error instanceof Error ? error.message : String(error));
    }
  }

  return (
    <section className="report-export-inline" aria-label="导出当前计算报告">
      <div>
        <h3>导出报告</h3>
        <p>报告包含输入参数、公式过程、计算结果和风险提示。</p>
      </div>
      <div className="report-export-inline__controls">
        <label>
          <span>格式</span>
          <select
            aria-label="当前报告导出格式"
            value={format}
            onChange={(event) => setFormat(event.target.value)}
          >
            <option value="pdf">PDF</option>
            <option value="xlsx">Excel</option>
          </select>
        </label>
        <label>
          <span>保存路径</span>
          <input
            aria-label="当前报告输出路径"
            placeholder={`${defaultName}.${extension}`}
            value={outputPath}
            onChange={(event) => setOutputPath(event.target.value)}
          />
        </label>
        <button
          className="primary-button primary-button--secondary"
          type="button"
          onClick={() => void handleSelectOutputPath()}
        >
          <FolderOpen size={16} aria-hidden="true" />
          选择位置
        </button>
        <button className="primary-button" type="button" onClick={() => void handleExport()}>
          <Download size={16} aria-hidden="true" />
          导出
        </button>
      </div>
      <span className="report-export-inline__status" role="status">
        {status}
      </span>
    </section>
  );
}

function extensionForFormat(format: string): "pdf" | "xlsx" {
  return format === "pdf" ? "pdf" : "xlsx";
}
