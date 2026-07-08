import { FileDown } from "lucide-react";
import { useMemo, useState } from "react";
import type { CalculationRequest, CalculationResult } from "../../domain/calculation";
import type { RecommendationCandidate } from "../../domain/vendor";
import { exportCalculationReport } from "../../shared/api/report";
import "./report-export-page.css";

interface ReportExportDialogProps {
  caseName: string;
  notes: string;
  request: CalculationRequest | null;
  result: CalculationResult;
  candidates: RecommendationCandidate[];
}

export function ReportExportDialog({
  caseName,
  notes,
  request,
  result,
  candidates,
}: ReportExportDialogProps) {
  const [format, setFormat] = useState("pdf");
  const [outputPath, setOutputPath] = useState("");
  const [finalModelName, setFinalModelName] = useState("");
  const [status, setStatus] = useState("等待导出");
  const [isBusy, setIsBusy] = useState(false);
  const defaultName = useMemo(() => safeFileName(caseName || result.moduleName), [caseName, result.moduleName]);

  async function handleExport() {
    if (!request) {
      setStatus("没有计算输入，不能导出");
      return;
    }
    const path = outputPath.trim() || `${defaultName}.${format === "pdf" ? "pdf" : "xlsx"}`;
    setIsBusy(true);
    setStatus("导出中");
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
        finalModelName: finalModelName || null,
      });
      setStatus(`已导出：${record.path}`);
    } catch (error: unknown) {
      setStatus(error instanceof Error ? error.message : String(error));
    } finally {
      setIsBusy(false);
    }
  }

  return (
    <section className="report-export-dialog" aria-label="导出当前计算报告">
      <header>
        <div>
          <h3>报告导出</h3>
          <p>{status}</p>
        </div>
        <button
          className="secondary-button"
          type="button"
          disabled={isBusy || !request}
          onClick={() => void handleExport()}
        >
          <FileDown size={16} aria-hidden="true" />
          导出
        </button>
      </header>
      <div className="report-export-dialog__grid">
        <label>
          格式
          <select
            aria-label="当前报告导出格式"
            value={format}
            onChange={(event) => setFormat(event.currentTarget.value)}
          >
            <option value="pdf">PDF</option>
            <option value="xlsx">Excel</option>
          </select>
        </label>
        <label>
          输出路径
          <input
            aria-label="当前报告输出路径"
            placeholder={`${defaultName}.${format === "pdf" ? "pdf" : "xlsx"}`}
            value={outputPath}
            onChange={(event) => setOutputPath(event.currentTarget.value)}
          />
        </label>
        <label>
          最终型号
          <select
            aria-label="最终选择型号"
            value={finalModelName}
            onChange={(event) => setFinalModelName(event.currentTarget.value)}
          >
            <option value="">未选择</option>
            {candidates.map((candidate) => (
              <option key={candidate.model.id} value={candidate.model.modelName}>
                {candidate.model.modelName}
              </option>
            ))}
          </select>
        </label>
      </div>
    </section>
  );
}

function safeFileName(value: string): string {
  return value.trim().replace(/[\\/:*?"<>|]/g, "-") || "selector-report";
}
