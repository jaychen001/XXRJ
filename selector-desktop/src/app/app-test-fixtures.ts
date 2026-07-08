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
  let vendorLibraries = [vendorLibraryRecord("vendor-lib-1", "伺服样本库")];
  let vendorModels = [
    vendorModel("vendor-model-1", "vendor-lib-1", "SV-400", 0.5, 3000),
    vendorModel("vendor-model-2", "vendor-lib-1", "SV-750", 0.9, 3000),
  ];
  let reportExports: ReturnType<typeof reportExportRecord>[] = [];

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
    if (command === "preview_vendor_import") {
      return Promise.resolve(vendorPreview());
    }
    if (command === "confirm_vendor_import") {
      const request = getMockObjectArg(args, "request");
      if (!getMockBooleanField(request, "confirmed")) {
        return Promise.reject(new Error("字段映射未经人工确认"));
      }
      const id = `vendor-lib-${vendorLibraries.length + 1}`;
      const library = vendorLibraryRecord(id, getMockStringField(request, "libraryName") || "导入样本库");
      vendorLibraries = [library, ...vendorLibraries];
      vendorModels = [
        vendorModel(`vendor-model-${vendorModels.length + 1}`, id, "SV-400", 0.5, 3000),
        vendorModel(`vendor-model-${vendorModels.length + 2}`, id, "SV-750", 0.9, 3000),
        ...vendorModels,
      ];
      return Promise.resolve({
        library,
        importedModels: 2,
        failedRows: 0,
        jobId: "vendor-import-preview",
      });
    }
    if (command === "list_vendor_libraries") {
      return Promise.resolve(vendorLibraries);
    }
    if (command === "list_vendor_models") {
      const libraryId = getMockArg(args, "libraryId");
      return Promise.resolve(
        libraryId ? vendorModels.filter((item) => item.libraryId === libraryId) : vendorModels,
      );
    }
    if (command === "set_vendor_library_enabled") {
      const id = getMockArg(args, "id");
      const enabled = getMockBooleanArg(args, "enabled");
      vendorLibraries = vendorLibraries.map((item) =>
        item.id === id ? { ...item, enabled } : item,
      );
      return Promise.resolve(vendorLibraries.find((item) => item.id === id) ?? null);
    }
    if (command === "delete_vendor_library") {
      const id = getMockArg(args, "id");
      vendorLibraries = vendorLibraries.filter((item) => item.id !== id);
      vendorModels = vendorModels.filter((item) => item.libraryId !== id);
      return Promise.resolve(true);
    }
    if (command === "recommend_vendor_models") {
      return Promise.resolve(vendorModels.slice(0, 2).map(recommendationCandidate));
    }
    if (command === "export_calculation_report") {
      const request = getMockObjectArg(args, "request");
      const record = reportExportRecord("report-current", getMockStringField(request, "format"), getMockStringField(request, "outputPath"));
      return Promise.resolve(record);
    }
    if (command === "export_case_report") {
      const request = getMockObjectArg(args, "request");
      const record = reportExportRecord(
        `report-${reportExports.length + 1}`,
        getMockStringField(request, "format"),
        getMockStringField(request, "outputPath"),
      );
      reportExports = [record, ...reportExports];
      return Promise.resolve(record);
    }
    if (command === "list_report_exports") {
      return Promise.resolve(reportExports);
    }
    if (command === "get_qa_coverage_audit") {
      return Promise.resolve(qaCoverageAudit());
    }
    if (command === "run_qa_regression") {
      return Promise.resolve(qaRegressionReport());
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

function getMockBooleanArg(args: unknown, key: string): boolean {
  if (typeof args !== "object" || args === null || Array.isArray(args)) {
    return false;
  }
  const value = (args as Record<string, unknown>)[key];
  return typeof value === "boolean" ? value : false;
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

function getMockBooleanField(record: Record<string, unknown>, key: string): boolean {
  return typeof record[key] === "boolean" ? record[key] : false;
}

function getRequestModuleId(args: unknown): string {
  return getMockStringField(getMockObjectArg(args, "request"), "moduleId");
}

function vendorPreview() {
  return {
    jobId: "vendor-import-preview",
    sourceFile: "D:\\samples\\servo.csv",
    sourceFormat: "csv",
    confidence: 0.86,
    rows: [
      previewRow(1, "SV-400", 0.5, 3000),
      previewRow(2, "SV-750", 0.9, 3000),
    ],
    failedRows: [],
    suggestedMappings: [
      { sourceField: "modelName", targetField: "modelName", unit: null },
      { sourceField: "额定扭矩(Nm)", targetField: "outputTorque", unit: "Nm" },
      { sourceField: "额定转速(rpm)", targetField: "requiredSpeed", unit: "rpm" },
    ],
    warnings: [],
  };
}

function previewRow(rowIndex: number, modelName: string, torque: number, speed: number) {
  return {
    rowIndex,
    modelName,
    brand: "ACME",
    series: "SV",
    sourcePage: null,
    confidence: 0.86,
    rawText: `${modelName},${torque},${speed}`,
    parameters: [
      parameter("outputTorque", "输出扭矩", torque, "Nm", "额定扭矩(Nm)"),
      parameter("requiredSpeed", "需求转速", speed, "rpm", "额定转速(rpm)"),
    ],
  };
}

function vendorLibraryRecord(id: string, name: string) {
  return {
    id,
    name,
    componentType: "伺服/步进电机",
    sourceFile: "D:\\samples\\servo.csv",
    sourceFormat: "csv",
    versionName: "v1",
    importedAt: "2026-07-08 10:00:00",
    enabled: true,
    modelCount: 2,
    createdAt: "2026-07-08 10:00:00",
    updatedAt: "2026-07-08 10:00:00",
  };
}

function vendorModel(
  id: string,
  libraryId: string,
  modelName: string,
  torque: number,
  speed: number,
) {
  return {
    id,
    libraryId,
    libraryName: "伺服样本库",
    componentType: "伺服/步进电机",
    modelName,
    brand: "ACME",
    series: "SV",
    parameters: [
      parameter("outputTorque", "输出扭矩", torque, "Nm", "额定扭矩(Nm)"),
      parameter("requiredSpeed", "需求转速", speed, "rpm", "额定转速(rpm)"),
    ],
    normalizedParameters: {
      outputTorque: {
        label: "输出扭矩",
        value: torque,
        unit: "Nm",
        sourceField: "额定扭矩(Nm)",
      },
      requiredSpeed: {
        label: "需求转速",
        value: speed,
        unit: "rpm",
        sourceField: "额定转速(rpm)",
      },
    },
    sourcePage: null,
    enabled: true,
  };
}

function parameter(
  field: string,
  label: string,
  value: number,
  unit: string,
  sourceField: string,
) {
  return { field, label, value, unit, sourceField };
}

function recommendationCandidate(model: ReturnType<typeof vendorModel>) {
  return {
    model,
    score: model.modelName === "SV-400" ? 1 : 0.8,
    matchedRules: [
      {
        requirementId: "outputTorque",
        label: "输出扭矩",
        message: `${model.modelName} 输出扭矩满足`,
        requiredValue: 0.351,
        candidateValue: model.normalizedParameters.outputTorque.value,
        unit: "Nm",
      },
      {
        requirementId: "requiredSpeed",
        label: "需求转速",
        message: `${model.modelName} 转速满足`,
        requiredValue: 300,
        candidateValue: model.normalizedParameters.requiredSpeed.value,
        unit: "rpm",
      },
    ],
    failedRules: [],
  };
}

function reportExportRecord(id: string, format: string, path: string) {
  return {
    id,
    caseId: id === "report-current" ? null : "case-1",
    runId: id === "report-current" ? null : "run-1",
    format: format || "pdf",
    path: path || "selector-report.pdf",
    exportedAt: "2026-07-08 11:30:00",
  };
}

function qaCoverageAudit() {
  return {
    status: "pass",
    totalChapters: 23,
    doneChapters: 23,
    missingChapters: [],
    checks: [
      {
        label: "PDF 23 章覆盖",
        passed: true,
        detail: "已完成 23/23 章",
      },
      {
        label: "报告导出能力",
        passed: true,
        detail: "PDF 与 Excel 导出命令已注册",
      },
      {
        label: "来源页码追溯",
        passed: true,
        detail: "每章都有实现来源页",
      },
    ],
    items: coverageRecords.map((record) => ({
      id: record.id,
      chapter: record.chapter,
      status: "done",
      sourcePage: record.sourcePageRange,
      implementationShape: record.implementationShape,
    })),
  };
}

function qaRegressionReport() {
  return {
    status: "pass",
    totalCases: 4,
    passedCases: 4,
    failedCases: 0,
    groups: [
      regressionGroup("Phase 4 驱动与线性传动", "同步带回归样例", "timing-belt"),
      regressionGroup("Phase 5 机械传动与间歇机构", "V 带回归样例", "v-belt"),
      regressionGroup("Phase 6 气动与支撑件", "气缸回归样例", "cylinder"),
      regressionGroup("Phase 7 规则选型模块", "机器人回归样例", "robot"),
    ],
  };
}

function regressionGroup(label: string, name: string, moduleId: string) {
  return {
    label,
    totalCases: 1,
    passedCases: 1,
    failedCases: 0,
    cases: [
      {
        name,
        moduleId,
        status: "pass",
        detail: "步骤 3 项，规则 3 项，来源 1 项",
      },
    ],
  };
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
