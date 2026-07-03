import { useEffect, useState } from "react";
import { Check, DatabaseZap, X } from "lucide-react";
import type { ParameterCandidateRecord, RootPdfIngestSummary } from "../../domain/knowledge";
import {
  listParameterCandidates,
  updateParameterCandidateStatus,
} from "../../shared/api/knowledge";
import "./parameter-candidate-page.css";

interface ParameterCandidatePageProps {
  ingestSummary: RootPdfIngestSummary | null;
  onIngestRootPdf: () => Promise<RootPdfIngestSummary>;
}

export function ParameterCandidatePage({
  ingestSummary,
  onIngestRootPdf,
}: ParameterCandidatePageProps) {
  const [candidates, setCandidates] = useState<ParameterCandidateRecord[]>([]);
  const [status, setStatus] = useState("候选参数未加载");
  const [isBusy, setIsBusy] = useState(false);

  useEffect(() => {
    void loadCandidates();
  }, []);

  async function loadCandidates() {
    try {
      const records = await listParameterCandidates();
      setCandidates(records);
      setStatus(records.length > 0 ? `候选参数 ${records.length} 条` : "暂无候选参数");
    } catch (error: unknown) {
      setStatus(toErrorMessage(error));
    }
  }

  async function handleIngest() {
    setIsBusy(true);
    setStatus("正在从根目录 PDF 抽取候选参数");
    try {
      const summary = await onIngestRootPdf();
      await loadCandidates();
      setStatus(`抽取完成：${summary.parameterCandidateCount} 条候选参数`);
    } catch (error: unknown) {
      setStatus(toErrorMessage(error));
    } finally {
      setIsBusy(false);
    }
  }

  async function handleUpdate(id: string, nextStatus: "confirmed" | "ignored") {
    setIsBusy(true);
    try {
      await updateParameterCandidateStatus(id, nextStatus);
      await loadCandidates();
    } catch (error: unknown) {
      setStatus(toErrorMessage(error));
    } finally {
      setIsBusy(false);
    }
  }

  return (
    <section className="parameter-page" aria-labelledby="parameter-title">
      <div className="parameter-page__header">
        <div>
          <h1 className="page-title" id="parameter-title">
            内部参数库
          </h1>
          <p className="page-subtitle">
            PDF 候选值必须人工确认后才能入库；安全系数只作为提示，不自动代入计算。
          </p>
        </div>
        <button className="secondary-button" type="button" disabled={isBusy} onClick={handleIngest}>
          <DatabaseZap size={16} aria-hidden="true" />
          从 PDF 抽取候选
        </button>
      </div>

      <div className="parameter-status" role="status">
        <span>{status}</span>
        <span>{ingestSummary ? `${ingestSummary.pageCount} 页已索引` : "等待索引"}</span>
      </div>

      <div className="parameter-table-wrap">
        <table className="parameter-table" aria-label="内部参数候选列表">
          <thead>
            <tr>
              <th>参数</th>
              <th>候选值</th>
              <th>单位</th>
              <th>来源</th>
              <th>状态</th>
              <th aria-label="操作" />
            </tr>
          </thead>
          <tbody>
            {candidates.map((candidate) => (
              <tr key={candidate.id}>
                <td>
                  <strong>{candidate.name}</strong>
                  <span>{candidate.scenario}</span>
                </td>
                <td>{candidate.value}</td>
                <td>{candidate.unit ?? "-"}</td>
                <td>{candidate.sourcePage ?? "根目录 PDF"}</td>
                <td>{candidate.status}</td>
                <td>
                  <div className="parameter-actions">
                    <button
                      className="icon-button"
                      type="button"
                      aria-label={`确认${candidate.name}`}
                      disabled={isBusy || candidate.status !== "pending"}
                      onClick={() => void handleUpdate(candidate.id, "confirmed")}
                    >
                      <Check size={16} aria-hidden="true" />
                    </button>
                    <button
                      className="icon-button"
                      type="button"
                      aria-label={`忽略${candidate.name}`}
                      disabled={isBusy || candidate.status !== "pending"}
                      onClick={() => void handleUpdate(candidate.id, "ignored")}
                    >
                      <X size={16} aria-hidden="true" />
                    </button>
                  </div>
                </td>
              </tr>
            ))}
            {candidates.length === 0 ? (
              <tr>
                <td colSpan={6}>暂无候选参数，请先从 PDF 抽取。</td>
              </tr>
            ) : null}
          </tbody>
        </table>
      </div>
    </section>
  );
}

function toErrorMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}
