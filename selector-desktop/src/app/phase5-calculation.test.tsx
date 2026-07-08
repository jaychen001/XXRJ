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

describe("Phase 5 mechanical transmission modules", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    setupAppInvokeMock(invokeMock);
  });

  it("opens mechanical chapter entry and runs gear calculation", async () => {
    const user = userEvent.setup();
    render(<App />);
    await screen.findByText("本地数据正常");

    await user.type(screen.getByLabelText("全局搜索"), "齿轮");
    const coverageTable = screen.getByRole("table", { name: "PDF 覆盖矩阵" });
    await user.click(within(coverageTable).getByRole("button", { name: "打开齿轮章节入口" }));
    expect(screen.getByRole("heading", { name: "齿轮 · 章节入口" })).toBeInTheDocument();
    expect(screen.getByText("齿轮 · 机械传动入口")).toBeInTheDocument();
    expect(screen.getAllByText("齿轮参数计算").length).toBeGreaterThan(0);
    expect(screen.getAllByText("PDF P44 / 文档页 41 / 齿轮").length).toBeGreaterThan(0);

    await user.click(screen.getByRole("button", { name: /选型计算/ }));
    await user.type(screen.getByLabelText("搜索计算模块"), "齿轮");
    await user.click(screen.getByRole("button", { name: /齿轮参数计算/ }));
    await user.click(screen.getByLabelText("我已确认安全系数"));
    await user.click(screen.getByRole("button", { name: "计算" }));

    expect((await screen.findAllByText("中心距")).length).toBeGreaterThan(0);
    expect(screen.getAllByText("减速比").length).toBeGreaterThan(0);
    expect(screen.getByText(/中心距 80.000 mm/)).toBeInTheDocument();
    expect(screen.getAllByText("PDF P44 / 文档页 41 / 齿轮").length).toBeGreaterThan(0);

    await waitFor(() => {
      expect(invokeMock).toHaveBeenCalledWith(
        "run_calculation",
        expect.objectContaining({
          request: expect.objectContaining({ moduleId: "gear-basic" }),
        }),
      );
    });
  });
});
