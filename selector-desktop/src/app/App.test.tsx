import { render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { invoke } from "@tauri-apps/api/core";
import { beforeEach, describe, expect, it, vi } from "vitest";
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

describe("App Phase 1 shell", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    invokeMock.mockResolvedValue(okHealth);
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
    await user.click(screen.getByRole("button", { name: "刷新覆盖矩阵" }));

    await waitFor(() => {
      expect(refreshMetric?.textContent).not.toEqual(beforeRefresh);
    });
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
});
