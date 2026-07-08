import { COVERAGE_ITEMS } from "../domain/coverage";
import {
  calculationModules,
  calculationRequest,
  calculationResult,
  calculationResultForModule,
  initialCaseRecord,
  type CalculationRequestFixture,
} from "./app-calculation-test-data";

type InvokeMock = {
  mockImplementation: (
    implementation: (command: string, args?: unknown) => Promise<unknown>,
  ) => void;
};

export const okHealth = {
  status: "ok",
  databasePath: "test-selector.db",
  appliedMigrations: 1,
  tableCount: 18,
  message: "数据库可用",
};

export const coverageRecords = COVERAGE_ITEMS.map((item, index) => ({
  id: item.id,
  chapter: item.chapter,
  implementationShape: item.shape,
  status: item.status,
  sourcePageRange: item.source.startsWith("PDF P") ? item.source : `P${index + 1}`,
  catalogPage: `P${index + 1}`,
  catalogExcerpt: `${item.chapter} PDF 目录摘录`,
  knowledgeEntryCount: 2,
  notes: item.requirement,
}));

const ingestSummary = {
  pdfPath: "root-note.pdf",
  pageCount: 120,
  catalogCount: 23,
  coverageCount: 23,
  knowledgeEntryCount: 46,
  parameterCandidateCount: 3,
};

export function setupAppInvokeMock(invokeMock: InvokeMock) {
  let hasKnowledgeIndex = false;
  let candidateStatus: "pending" | "confirmed" | "ignored" = "pending";
  let caseRecords = [initialCaseRecord];
  let caseRequests: Record<string, CalculationRequestFixture> = {
    [initialCaseRecord.id]: calculationRequest,
  };

  invokeMock.mockImplementation((command, args) => {
    if (command === "get_database_health") {
      return Promise.resolve(okHealth);
    }
    if (command === "get_pdf_coverage_items") {
      return Promise.resolve(coverageRecords);
    }
    if (command === "ingest_root_pdf_note") {
      hasKnowledgeIndex = true;
      return Promise.resolve(ingestSummary);
    }
    if (command === "list_recent_knowledge_entries") {
      return Promise.resolve(hasKnowledgeIndex ? [recentKnowledgeEntry] : []);
    }
    if (command === "search_knowledge_entries") {
      return Promise.resolve([knowledgeSearchResult(getMockArg(args, "query"))]);
    }
    if (command === "list_parameter_candidates") {
      return Promise.resolve([parameterCandidate(candidateStatus)]);
    }
    if (command === "update_parameter_candidate_status") {
      candidateStatus = getMockArg(args, "status") as "confirmed" | "ignored";
      return Promise.resolve(parameterCandidate(candidateStatus));
    }
    if (command === "list_calculation_modules") {
      return Promise.resolve(calculationModules);
    }
    if (command === "run_calculation") {
      return Promise.resolve(calculationResultForModule(getRequestModuleId(args)));
    }
    if (command === "save_calculation_case") {
      const payload = getMockObjectArg(args, "payload");
      const saved = { ...initialCaseRecord, id: "case-saved", name: "同步带计算案例" };
      caseRecords = [saved, ...caseRecords];
      caseRequests[saved.id] =
        (payload.request as CalculationRequestFixture | undefined) ?? calculationRequest;
      return Promise.resolve({
        caseRecord: saved,
        result: calculationResultForModule(getMockStringField(getMockObjectArg(payload, "request"), "moduleId")),
      });
    }
    if (command === "list_calculation_cases") {
      const filter = getMockObjectArg(args, "filter");
      const query = getMockStringField(filter, "query");
      const moduleId = getMockStringField(filter, "moduleId");
      return Promise.resolve(
        caseRecords.filter((item) => {
          const queryMatches = query
            ? item.name.includes(query) || item.moduleName.includes(query)
            : true;
          const moduleMatches = moduleId ? item.moduleId === moduleId : true;
          return queryMatches && moduleMatches;
        }),
      );
    }
    if (command === "update_calculation_case") {
      const payload = getMockObjectArg(args, "payload");
      const id = getMockStringField(payload, "id");
      const name = getMockStringField(payload, "name");
      const notes = getMockStringField(payload, "notes");
      caseRecords = caseRecords.map((item) =>
        item.id === id ? { ...item, name, notes, updatedAt: "2026-07-03 16:00:00" } : item,
      );
      return Promise.resolve(caseRecords.find((item) => item.id === id) ?? null);
    }
    if (command === "duplicate_calculation_case") {
      const id = getMockArg(args, "id");
      const source = caseRecords.find((item) => item.id === id) ?? initialCaseRecord;
      const copy = { ...source, id: "case-copy", name: `${source.name} - 副本` };
      caseRecords = [copy, ...caseRecords];
      caseRequests[copy.id] = caseRequests[id] ?? calculationRequest;
      return Promise.resolve(copy);
    }
    if (command === "get_calculation_case_detail") {
      const id = getMockArg(args, "id");
      const caseRecord = caseRecords.find((item) => item.id === id) ?? initialCaseRecord;
      return Promise.resolve({
        caseRecord,
        request: caseRequests[id] ?? calculationRequest,
        result: calculationResult,
      });
    }
    if (command === "rerun_calculation_case") {
      return Promise.resolve({ caseRecord: caseRecords[0], result: calculationResult });
    }
    if (command === "rerun_calculation_case_with_request") {
      const id = getMockArg(args, "id");
      const request = getMockObjectArg(args, "request") as unknown as CalculationRequestFixture;
      caseRequests[id] = request;
      return Promise.resolve({
        caseRecord: caseRecords.find((item) => item.id === id) ?? caseRecords[0],
        result: calculationResultForModule(request.moduleId),
      });
    }
    if (command === "delete_calculation_case") {
      const id = getMockArg(args, "id");
      caseRecords = caseRecords.filter((item) => item.id !== id);
      return Promise.resolve(true);
    }
    return Promise.resolve(null);
  });
}

