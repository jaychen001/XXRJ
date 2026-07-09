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

describe("驱动与线性传动计算", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    setupAppInvokeMock(invokeMock);
  });

  it("在选型计算中完成滚珠丝杠伺服计算且不展示 PDF 来源", async () => {
    const user = userEvent.setup();
    render(<App />);
    await screen.findByText("本地数据正常");

    await user.type(screen.getByLabelText("搜索计算对象"), "丝杠");
    await user.click(screen.getByRole("button", { name: /滚珠丝杠伺服计算/ }));
    expect(screen.getByRole("heading", { name: "滚珠丝杠伺服计算" })).toBeInTheDocument();

    await user.click(screen.getByLabelText("我已确认本次计算使用的安全系数"));
    await user.click(screen.getByRole("button", { name: "计算" }));

    expect((await screen.findAllByText("直动惯量")).length).toBeGreaterThan(0);
    expect(screen.getByText("角加速度")).toBeInTheDocument();
    expect(screen.getByText("加速力矩")).toBeInTheDocument();
    expect(screen.getAllByText("总力矩").length).toBeGreaterThan(0);
    expect(screen.getByText(/总力矩 0.056 Nm/)).toBeInTheDocument();
    expect(screen.queryByText(/PDF P/)).not.toBeInTheDocument();

    await waitFor(() => {
      expect(invokeMock).toHaveBeenCalledWith(
        "run_calculation",
        expect.objectContaining({
          request: expect.objectContaining({ moduleId: "ball-screw-servo" }),
        }),
      );
    });
  });

  it("在选型计算中完成减速机承载校核", async () => {
    const user = userEvent.setup();
    render(<App />);
    await screen.findByText("本地数据正常");

    await user.type(screen.getByLabelText("搜索计算对象"), "减速机");
    await user.click(screen.getByRole("button", { name: /减速机基础计算/ }));
    await user.click(screen.getByLabelText("我已确认本次计算使用的安全系数"));
    await user.click(screen.getByRole("button", { name: "计算" }));

    expect((await screen.findAllByText("减速比")).length).toBeGreaterThan(0);
    expect(screen.getAllByText("设计输出扭矩").length).toBeGreaterThan(0);
    expect(screen.getAllByText("扭矩余量").length).toBeGreaterThan(0);
    expect(screen.getAllByText(/减速比 25.000/).length).toBeGreaterThan(0);
    expect(screen.queryByText(/PDF P/)).not.toBeInTheDocument();

    await waitFor(() => {
      expect(invokeMock).toHaveBeenCalledWith(
        "run_calculation",
        expect.objectContaining({
          request: expect.objectContaining({ moduleId: "reducer-basic" }),
        }),
      );
    });
  });

  it("在选型计算中完成直线模组推力和寿命校核", async () => {
    const user = userEvent.setup();
    render(<App />);
    await screen.findByText("本地数据正常");

    await user.type(screen.getByLabelText("搜索计算对象"), "直线模组");
    await user.click(screen.getByRole("button", { name: /直线模组选型判断/ }));
    await user.click(screen.getByLabelText("我已确认本次计算使用的安全系数"));
    await user.click(screen.getByRole("button", { name: "计算" }));

    expect((await screen.findAllByText("推力需求")).length).toBeGreaterThan(0);
    expect(screen.getAllByText("额定寿命").length).toBeGreaterThan(0);
    expect(screen.getAllByText("静载余量").length).toBeGreaterThan(0);
    expect(screen.getAllByText(/推荐 滚珠丝杠模组/).length).toBeGreaterThan(0);
    expect(screen.queryByText(/PDF P/)).not.toBeInTheDocument();

    await waitFor(() => {
      expect(invokeMock).toHaveBeenCalledWith(
        "run_calculation",
        expect.objectContaining({
          request: expect.objectContaining({ moduleId: "linear-module-selector" }),
        }),
      );
    });
  });
});
