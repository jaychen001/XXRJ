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

describe("规则型选型计算", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    setupAppInvokeMock(invokeMock);
  });

  it.each([
    ["机器人", /机器人规则选型/, "robot-rule-selector", "推荐类型", /建议 SCARA 或小型六轴机器人/],
    ["拖链", /拖链规则选型/, "cable-chain-rule-selector", "填充率", /安装长度估算 635.619 mm/],
    ["传感器", /传感器规则选型/, "sensor-rule-selector", "环境适配", /建议 接近开关/],
    ["材料", /材料规则选型/, "material-rule-selector", "推荐材料", /45 钢调质或耐磨工程塑料/],
    ["机加工", /机加工规则选型/, "machining-rule-selector", "加工方式", /建议 车铣常规加工/],
    ["热处理", /热处理&表面处理规则选型/, "heat-surface-rule-selector", "推荐处理", /调质、发黑或常规防锈/],
    ["五金件", /常用五金件规则选型/, "hardware-rule-selector", "推荐五金件", /常规内六角螺钉/],
  ])("在选型计算中完成%s规则选型", async (query, buttonName, moduleId, expectedRule, expectedSummary) => {
    const user = userEvent.setup();
    render(<App />);
    await screen.findByText("本地数据正常");

    await user.type(screen.getByLabelText("搜索计算对象"), query);
    await user.click(screen.getByRole("button", { name: buttonName }));
    await user.click(screen.getByLabelText("我已确认本次计算使用的安全系数"));
    await user.click(screen.getByRole("button", { name: "计算" }));

    expect((await screen.findAllByText(expectedRule)).length).toBeGreaterThan(0);
    expect(screen.getAllByText(expectedSummary).length).toBeGreaterThan(0);
    expect(screen.queryByText(/PDF P/)).not.toBeInTheDocument();

    await waitFor(() => {
      expect(invokeMock).toHaveBeenCalledWith(
        "run_calculation",
        expect.objectContaining({
          request: expect.objectContaining({ moduleId }),
        }),
      );
    });
  });
});
