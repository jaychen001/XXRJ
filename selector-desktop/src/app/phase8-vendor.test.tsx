import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { invoke } from "@tauri-apps/api/core";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { setupAppInvokeMock } from "./app-test-fixtures";
import { App } from "./App";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

const invokeMock = vi.mocked(invoke);

describe("厂家样本能力的用户可见边界", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    setupAppInvokeMock(invokeMock);
  });

  it("不在主界面暴露独立厂家样本库和型号匹配入口", async () => {
    const user = userEvent.setup();
    render(<App />);

    await screen.findByText("本地数据正常");
    expect(screen.queryByRole("button", { name: /厂家样本库/ })).not.toBeInTheDocument();

    await user.click(screen.getByLabelText("我已确认本次计算使用的安全系数"));
    await user.click(screen.getByRole("button", { name: "计算" }));

    expect(await screen.findByText("摩擦力")).toBeInTheDocument();
    expect(screen.queryByText("厂家型号推荐")).not.toBeInTheDocument();
    expect(screen.queryByRole("button", { name: /匹配型号/ })).not.toBeInTheDocument();
    expect(invokeMock).not.toHaveBeenCalledWith(
      "recommend_vendor_models",
      expect.anything(),
    );
  });
});
