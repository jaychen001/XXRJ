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
});
