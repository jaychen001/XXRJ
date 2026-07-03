import { useEffect, useState } from "react";
import type {
  CaseDetailRecord,
  CaseFilter,
  CaseRecord,
  ModuleDefinition,
} from "../../domain/calculation";
import {
  deleteCalculationCase,
  duplicateCalculationCase,
  getCalculationCaseDetail,
  listCalculationModules,
  listCalculationCases,
  rerunCalculationCase,
  updateCalculationCase,
} from "../../shared/api/calculation";
import { CaseDetailPanel } from "./CaseDetailPanel";
import "./case-library-page.css";

interface CaseLibraryPageProps {
  onOpenCaseForCalculation: (detail: CaseDetailRecord) => void;
}

export function CaseLibraryPage({ onOpenCaseForCalculation }: CaseLibraryPageProps) {
  const [cases, setCases] = useState<CaseRecord[]>([]);
  const [modules, setModules] = useState<ModuleDefinition[]>([]);
  const [query, setQuery] = useState("");
  const [moduleFilter, setModuleFilter] = useState("");
  const [createdFrom, setCreatedFrom] = useState("");
  const [createdTo, setCreatedTo] = useState("");
  const [selectedCase, setSelectedCase] = useState<CaseRecord | null>(null);
  const [selectedDetail, setSelectedDetail] = useState<CaseDetailRecord | null>(null);
  const [editName, setEditName] = useState("");
  const [editNotes, setEditNotes] = useState("");
  const [status, setStatus] = useState("读取案例中");

  useEffect(() => {
    void loadCases();
    void loadModules();
  }, []);

  useEffect(() => {
    setEditName(selectedCase?.name ?? "");
    setEditNotes(selectedCase?.notes ?? "");
    if (selectedCase) {
      void loadCaseDetail(selectedCase.id);
    } else {
      setSelectedDetail(null);
    }
  }, [selectedCase]);

  async function loadModules() {
    try {
      setModules(await listCalculationModules());
    } catch (error: unknown) {
      setStatus(toErrorMessage(error));
    }
  }

  async function loadCases(overrides: Partial<CaseFilter> = {}) {
    try {
      const records = await listCalculationCases({
        query,
        moduleId: moduleFilter,
        createdFrom,
        createdTo,
        ...overrides,
      });
      setCases(records);
      setSelectedCase(
        (current) => records.find((item) => item.id === current?.id) ?? records[0] ?? null,
      );
      setStatus(records.length > 0 ? `案例 ${records.length} 条` : "暂无案例");
    } catch (error: unknown) {
      setStatus(toErrorMessage(error));
    }
  }

  async function loadCaseDetail(id: string) {
    try {
      setSelectedDetail(await getCalculationCaseDetail(id));
    } catch {
      setSelectedDetail(null);
    }
  }

  async function handleDuplicate(id: string) {
    const record = await duplicateCalculationCase(id);
    await loadCases();
    setSelectedCase(record);
    await handleOpenForCalculation(record.id);
  }

  async function handleRerun(id: string) {
    const record = await rerunCalculationCase(id);
    await loadCases();
    setStatus(`已重新计算：${record.result.summary}`);
  }

  async function handleDelete(id: string) {
    if (!window.confirm("确认删除该案例？")) {
      return;
    }
    await deleteCalculationCase(id);
    await loadCases();
  }

  async function handleUpdate() {
    if (!selectedCase) {
      return;
    }
    try {
      const record = await updateCalculationCase({
        id: selectedCase.id,
        name: editName,
        notes: editNotes,
      });
      await loadCases();
      setSelectedCase(record);
      setStatus("案例已更新");
    } catch (error: unknown) {
      setStatus(toErrorMessage(error));
    }
  }

  async function handleOpenForCalculation(id = selectedCase?.id) {
    if (!id) {
      return;
    }
    try {
      const detail = await getCalculationCaseDetail(id);
      onOpenCaseForCalculation(detail);
    } catch (error: unknown) {
      setStatus(toErrorMessage(error));
    }
  }

  return (
    <section className="case-page" aria-labelledby="case-title">
      <div className="case-page__header">
        <div>
          <h1 className="page-title" id="case-title">
            案例库
          </h1>
          <p className="page-subtitle">保存、复制、重新计算和删除历史选型案例。</p>
        </div>
        <span className="case-status" role="status">
          {status}
        </span>
      </div>

      <div className="case-toolbar">
        <input
          aria-label="搜索案例"
          placeholder="搜索案例"
          value={query}
          onChange={(event) => setQuery(event.target.value)}
          onKeyDown={(event) => {
            if (event.key === "Enter") {
              void loadCases({ query: event.currentTarget.value });
            }
          }}
        />
        <select
          aria-label="模块筛选"
          value={moduleFilter}
          onChange={(event) => setModuleFilter(event.target.value)}
        >
          <option value="">全部模块</option>
          {modules.map((module) => (
            <option key={module.id} value={module.id}>
              {module.name}
            </option>
          ))}
        </select>
        <input
          aria-label="开始日期"
          type="date"
          value={createdFrom}
          onChange={(event) => setCreatedFrom(event.target.value)}
        />
        <input
          aria-label="结束日期"
          type="date"
          value={createdTo}
          onChange={(event) => setCreatedTo(event.target.value)}
        />
        <button className="secondary-button" type="button" onClick={() => void loadCases()}>
          搜索
        </button>
      </div>

      <div className="case-layout">
        <div className="case-table-wrap">
          <table className="case-table" aria-label="案例列表">
            <thead>
              <tr>
                <th>案例</th>
                <th>模块</th>
                <th>结果摘要</th>
                <th>风险</th>
                <th aria-label="操作" />
              </tr>
            </thead>
            <tbody>
              {cases.map((item) => (
                <tr
                  className={item.id === selectedCase?.id ? "case-table__row--active" : ""}
                  key={item.id}
                  onClick={() => setSelectedCase(item)}
                >
                  <td>
                    <strong>{item.name}</strong>
                    <span>{item.updatedAt}</span>
                  </td>
                  <td>{item.moduleName}</td>
                  <td>{item.resultSummary}</td>
                  <td>{item.riskCount}</td>
                  <td>
                    <div className="case-actions">
                      <button
                        type="button"
                        onClick={(event) => {
                          event.stopPropagation();
                          void handleRerun(item.id);
                        }}
                      >
                        重算
                      </button>
                      <button
                        type="button"
                        onClick={(event) => {
                          event.stopPropagation();
                          void handleDuplicate(item.id);
                        }}
                      >
                        复制
                      </button>
                      <button
                        type="button"
                        onClick={(event) => {
                          event.stopPropagation();
                          void handleOpenForCalculation(item.id);
                        }}
                      >
                        改参数
                      </button>
                      <button
                        type="button"
                        onClick={(event) => {
                          event.stopPropagation();
                          void handleDelete(item.id);
                        }}
                      >
                        删除
                      </button>
                    </div>
                  </td>
                </tr>
              ))}
              {cases.length === 0 ? (
                <tr>
                  <td colSpan={5}>暂无案例，请先完成一次计算并保存。</td>
                </tr>
              ) : null}
            </tbody>
          </table>
        </div>
        <CaseDetailPanel
          selectedCase={selectedCase}
          selectedDetail={selectedDetail}
          editName={editName}
          editNotes={editNotes}
          onEditNameChange={setEditName}
          onEditNotesChange={setEditNotes}
          onSave={() => void handleUpdate()}
          onOpenCalculation={() => void handleOpenForCalculation()}
        />
      </div>
    </section>
  );
}

function toErrorMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}
