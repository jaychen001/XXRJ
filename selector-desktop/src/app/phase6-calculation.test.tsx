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

describe("Phase 6 pneumatic and support modules", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    setupAppInvokeMock(invokeMock);
  });

  it("opens pneumatic chapter entry and runs cylinder calculation", async () => {
    const user = userEvent.setup();
    render(<App />);
    await screen.findByText("本地数据正常");

    await user.type(screen.getByLabelText("全局搜索"), "气动执行");
    const coverageTable = screen.getByRole("table", { name: "PDF 覆盖矩阵" });
    await user.click(within(coverageTable).getByRole("button", { name: "打开气动执行元件章节入口" }));
    expect(screen.getByRole("heading", { name: "气动执行元件 · 章节入口" })).toBeInTheDocument();
    expect(screen.getByText("气动执行元件 · 气动与支撑入口")).toBeInTheDocument();
    expect(screen.getAllByText("气缸").length).toBeGreaterThan(0);
    expect(screen.getByText("真空吸附")).toBeInTheDocument();
    expect(screen.getAllByText("PDF P69 / 文档页 66 / 气动执行元件").length).toBeGreaterThan(0);

    await user.click(screen.getByRole("button", { name: /选型计算/ }));
    await user.type(screen.getByLabelText("搜索计算模块"), "气缸");
    await user.click(screen.getAllByRole("button", { name: /气缸/ })[0]);
    await user.click(screen.getByLabelText("我已确认安全系数"));
    await user.click(screen.getByRole("button", { name: "计算" }));

    expect((await screen.findAllByText("负载率修正")).length).toBeGreaterThan(0);
    expect(screen.getAllByText("选型输出力").length).toBeGreaterThan(0);
    expect(screen.getByText(/选型输出力 92.302 N/)).toBeInTheDocument();
    expect(screen.getAllByText("PDF P69 / 文档页 66 / 气动执行元件").length).toBeGreaterThan(0);

    await waitFor(() => {
      expect(invokeMock).toHaveBeenCalledWith(
        "run_calculation",
        expect.objectContaining({
          request: expect.objectContaining({ moduleId: "pneumatic-cylinder-sizing" }),
        }),
      );
    });
  });
});
