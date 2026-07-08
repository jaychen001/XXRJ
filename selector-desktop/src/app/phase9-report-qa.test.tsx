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

describe("Phase 9 report export and QA", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    setupAppInvokeMock(invokeMock);
  });

  it("exports a saved case report and records the output path", async () => {
    const user = userEvent.setup();
    render(<App />);

    await user.click(await screen.findByRole("button", { name: /报告导出/ }));
    expect(screen.getByRole("heading", { name: "报告导出" })).toBeInTheDocument();

    await user.selectOptions(screen.getByLabelText("报告导出格式"), "xlsx");
    await user.type(screen.getByLabelText("报告输出路径"), "D:\\reports\\case.xlsx");
    await user.click(screen.getByRole("button", { name: /导出报告/ }));

    expect(await screen.findByText("已导出：D:\\reports\\case.xlsx")).toBeInTheDocument();
    const table = screen.getByRole("table", { name: "报告导出记录表" });
    expect(within(table).getByText("D:\\reports\\case.xlsx")).toBeInTheDocument();
    expect(invokeMock).toHaveBeenCalledWith(
      "export_case_report",
      expect.objectContaining({
        request: expect.objectContaining({
          caseId: "case-1",
          format: "xlsx",
          outputPath: "D:\\reports\\case.xlsx",
        }),
      }),
    );
  });

  it("exports the current calculation result and includes recommended candidates", async () => {
    const user = userEvent.setup();
    render(<App />);

    await user.click(await screen.findByRole("button", { name: /选型计算/ }));
    await user.click(screen.getByLabelText("我已确认安全系数"));
    await user.click(screen.getByRole("button", { name: "计算" }));
    await user.click(await screen.findByRole("button", { name: /匹配型号/ }));
    expect(await screen.findByText("匹配到 2 个候选型号")).toBeInTheDocument();

    await user.type(screen.getByLabelText("当前报告输出路径"), "current.pdf");
    await user.selectOptions(screen.getByLabelText("最终选择型号"), "SV-400");
    const reportRegion = screen.getByRole("region", { name: "导出当前计算报告" });
    await user.click(within(reportRegion).getByRole("button", { name: /导出/ }));

    expect(await screen.findByText("已导出：current.pdf")).toBeInTheDocument();
    await waitFor(() => {
      expect(invokeMock).toHaveBeenCalledWith(
        "export_calculation_report",
        expect.objectContaining({
          request: expect.objectContaining({
            outputPath: "current.pdf",
            candidates: expect.arrayContaining([
              expect.objectContaining({
                model: expect.objectContaining({ modelName: "SV-400" }),
              }),
            ]),
            finalModelName: "SV-400",
          }),
        }),
      );
    });
  });

  it("shows QA audit status for all 23 PDF chapters", async () => {
    const user = userEvent.setup();
    render(<App />);

    await user.click(await screen.findByRole("button", { name: /QA 覆盖检查/ }));
    expect(screen.getByRole("heading", { name: "QA 覆盖检查" })).toBeInTheDocument();
    expect(await screen.findByText("QA 覆盖检查通过")).toBeInTheDocument();
    expect(screen.getAllByText("23").length).toBeGreaterThan(0);
    expect(screen.getByText("PDF 23 章覆盖")).toBeInTheDocument();
    expect(invokeMock).toHaveBeenCalledWith("get_qa_coverage_audit");
  });
});
