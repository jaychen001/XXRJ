import { render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { invoke } from "@tauri-apps/api/core";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { setupAppInvokeMock } from "./app-test-fixtures";
import { App } from "./App";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

const invokeMock = vi.mocked(invoke);

describe("App desktop shell", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    setupAppInvokeMock(invokeMock);
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

  it("runs a calculation only after safety factor confirmation and saves the case", async () => {
    const user = userEvent.setup();
    render(<App />);
    await screen.findByText("本地数据正常");

    await user.click(screen.getByRole("button", { name: /选型计算/ }));
    expect(await screen.findByRole("heading", { name: "同步带基础计算" })).toBeInTheDocument();

    await user.type(screen.getByLabelText("搜索计算模块"), "气");
    expect(screen.getByText("气缸")).toBeInTheDocument();
    expect(screen.getByText("真空吸附")).toBeInTheDocument();
    expect(screen.getByText("电磁阀")).toBeInTheDocument();
    await user.clear(screen.getByLabelText("搜索计算模块"));

    await user.clear(screen.getByLabelText("摩擦系数"));
    await user.click(screen.getByRole("button", { name: "计算" }));
    expect(screen.getAllByText(/摩擦系数不能为空/).length).toBeGreaterThan(0);
    expect(screen.getByLabelText("摩擦系数")).toHaveFocus();
    await user.type(screen.getByLabelText("摩擦系数"), "0.1");

    await user.clear(screen.getByLabelText("目标速度"));
    await user.type(screen.getByLabelText("目标速度"), "0.5");
    await user.selectOptions(screen.getByLabelText("目标速度单位"), "m/s");

    await user.click(screen.getByRole("button", { name: "计算" }));
    expect(screen.getAllByText(/安全系数未确认/).length).toBeGreaterThan(0);
    expect(screen.getByLabelText("安全系数")).toHaveFocus();

    await user.click(screen.getByLabelText("我已确认安全系数"));
    await user.click(screen.getByRole("button", { name: "计算" }));

    expect(await screen.findByText("摩擦力")).toBeInTheDocument();
    expect(screen.getByText("速度区间")).toBeInTheDocument();
    expect(screen.getAllByText("根目录 PDF / 同步带匹配页").length).toBeGreaterThan(0);
    expect(screen.getByText(/输出扭矩 0.221 Nm/)).toBeInTheDocument();
    expect(invokeMock).toHaveBeenCalledWith(
      "run_calculation",
      expect.objectContaining({
        request: expect.objectContaining({
          fields: expect.arrayContaining([
            expect.objectContaining({ id: "targetSpeed", unit: "m/s" }),
          ]),
        }),
      }),
    );

    await user.click(screen.getByRole("button", { name: "保存案例" }));
    expect(await screen.findByText("案例已保存")).toBeInTheDocument();
    expect(invokeMock).toHaveBeenCalledWith(
      "save_calculation_case",
      expect.objectContaining({
        payload: expect.objectContaining({ name: "同步带计算案例" }),
      }),
    );
  });

  it("copies reruns and deletes a saved calculation case", async () => {
    const user = userEvent.setup();
    vi.spyOn(window, "confirm").mockReturnValue(true);
    render(<App />);
    await screen.findByText("本地数据正常");

    await user.click(screen.getByRole("button", { name: /案例库/ }));
    const table = await screen.findByRole("table", { name: "案例列表" });
    expect(within(table).getByText("同步带测试")).toBeInTheDocument();

    await user.selectOptions(screen.getByLabelText("模块筛选"), "timing-belt-basic");
    await user.type(screen.getByLabelText("开始日期"), "2026-07-03");
    await user.type(screen.getByLabelText("结束日期"), "2026-07-03");
    await user.click(screen.getByRole("button", { name: "搜索" }));
    expect(invokeMock).toHaveBeenCalledWith(
      "list_calculation_cases",
      expect.objectContaining({
        filter: expect.objectContaining({ moduleId: "timing-belt-basic" }),
      }),
    );

    await user.clear(screen.getByLabelText("详情案例名称"));
    await user.type(screen.getByLabelText("详情案例名称"), "同步带更新");
    await user.type(screen.getByLabelText("详情备注"), "复核后保存");
    await user.click(screen.getByRole("button", { name: "保存修改" }));
    expect(await screen.findByText("案例已更新")).toBeInTheDocument();
    expect(invokeMock).toHaveBeenCalledWith(
      "update_calculation_case",
      expect.objectContaining({
        payload: expect.objectContaining({ id: "case-1", name: "同步带更新" }),
      }),
    );

    await user.click(within(table).getAllByRole("button", { name: "重算" })[0]);
    expect(await screen.findByText(/已重新计算/)).toBeInTheDocument();

    await user.click(within(table).getByRole("button", { name: "复制" }));
    expect(await screen.findByRole("heading", { name: "同步带基础计算" })).toBeInTheDocument();
    expect(await screen.findByText(/正在修改：同步带更新 - 副本/)).toBeInTheDocument();

    await user.clear(screen.getByLabelText("目标速度"));
    await user.type(screen.getByLabelText("目标速度"), "0.4");
    await user.selectOptions(screen.getByLabelText("目标速度单位"), "m/s");
    await user.click(screen.getByRole("button", { name: "计算" }));
    expect(await screen.findByText("摩擦力")).toBeInTheDocument();

    await user.click(screen.getByRole("button", { name: "保存案例" }));
    expect(await screen.findByText("当前案例已更新")).toBeInTheDocument();
    expect(invokeMock).toHaveBeenCalledWith(
      "rerun_calculation_case_with_request",
      expect.objectContaining({
        id: "case-copy",
        request: expect.objectContaining({
          fields: expect.arrayContaining([
            expect.objectContaining({ id: "targetSpeed", value: 0.4, unit: "m/s" }),
          ]),
        }),
      }),
    );

    await user.click(screen.getByRole("button", { name: /案例库/ }));
    const refreshedTable = await screen.findByRole("table", { name: "案例列表" });
    await user.click(within(refreshedTable).getAllByRole("button", { name: "删除" })[0]);
    expect(invokeMock).toHaveBeenCalledWith("delete_calculation_case", { id: "case-copy" });
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
