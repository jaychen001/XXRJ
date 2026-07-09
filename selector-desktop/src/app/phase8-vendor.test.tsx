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

  it("主界面不暴露独立厂家样本库，结果页可导入样本并匹配型号", async () => {
    const user = userEvent.setup();
    render(<App />);

    await screen.findByText("本地数据正常");
    expect(screen.queryByRole("button", { name: /厂家样本库/ })).not.toBeInTheDocument();

    await user.click(screen.getByLabelText("我已确认本次计算使用的安全系数"));
    await user.click(screen.getByRole("button", { name: "计算" }));

    expect(await screen.findByText("摩擦力")).toBeInTheDocument();
    expect(screen.queryByText("厂家型号推荐")).not.toBeInTheDocument();

    await user.type(screen.getByLabelText("样本文件路径"), "D:\\samples\\servo.csv");
    await user.selectOptions(screen.getByLabelText("样本文件格式"), "csv");
    await user.click(screen.getByRole("button", { name: "读取预览" }));

    expect(await screen.findByText("识别到 2 个型号，失败 0 行。")).toBeInTheDocument();
    expect(screen.getByText("SV-400 · 2 项参数")).toBeInTheDocument();
    expect(screen.getByLabelText("额定扭矩(Nm)目标字段")).toHaveValue("outputTorque");

    await user.click(screen.getByLabelText("我已核对字段映射"));
    await user.click(screen.getByRole("button", { name: "确认导入" }));

    expect(await screen.findByText("已导入 2 个型号，失败 0 行。")).toBeInTheDocument();
    expect(invokeMock).toHaveBeenCalledWith("confirm_vendor_import", {
      request: expect.objectContaining({
        libraryName: "同步带基础计算样本库",
        componentType: "同步轮同步带",
        confirmed: true,
      }),
    });

    await user.click(screen.getByRole("button", { name: /匹配型号/ }));

    expect(await screen.findByText("SV-400")).toBeInTheDocument();
    expect(screen.getByText("SV-400 输出扭矩满足")).toBeInTheDocument();
    expect(invokeMock).toHaveBeenCalledWith("recommend_vendor_models", {
      request: expect.objectContaining({
        moduleId: "timing-belt-basic",
        componentType: null,
        limit: 5,
        requirements: expect.arrayContaining([
          expect.objectContaining({ id: "outputTorque", unit: "Nm" }),
          expect.objectContaining({ id: "requiredSpeed", unit: "rpm" }),
        ]),
      }),
    });
  });

  it("结果页内可停用和删除样本库", async () => {
    const user = userEvent.setup();
    render(<App />);

    await screen.findByText("本地数据正常");
    expect(screen.queryByRole("button", { name: /厂家样本库/ })).not.toBeInTheDocument();

    await user.click(screen.getByLabelText("我已确认本次计算使用的安全系数"));
    await user.click(screen.getByRole("button", { name: "计算" }));

    expect(await screen.findByText("伺服样本库")).toBeInTheDocument();
    await user.click(screen.getByRole("button", { name: "停用伺服样本库" }));

    expect(invokeMock).toHaveBeenCalledWith("set_vendor_library_enabled", {
      id: "vendor-lib-1",
      enabled: false,
    });
    expect(await screen.findByText("样本库已停用")).toBeInTheDocument();

    await user.click(screen.getByRole("button", { name: "删除伺服样本库" }));
    expect(await screen.findByText("再次点击确认删除 伺服样本库")).toBeInTheDocument();
    await user.click(screen.getByRole("button", { name: "确认删除伺服样本库" }));

    expect(invokeMock).toHaveBeenCalledWith("delete_vendor_library", {
      id: "vendor-lib-1",
    });
    expect(await screen.findByText("样本库已删除")).toBeInTheDocument();
    expect(screen.getByText("还没有样本库。")).toBeInTheDocument();
  });
});
