import { Activity, ShieldCheck } from "lucide-react";
import { useEffect, useState } from "react";
import type { QaCoverageAudit, QaRegressionReport } from "../../domain/qa";
import { getQaCoverageAudit, runQaRegression } from "../../shared/api/qa";
import "./qa-coverage-page.css";

export function QaCoveragePage() {
  const [audit, setAudit] = useState<QaCoverageAudit | null>(null);
  const [status, setStatus] = useState("读取 QA 覆盖检查中");
  const [regression, setRegression] = useState<QaRegressionReport | null>(null);
  const [regressionStatus, setRegressionStatus] = useState("回归样例尚未运行");
  const [isRegressionRunning, setIsRegressionRunning] = useState(false);

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

  async function handleRunRegression() {
    setIsRegressionRunning(true);
    setRegressionStatus("回归样例执行中");
    try {
      const report = await runQaRegression();
      setRegression(report);
      setRegressionStatus(
        report.status === "pass"
          ? `回归样例通过：${report.passedCases}/${report.totalCases}`
          : `回归样例失败：${report.failedCases}/${report.totalCases}`,
      );
    } catch (error: unknown) {
      setRegressionStatus(error instanceof Error ? error.message : String(error));
    } finally {
      setIsRegressionRunning(false);
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
        <div className="qa-page__actions">
          <button className="secondary-button" type="button" onClick={() => void loadAudit()}>
            <ShieldCheck size={16} aria-hidden="true" />
            重新检查
          </button>
          <button
            className="primary-button"
            type="button"
            onClick={() => void handleRunRegression()}
            disabled={isRegressionRunning}
          >
            <Activity size={16} aria-hidden="true" />
            运行回归样例
          </button>
        </div>
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

          <section className="qa-regression" aria-label="回归样例验证">
            <div className="qa-section-heading">
              <div>
                <h2>回归样例</h2>
                <p>执行公式和规则模块的内置样例，核对过程步骤、需求参数、规则判断和来源页码。</p>
              </div>
              <span className="qa-status" role="status">
                {regressionStatus}
              </span>
            </div>

            {regression ? (
              <>
                <div className="qa-summary">
                  <div>
                    <span>样例总数</span>
                    <strong>{regression.totalCases}</strong>
                  </div>
                  <div>
                    <span>通过</span>
                    <strong>{regression.passedCases}</strong>
                  </div>
                  <div>
                    <span>失败</span>
                    <strong>{regression.failedCases}</strong>
                  </div>
                </div>

                <div className="qa-table-wrap">
                  <table aria-label="回归样例结果表">
                    <thead>
                      <tr>
                        <th>分组</th>
                        <th>样例</th>
                        <th>模块</th>
                        <th>状态</th>
                        <th>结果</th>
                      </tr>
                    </thead>
                    <tbody>
                      {regression.groups.flatMap((group) =>
                        group.cases.map((item) => (
                          <tr key={`${group.label}-${item.name}`}>
                            <td>{group.label}</td>
                            <td>{item.name}</td>
                            <td>{item.moduleId}</td>
                            <td>{item.status === "pass" ? "通过" : "失败"}</td>
                            <td>{item.detail}</td>
                          </tr>
                        )),
                      )}
                    </tbody>
                  </table>
                </div>
              </>
            ) : null}
          </section>
        </>
      ) : null}
    </section>
  );
}
