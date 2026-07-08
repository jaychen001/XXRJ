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

describe("Phase 8 vendor library", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    setupAppInvokeMock(invokeMock);
  });

  it("previews a vendor sample and only imports after field mapping confirmation", async () => {
    const user = userEvent.setup();
    render(<App />);

    await user.click(await screen.findByRole("button", { name: /厂家样本库/ }));
    expect(screen.getByRole("heading", { name: "厂家样本库" })).toBeInTheDocument();
    expect((await screen.findAllByText("伺服样本库")).length).toBeGreaterThan(0);

    await user.clear(screen.getByLabelText("样本库名称"));
    await user.type(screen.getByLabelText("样本库名称"), "伺服 CSV 样本");
    await user.selectOptions(screen.getByLabelText("样本格式"), "csv");
    await user.type(screen.getByLabelText("厂家样本文件路径"), "D:\\samples\\servo.csv");

    await user.click(screen.getByRole("button", { name: /抽取预览/ }));
    const previewTable = await screen.findByRole("table", { name: "厂家样本抽取预览" });
    expect(within(previewTable).getByText("SV-400")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: /确认导入/ })).toBeDisabled();

    await user.click(screen.getByLabelText("已确认字段映射"));
    await user.click(screen.getByRole("button", { name: /确认导入/ }));

    expect(await screen.findByText("已导入 2 个型号，失败 0 行")).toBeInTheDocument();
    expect(await screen.findByText("伺服 CSV 样本")).toBeInTheDocument();
    expect(invokeMock).toHaveBeenCalledWith(
      "confirm_vendor_import",
      expect.objectContaining({
        request: expect.objectContaining({
          confirmed: true,
          libraryName: "伺服 CSV 样本",
        }),
      }),
    );
  });

  it("recommends enabled vendor models from a calculation result", async () => {
    const user = userEvent.setup();
    render(<App />);

    await user.click(await screen.findByRole("button", { name: /选型计算/ }));
    await user.click(screen.getByLabelText("我已确认安全系数"));
    await user.click(screen.getByRole("button", { name: "计算" }));

    expect(await screen.findByText("厂家型号推荐")).toBeInTheDocument();
    await user.click(screen.getByRole("button", { name: /匹配型号/ }));

    expect(await screen.findByText("匹配到 2 个候选型号")).toBeInTheDocument();
    expect(screen.getByText("SV-400")).toBeInTheDocument();
    await waitFor(() => {
      expect(invokeMock).toHaveBeenCalledWith(
        "recommend_vendor_models",
        expect.objectContaining({
          request: expect.objectContaining({
            moduleId: "timing-belt-basic",
            requirements: expect.arrayContaining([
              expect.objectContaining({ id: "outputTorque" }),
            ]),
          }),
        }),
      );
    });
  });
});
