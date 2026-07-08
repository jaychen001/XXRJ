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

describe("机械传动计算", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    setupAppInvokeMock(invokeMock);
  });

  it("在选型计算中完成齿轮参数计算", async () => {
    const user = userEvent.setup();
    render(<App />);
    await screen.findByText("本地数据正常");

    await user.type(screen.getByLabelText("搜索计算对象"), "齿轮");
    await user.click(screen.getByRole("button", { name: /齿轮参数计算/ }));
    await user.click(screen.getByLabelText("我已确认本次计算使用的安全系数"));
    await user.click(screen.getByRole("button", { name: "计算" }));

    expect((await screen.findAllByText("中心距")).length).toBeGreaterThan(0);
    expect(screen.getAllByText("减速比").length).toBeGreaterThan(0);
    expect(screen.getByText(/中心距 80.000 mm/)).toBeInTheDocument();
    expect(screen.queryByText(/PDF P/)).not.toBeInTheDocument();

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
