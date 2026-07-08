import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { invoke } from "@tauri-apps/api/core";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { setupAppInvokeMock } from "./app-test-fixtures";
import { App } from "./App";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

const invokeMock = vi.mocked(invoke);

describe("Phase 4 drive and transmission modules", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    setupAppInvokeMock(invokeMock);
  });

  it("lists implemented Phase 4 modules and renders ball screw calculation steps", async () => {
    const user = userEvent.setup();
    render(<App />);
    await screen.findByText("本地数据正常");

    expect(screen.getByText("Phase 5")).toBeInTheDocument();
    expect(screen.getAllByText("已实现").length).toBeGreaterThanOrEqual(5);
    expect(screen.getByText("计算项")).toBeInTheDocument();
    expect(screen.getByText("规则项")).toBeInTheDocument();
    expect(screen.getByText("知识引用")).toBeInTheDocument();
    expect(screen.getByText("通用电机功率计算")).toBeInTheDocument();
    expect(screen.getAllByText("PDF P4 / 文档页 1 / 电机篇").length).toBeGreaterThan(0);

    await user.click(screen.getByRole("button", { name: /选型计算/ }));
    await screen.findByRole("heading", { name: "同步带基础计算" });

    expect(screen.getByText("通用电机功率计算")).toBeInTheDocument();
    expect(screen.getByText("伺服/步进选型计算")).toBeInTheDocument();
    expect(screen.getByText("滚珠丝杠伺服计算")).toBeInTheDocument();
    expect(screen.getByText("减速机基础计算")).toBeInTheDocument();
    expect(screen.getByText("直线模组选型判断")).toBeInTheDocument();

    await user.type(screen.getByLabelText("搜索计算模块"), "丝杆");
    await user.click(screen.getByRole("button", { name: /滚珠丝杠伺服计算/ }));
    expect(screen.getByRole("heading", { name: "滚珠丝杠伺服计算" })).toBeInTheDocument();

    await user.click(screen.getByLabelText("我已确认安全系数"));
    await user.click(screen.getByRole("button", { name: "计算" }));

    expect((await screen.findAllByText("直动惯量")).length).toBeGreaterThan(0);
    expect(screen.getByText("角加速度")).toBeInTheDocument();
    expect(screen.getByText("加速力矩")).toBeInTheDocument();
    expect(screen.getAllByText("总力矩").length).toBeGreaterThan(0);
    expect(screen.getByText(/总力矩 0.056 Nm/)).toBeInTheDocument();
    expect(screen.getAllByText(/需求转速/).length).toBeGreaterThan(0);
    expect(screen.getAllByText("PDF P25 / 文档页 22 / 丝杆篇").length).toBeGreaterThan(0);

    await waitFor(() => {
      expect(invokeMock).toHaveBeenCalledWith(
        "run_calculation",
        expect.objectContaining({
          request: expect.objectContaining({ moduleId: "ball-screw-servo" }),
        }),
      );
    });
  });
});
