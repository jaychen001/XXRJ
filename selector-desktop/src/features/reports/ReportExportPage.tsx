import { FileDown } from "lucide-react";
import { useEffect, useState } from "react";
import type { CaseRecord } from "../../domain/calculation";
import type { ReportExportRecord } from "../../domain/report";
import { listCalculationCases } from "../../shared/api/calculation";
import { exportCaseReport, listReportExports } from "../../shared/api/report";
import "./report-export-page.css";

export function ReportExportPage() {
  const [cases, setCases] = useState<CaseRecord[]>([]);
  const [exports, setExports] = useState<ReportExportRecord[]>([]);
  const [selectedCaseId, setSelectedCaseId] = useState("");
  const [format, setFormat] = useState("pdf");
  const [outputPath, setOutputPath] = useState("");
  const [status, setStatus] = useState("读取案例和导出记录中");
  const [isBusy, setIsBusy] = useState(false);

  useEffect(() => {
    void loadData();
  }, []);

  async function loadData() {
    try {
      const [caseRecords, exportRecords] = await Promise.all([
        listCalculationCases(),
        listReportExports(),
      ]);
      setCases(caseRecords);
      setExports(exportRecords);
      setSelectedCaseId((current) => current || caseRecords[0]?.id || "");
      setStatus(caseRecords.length > 0 ? `可导出案例 ${caseRecords.length} 个` : "暂无可导出案例");
    } catch (error: unknown) {
      setStatus(toErrorMessage(error));
    }
  }

  async function handleExport() {
    if (!selectedCaseId) {
      setStatus("请先选择案例");
      return;
    }
    const selectedCase = cases.find((item) => item.id === selectedCaseId);
    const path = outputPath.trim() || `${safeFileName(selectedCase?.name ?? "selector-report")}.${format === "pdf" ? "pdf" : "xlsx"}`;
    setIsBusy(true);
    setStatus("导出中");
    try {
      const record = await exportCaseReport({ caseId: selectedCaseId, format, outputPath: path });
      setStatus(`已导出：${record.path}`);
      setExports(await listReportExports());
    } catch (error: unknown) {
      setStatus(toErrorMessage(error));
    } finally {
      setIsBusy(false);
    }
  }

  return (
    <section className="report-page" aria-labelledby="report-title">
      <div className="report-page__header">
        <div>
          <h1 className="page-title" id="report-title">
            报告导出
          </h1>
          <p className="page-subtitle">从已保存案例导出 PDF 或 Excel 报告，并记录导出路径。</p>
        </div>
        <span className="report-status" role="status">
          {status}
        </span>
      </div>

      <section className="report-export-card" aria-label="案例报告导出">
        <div className="report-export-card__grid">
          <label>
            案例
            <select
              aria-label="报告导出案例"
              value={selectedCaseId}
              onChange={(event) => setSelectedCaseId(event.currentTarget.value)}
            >
              {cases.map((item) => (
                <option key={item.id} value={item.id}>
                  {item.name} / {item.moduleName}
                </option>
              ))}
            </select>
          </label>
          <label>
            格式
            <select
              aria-label="报告导出格式"
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
              aria-label="报告输出路径"
              placeholder="D:\\reports\\selector-report.pdf"
              value={outputPath}
              onChange={(event) => setOutputPath(event.currentTarget.value)}
            />
          </label>
        </div>
        <button
          className="secondary-button"
          type="button"
          disabled={isBusy || !selectedCaseId}
          onClick={() => void handleExport()}
        >
          <FileDown size={16} aria-hidden="true" />
          导出报告
        </button>
      </section>

      <section className="report-history" aria-label="报告导出记录">
        <h2>导出记录</h2>
        <table aria-label="报告导出记录表">
          <thead>
            <tr>
              <th>格式</th>
              <th>路径</th>
              <th>时间</th>
            </tr>
          </thead>
          <tbody>
            {exports.map((record) => (
              <tr key={record.id}>
                <td>{record.format.toUpperCase()}</td>
                <td>{record.path}</td>
                <td>{record.exportedAt}</td>
              </tr>
            ))}
            {exports.length === 0 ? (
              <tr>
                <td colSpan={3}>暂无导出记录。</td>
              </tr>
            ) : null}
          </tbody>
        </table>
      </section>
    </section>
  );
}

function safeFileName(value: string): string {
  return value.trim().replace(/[\\/:*?"<>|]/g, "-") || "selector-report";
}

function toErrorMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}
