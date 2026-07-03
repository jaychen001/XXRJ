import { render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { invoke } from "@tauri-apps/api/core";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { COVERAGE_ITEMS } from "../domain/coverage";
import { App } from "./App";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

const invokeMock = vi.mocked(invoke);

const okHealth = {
  status: "ok",
  databasePath: "test-selector.db",
  appliedMigrations: 1,
  tableCount: 18,
  message: "数据库可用",
};

const coverageRecords = COVERAGE_ITEMS.map((item, index) => ({
  id: item.id,
  chapter: item.chapter,
  implementationShape: item.shape,
  status: "partial",
  sourcePageRange: `P${index + 1}`,
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

function getMockArg(args: unknown, key: string): string {
  if (typeof args !== "object" || args === null || Array.isArray(args)) {
    return "";
  }

  const value = (args as Record<string, unknown>)[key];
  return typeof value === "string" ? value : "";
}

describe("App desktop shell", () => {
  let hasKnowledgeIndex = false;
  let candidateStatus: "pending" | "confirmed" | "ignored" = "pending";

  beforeEach(() => {
    vi.clearAllMocks();
    hasKnowledgeIndex = false;
    candidateStatus = "pending";
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
        return Promise.resolve(
          hasKnowledgeIndex
            ? [
                {
                  id: "chapter-motor-p3",
                  title: "电机篇 / 来源 P3",
                  content: "电机篇 PDF 目录摘录",
                  page: "P3",
                  tags: ["PDF章节", "电机篇"],
                  sourceTitle: "根目录非标笔记 PDF",
                },
              ]
            : [],
        );
      }
      if (command === "search_knowledge_entries") {
        const query = getMockArg(args, "query");
        const page = query === "摩擦系数" ? "P24" : query === "负载率" ? "P64" : "P8";
        return Promise.resolve([
          {
            id: `keyword-${query}`,
            title: `${query} / 来源 ${page}`,
            content: `${query} 对应 PDF 片段。`,
            page,
            tags: ["PDF知识检索", query],
            sourceTitle: "根目录非标笔记 PDF",
          },
        ]);
      }
      if (command === "list_parameter_candidates") {
        return Promise.resolve([
          {
            id: "root-pdf-摩擦系数",
            name: "摩擦系数",
            value: "0.1",
            unit: null,
            scenario: "同步带摩擦系数候选，需要人工确认。",
            sourcePage: "P24",
            status: candidateStatus,
          },
        ]);
      }
      if (command === "update_parameter_candidate_status") {
        candidateStatus = getMockArg(args, "status") as "confirmed" | "ignored";
        return Promise.resolve({
          id: getMockArg(args, "id"),
          name: "摩擦系数",
          value: "0.1",
          unit: null,
          scenario: "同步带摩擦系数候选，需要人工确认。",
          sourcePage: "P24",
          status: candidateStatus,
        });
      }
      return Promise.resolve(null);
    });
  });

  it("renders 23 PDF chapter entries and filters the matrix from global search", async () => {
    const user = userEvent.setup();
    render(<App />);

    expect(await screen.findByText("本地数据正常")).toBeInTheDocument();

    const chapterNav = screen.getByRole("navigation", { name: "PDF 章节导航" });
    expect(within(chapterNav).getAllByRole("button")).toHaveLength(23);

    await user.type(screen.getByLabelText("全局搜索"), "滚珠");

    const table = screen.getByRole("table", { name: "PDF 覆盖矩阵" });
    expect(within(table).getByText("丝杆篇")).toBeInTheDocument();
    expect(within(table).queryByText("电机篇")).not.toBeInTheDocument();
    expect(within(chapterNav).getAllByRole("button")).toHaveLength(1);
  });

  it("focuses search with Ctrl+K and refreshes the visible timestamp", async () => {
    const user = userEvent.setup();
    render(<App />);
    await screen.findByText("本地数据正常");

    const searchInput = screen.getByLabelText("全局搜索");
    await user.keyboard("{Control>}k{/Control}");
    expect(searchInput).toHaveFocus();

    const refreshMetric = screen.getByText("最近刷新").closest(".summary-metric");
    expect(refreshMetric).not.toBeNull();
    const beforeRefresh = refreshMetric?.textContent;

    await new Promise((resolve) => window.setTimeout(resolve, 1100));
    await user.click(screen.getByRole("button", { name: "导入/刷新 PDF 索引" }));

    await waitFor(() => {
      expect(refreshMetric?.textContent).not.toEqual(beforeRefresh);
    });
    expect(invokeMock).toHaveBeenCalledWith("ingest_root_pdf_note");
  });

  it("opens chapter detail from row actions and keeps placeholder pages user-facing", async () => {
    const user = userEvent.setup();
    render(<App />);
    await screen.findByText("本地数据正常");

    const table = screen.getByRole("table", { name: "PDF 覆盖矩阵" });
    await user.click(within(table).getByRole("button", { name: "打开同步带章节入口" }));
    expect(screen.getByRole("heading", { name: "同步带 · 章节入口" })).toBeInTheDocument();

    await user.click(screen.getByRole("button", { name: /案例库/ }));
    expect(screen.getByRole("heading", { name: "案例库" })).toBeInTheDocument();
    expect(screen.queryByRole("heading", { name: "cases" })).not.toBeInTheDocument();
  });

  it("collapses and expands the left navigation and trace panel", async () => {
    const user = userEvent.setup();
    render(<App />);
    await screen.findByText("本地数据正常");

    await user.click(screen.getByRole("button", { name: "折叠左侧导航" }));
    expect(screen.getByRole("button", { name: "展开左侧导航" })).toBeInTheDocument();

    await user.click(screen.getByRole("button", { name: "折叠追溯区" }));
    expect(screen.getByRole("button", { name: "展开追溯区" })).toBeInTheDocument();
  });

  it("shows a global alert when the database health check fails", async () => {
    invokeMock.mockRejectedValueOnce(new Error("invoke unavailable"));

    render(<App />);

    expect(await screen.findByRole("alert")).toHaveTextContent("数据库检查失败");
    expect(screen.getByText(/浏览器预览模式/)).toBeInTheDocument();
  });

  it("disables PDF knowledge search until indexing and searches all acceptance keywords", async () => {
    const user = userEvent.setup();
    render(<App />);
    await screen.findByText("本地数据正常");

    await user.click(screen.getByRole("button", { name: /知识检索/ }));
    expect(screen.getByRole("button", { name: "搜索" })).toBeDisabled();

    await user.click(screen.getByRole("button", { name: "建立/刷新索引" }));
    expect(await screen.findByText(/索引完成：120 页/)).toBeInTheDocument();

    const searchButton = screen.getByRole("button", { name: "搜索" });
    const searchInput = screen.getByLabelText("知识检索词");
    await user.click(searchButton);
    expect(await screen.findByText("惯量比 / 来源 P8")).toBeInTheDocument();
    expect(screen.getByText("P8")).toBeInTheDocument();

    await user.clear(searchInput);
    await user.type(searchInput, "摩擦系数");
    await user.click(searchButton);
    expect(await screen.findByText("摩擦系数 / 来源 P24")).toBeInTheDocument();

    await user.clear(searchInput);
    await user.type(searchInput, "负载率");
    await user.click(searchButton);
    expect(await screen.findByText("负载率 / 来源 P64")).toBeInTheDocument();
  });

  it("loads PDF parameter candidates and confirms one into the parameter library", async () => {
    const user = userEvent.setup();
    render(<App />);
    await screen.findByText("本地数据正常");

    await user.click(screen.getByRole("button", { name: /内部参数库/ }));
    expect(await screen.findByText("摩擦系数")).toBeInTheDocument();

    await user.click(screen.getByRole("button", { name: "确认摩擦系数" }));
    expect(invokeMock).toHaveBeenCalledWith("update_parameter_candidate_status", {
      id: "root-pdf-摩擦系数",
      status: "confirmed",
    });
  });

  it("ignores a PDF parameter candidate without confirming it", async () => {
    const user = userEvent.setup();
    render(<App />);
    await screen.findByText("本地数据正常");

    await user.click(screen.getByRole("button", { name: /内部参数库/ }));
    expect(await screen.findByText("摩擦系数")).toBeInTheDocument();

    await user.click(screen.getByRole("button", { name: "忽略摩擦系数" }));
    expect(invokeMock).toHaveBeenCalledWith("update_parameter_candidate_status", {
      id: "root-pdf-摩擦系数",
      status: "ignored",
    });
  });
});
