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

describe("Phase 7 rule modules", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    setupAppInvokeMock(invokeMock);
  });

  it("opens rule chapter entry and runs robot rule selector", async () => {
    const user = userEvent.setup();
    render(<App />);
    await screen.findByText("本地数据正常");

    await user.type(screen.getByLabelText("全局搜索"), "机器人");
    const coverageTable = screen.getByRole("table", { name: "PDF 覆盖矩阵" });
    await user.click(within(coverageTable).getByRole("button", { name: "打开机器人章节入口" }));
    expect(screen.getByRole("heading", { name: "机器人 · 章节入口" })).toBeInTheDocument();
    expect(screen.getByText("机器人 · 规则选型入口")).toBeInTheDocument();
    expect(screen.getByText("负载")).toBeInTheDocument();
    expect(screen.getAllByText("PDF P67 / 文档页 64 / 机器人").length).toBeGreaterThan(0);

    await user.click(screen.getByRole("button", { name: /选型计算/ }));
    await user.type(screen.getByLabelText("搜索计算模块"), "机器人");
    await user.click(screen.getByRole("button", { name: /机器人规则选型/ }));
    await user.click(screen.getByLabelText("我已确认安全系数"));
    await user.click(screen.getByRole("button", { name: "计算" }));

    expect((await screen.findAllByText("推荐类型")).length).toBeGreaterThan(0);
    expect(screen.getByText("节拍判断")).toBeInTheDocument();
    expect(screen.getByText("精度风险")).toBeInTheDocument();
    expect(screen.getByText(/建议 SCARA 或小型六轴机器人/)).toBeInTheDocument();

    await waitFor(() => {
      expect(invokeMock).toHaveBeenCalledWith(
        "run_calculation",
        expect.objectContaining({
          request: expect.objectContaining({ moduleId: "robot-rule-selector" }),
        }),
      );
    });
  });
});
