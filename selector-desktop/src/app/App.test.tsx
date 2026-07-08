import { render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { invoke } from "@tauri-apps/api/core";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { calculationModules } from "./app-calculation-test-data";
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

  it("只保留中文选型计算入口并隐藏旧 PDF 工具页", async () => {
    render(<App />);

    expect(await screen.findByText("本地数据正常")).toBeInTheDocument();
    expect(screen.getByRole("heading", { name: "选型计算" })).toBeInTheDocument();

    const navigation = screen.getByRole("navigation", { name: "页面导航" });
    expect(within(navigation).getAllByRole("button")).toHaveLength(1);
    expect(within(navigation).getByRole("button", { name: "选型计算" })).toBeInTheDocument();

    expect(screen.queryByText(/PDF 覆盖矩阵/)).not.toBeInTheDocument();
    expect(screen.queryByText(/QA 覆盖检查/)).not.toBeInTheDocument();
    expect(screen.queryByText(/知识检索/)).not.toBeInTheDocument();
    expect(screen.queryByText(/内部参数库/)).not.toBeInTheDocument();
    expect(screen.queryByText(/^报告导出$/)).not.toBeInTheDocument();
    expect(screen.queryByText(/厂家样本库/)).not.toBeInTheDocument();
  });

  it("按中文工况输入完成计算并在结果页导出当前报告", async () => {
    const user = userEvent.setup();
    render(<App />);

    await screen.findByText("本地数据正常");
    expect(await screen.findByRole("heading", { name: "同步带基础计算" })).toBeInTheDocument();

    await user.clear(screen.getByLabelText("摩擦系数"));
    await user.click(screen.getByRole("button", { name: "计算" }));
    expect(screen.getAllByText(/摩擦系数不能为空/).length).toBeGreaterThan(0);
    expect(screen.getByLabelText("摩擦系数")).toHaveFocus();

    await user.type(screen.getByLabelText("摩擦系数"), "0.1");
    await user.clear(screen.getByLabelText("目标速度"));
    await user.type(screen.getByLabelText("目标速度"), "0.5");
    await user.selectOptions(screen.getByLabelText("目标速度单位"), "m/s");

    await user.click(screen.getByRole("button", { name: "计算" }));
    expect(screen.getAllByText(/请先输入并确认本次计算使用的安全系数/).length).toBeGreaterThan(0);
    expect(screen.getByLabelText("安全系数")).toHaveFocus();

    await user.click(screen.getByLabelText("我已确认本次计算使用的安全系数"));
    await user.click(screen.getByRole("button", { name: "计算" }));

    expect(await screen.findByText("摩擦力")).toBeInTheDocument();
    expect(screen.getByText("速度区间")).toBeInTheDocument();
    expect(screen.getByText(/输出扭矩 0.351 Nm/)).toBeInTheDocument();
    expect(screen.queryByText(/PDF P/)).not.toBeInTheDocument();
    expect(screen.queryByText(/timing-belt-basic@/)).not.toBeInTheDocument();

    const reportRegion = screen.getByRole("region", { name: "导出当前计算报告" });
    await user.selectOptions(within(reportRegion).getByLabelText("当前报告导出格式"), "xlsx");
    await user.type(within(reportRegion).getByLabelText("当前报告输出路径"), "D:\\reports\\同步带.xlsx");
    await user.click(within(reportRegion).getByRole("button", { name: "导出" }));

    expect(await screen.findByText("已导出：D:\\reports\\同步带.xlsx")).toBeInTheDocument();
    await waitFor(() => {
      expect(invokeMock).toHaveBeenCalledWith(
        "export_calculation_report",
        expect.objectContaining({
          request: expect.objectContaining({
            format: "xlsx",
            outputPath: "D:\\reports\\同步带.xlsx",
            caseId: null,
            finalModelName: null,
          }),
        }),
      );
    });
  });

  it("支持按计算对象搜索常用模块", async () => {
    const user = userEvent.setup();
    render(<App />);

    await screen.findByText("本地数据正常");
    await user.type(screen.getByLabelText("搜索计算对象"), "气");

    expect(screen.getByText("气缸")).toBeInTheDocument();
    expect(screen.getByText("真空吸附")).toBeInTheDocument();
    expect(screen.getByText("电磁阀")).toBeInTheDocument();
  });

  it("数据库检查失败时显示中文错误提示", async () => {
    invokeMock.mockImplementation((command) => {
      if (command === "get_database_health") {
        return Promise.reject(new Error("invoke unavailable"));
      }
      if (command === "list_calculation_modules") {
        return Promise.resolve(calculationModules);
      }
      return Promise.resolve(null);
    });

    render(<App />);

    expect(await screen.findByRole("alert")).toHaveTextContent("本地数据检查失败");
    expect(screen.getByText(/浏览器预览模式不能检查本地数据库/)).toBeInTheDocument();
  });
});