function getMockArg(args: unknown, key: string): string {
  if (typeof args !== "object" || args === null || Array.isArray(args)) {
    return "";
  }
  const value = (args as Record<string, unknown>)[key];
  return typeof value === "string" ? value : "";
}

function getMockObjectArg(args: unknown, key: string): Record<string, unknown> {
  if (typeof args !== "object" || args === null || Array.isArray(args)) {
    return {};
  }
  const value = (args as Record<string, unknown>)[key];
  return typeof value === "object" && value !== null && !Array.isArray(value)
    ? (value as Record<string, unknown>)
    : {};
}

function getMockStringField(record: Record<string, unknown>, key: string): string {
  return typeof record[key] === "string" ? record[key] : "";
}

function getRequestModuleId(args: unknown): string {
  return getMockStringField(getMockObjectArg(args, "request"), "moduleId");
}

function knowledgeSearchResult(query: string) {
  const page = query === "摩擦系数" ? "P24" : query === "负载率" ? "P64" : "P8";
  return {
    id: `keyword-${query}`,
    title: `${query} / 来源 ${page}`,
    content: `${query} 对应 PDF 片段。`,
    page,
    tags: ["PDF知识检索", query],
    sourceTitle: "根目录非标笔记 PDF",
  };
}

const recentKnowledgeEntry = {
  id: "chapter-motor-p3",
  title: "电机篇 / 来源 P3",
  content: "电机篇 PDF 目录摘录",
  page: "P3",
  tags: ["PDF章节", "电机篇"],
  sourceTitle: "根目录非标笔记 PDF",
};

function parameterCandidate(status: "pending" | "confirmed" | "ignored") {
  return {
    id: "root-pdf-摩擦系数",
    name: "摩擦系数",
    value: "0.1",
    unit: null,
    scenario: "同步带摩擦系数候选，需要人工确认。",
    sourcePage: "P24",
    status,
  };
}
