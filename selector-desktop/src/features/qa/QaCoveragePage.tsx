import { ShieldCheck } from "lucide-react";
import { useEffect, useState } from "react";
import type { QaCoverageAudit } from "../../domain/qa";
import { getQaCoverageAudit } from "../../shared/api/qa";
import "./qa-coverage-page.css";

export function QaCoveragePage() {
  const [audit, setAudit] = useState<QaCoverageAudit | null>(null);
  const [status, setStatus] = useState("读取 QA 覆盖检查中");

  useEffect(() => {
    void loadAudit();
  }, []);

  async function loadAudit() {
    try {
      const record = await getQaCoverageAudit();
      setAudit(record);
      setStatus(record.status === "pass" ? "QA 覆盖检查通过" : "QA 覆盖检查失败");
    } catch (error: unknown) {
      setStatus(error instanceof Error ? error.message : String(error));
    }
  }

  return (
    <section className="qa-page" aria-labelledby="qa-title">
      <div className="qa-page__header">
        <div>
          <h1 className="page-title" id="qa-title">
            QA 覆盖检查
          </h1>
          <p className="page-subtitle">核对根目录 PDF 23 章覆盖、来源页码和报告导出能力。</p>
        </div>
        <button className="secondary-button" type="button" onClick={() => void loadAudit()}>
          <ShieldCheck size={16} aria-hidden="true" />
          重新检查
        </button>
      </div>

      <span className="qa-status" role="status">
        {status}
      </span>

      {audit ? (
        <>
          <div className="qa-summary">
            <div>
              <span>PDF 章节</span>
              <strong>{audit.totalChapters}</strong>
            </div>
            <div>
              <span>已覆盖</span>
              <strong>{audit.doneChapters}</strong>
            </div>
            <div>
              <span>状态</span>
              <strong>{audit.status.toUpperCase()}</strong>
            </div>
          </div>

          <section className="qa-checks" aria-label="QA 检查项">
            {audit.checks.map((check) => (
              <article className={check.passed ? "qa-check--pass" : "qa-check--fail"} key={check.label}>
                <strong>{check.label}</strong>
                <span>{check.passed ? "通过" : "失败"}</span>
                <p>{check.detail}</p>
              </article>
            ))}
          </section>

          <section className="qa-table-wrap" aria-label="PDF 章节覆盖明细">
            <table aria-label="PDF 章节覆盖检查表">
              <thead>
                <tr>
                  <th>章节</th>
                  <th>状态</th>
                  <th>来源</th>
                  <th>实现形态</th>
                </tr>
              </thead>
              <tbody>
                {audit.items.map((item) => (
                  <tr key={item.id}>
                    <td>{item.chapter}</td>
                    <td>{item.status}</td>
                    <td>{item.sourcePage}</td>
                    <td>{item.implementationShape}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </section>
        </>
      ) : null}
    </section>
  );
}
