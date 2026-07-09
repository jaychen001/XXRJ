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

describe("气动与支撑计算", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    setupAppInvokeMock(invokeMock);
  });

  it("在选型计算中完成气缸计算", async () => {
    const user = userEvent.setup();
    render(<App />);
    await screen.findByText("本地数据正常");

    await user.type(screen.getByLabelText("搜索计算对象"), "气缸");
    await user.click(screen.getAllByRole("button", { name: /气缸/ })[0]);
    await user.click(screen.getByLabelText("我已确认本次计算使用的安全系数"));
    await user.click(screen.getByRole("button", { name: "计算" }));

    expect((await screen.findAllByText("负载率修正")).length).toBeGreaterThan(0);
    expect(screen.getAllByText("选型输出力").length).toBeGreaterThan(0);
    expect(screen.getByText(/选型输出力 92.302 N/)).toBeInTheDocument();
    expect(screen.queryByText(/PDF P/)).not.toBeInTheDocument();

    await waitFor(() => {
      expect(invokeMock).toHaveBeenCalledWith(
        "run_calculation",
        expect.objectContaining({
          request: expect.objectContaining({ moduleId: "pneumatic-cylinder-sizing" }),
        }),
      );
    });
  });

  it("在选型计算中完成手指气缸计算", async () => {
    const user = userEvent.setup();
    render(<App />);
    await screen.findByText("本地数据正常");

    await user.type(screen.getByLabelText("搜索计算对象"), "手指气缸");
    await user.click(screen.getAllByRole("button", { name: /手指气缸/ })[0]);
    await user.click(screen.getByLabelText("我已确认本次计算使用的安全系数"));
    await user.click(screen.getByRole("button", { name: "计算" }));

    expect((await screen.findAllByText("单爪夹持力")).length).toBeGreaterThan(0);
    expect(screen.getAllByText("夹持力余量").length).toBeGreaterThan(0);
    expect(screen.getAllByText(/单爪夹持力 29.517 N/).length).toBeGreaterThan(0);
    expect(screen.queryByText(/PDF P/)).not.toBeInTheDocument();

    await waitFor(() => {
      expect(invokeMock).toHaveBeenCalledWith(
        "run_calculation",
        expect.objectContaining({
          request: expect.objectContaining({ moduleId: "pneumatic-gripper-sizing" }),
        }),
      );
    });
  });

  it("在选型计算中完成滑台气缸计算", async () => {
    const user = userEvent.setup();
    render(<App />);
    await screen.findByText("本地数据正常");

    await user.type(screen.getByLabelText("搜索计算对象"), "滑台气缸");
    await user.click(screen.getAllByRole("button", { name: /滑台气缸/ })[0]);
    await user.click(screen.getByLabelText("我已确认本次计算使用的安全系数"));
    await user.click(screen.getByRole("button", { name: "计算" }));

    expect((await screen.findAllByText("推力需求")).length).toBeGreaterThan(0);
    expect(screen.getAllByText("负载动能").length).toBeGreaterThan(0);
    expect(screen.getAllByText("偏载力矩").length).toBeGreaterThan(0);
    expect(screen.queryByText(/PDF P/)).not.toBeInTheDocument();

    await waitFor(() => {
      expect(invokeMock).toHaveBeenCalledWith(
        "run_calculation",
        expect.objectContaining({
          request: expect.objectContaining({ moduleId: "pneumatic-slide-table-sizing" }),
        }),
      );
    });
  });

  it("在选型计算中完成旋转气缸计算", async () => {
    const user = userEvent.setup();
    render(<App />);
    await screen.findByText("本地数据正常");

    await user.type(screen.getByLabelText("搜索计算对象"), "旋转气缸");
    await user.click(screen.getAllByRole("button", { name: /旋转气缸/ })[0]);
    await user.click(screen.getByLabelText("我已确认本次计算使用的安全系数"));
    await user.click(screen.getByRole("button", { name: "计算" }));

    expect((await screen.findAllByText("需求扭矩")).length).toBeGreaterThan(0);
    expect(screen.getAllByText("负载动能").length).toBeGreaterThan(0);
    expect(screen.getAllByText("扭矩余量").length).toBeGreaterThan(0);
    expect(screen.queryByText(/PDF P/)).not.toBeInTheDocument();

    await waitFor(() => {
      expect(invokeMock).toHaveBeenCalledWith(
        "run_calculation",
        expect.objectContaining({
          request: expect.objectContaining({ moduleId: "pneumatic-rotary-actuator-sizing" }),
        }),
      );
    });
  });
});
