import { render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { invoke } from "@tauri-apps/api/core";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { setupAppInvokeMock } from "./app-test-fixtures";
import { App } from "./App";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

const invokeMock = vi.mocked(invoke);

describe("报告导出和 QA 可见边界", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    setupAppInvokeMock(invokeMock);
  });

  it("报告导出只在当前计算结果页出现", async () => {
    const user = userEvent.setup();
    render(<App />);

    await screen.findByText("本地数据正常");
    expect(screen.queryByRole("button", { name: /^报告导出$/ })).not.toBeInTheDocument();

    await user.click(screen.getByLabelText("我已确认本次计算使用的安全系数"));
    await user.click(screen.getByRole("button", { name: "计算" }));

    const reportRegion = await screen.findByRole("region", { name: "导出当前计算报告" });
    expect(within(reportRegion).getByText("导出报告")).toBeInTheDocument();

    await user.type(within(reportRegion).getByLabelText("当前报告输出路径"), "current.pdf");
    await user.click(within(reportRegion).getByRole("button", { name: "导出" }));

    expect(await screen.findByText("已导出：current.pdf")).toBeInTheDocument();
    expect(invokeMock).toHaveBeenCalledWith(
      "export_calculation_report",
      expect.objectContaining({
        request: expect.objectContaining({
          outputPath: "current.pdf",
          candidates: [],
          finalModelName: null,
        }),
      }),
    );
  });

  it("不在用户主界面暴露 QA 覆盖检查", async () => {
    render(<App />);

    await screen.findByText("本地数据正常");
    expect(screen.queryByRole("button", { name: /QA/ })).not.toBeInTheDocument();
    expect(screen.queryByText(/覆盖检查/)).not.toBeInTheDocument();
    expect(screen.queryByText(/PDF 章节/)).not.toBeInTheDocument();
  });
});
