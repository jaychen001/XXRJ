import type { CalculationResult, ModuleDefinition } from "../domain/calculation";

export const ruleModules: ModuleDefinition[] = [
  module("robot-rule-selector", "机器人规则选型", "机器人", "PDF P67 / 文档页 64 / 机器人", [
    field("payload", "负载", "kg", 5),
    field("reach", "臂展", "mm", 600, ["mm", "m"]),
    field("cycleTime", "节拍", "s", 2, ["s", "min"]),
    field("precision", "重复精度", "mm", 0.05),
    field("applicationCode", "应用场景", "code", 1),
  ]),
  module("cable-chain-rule-selector", "拖链规则选型", "拖链", "PDF P121 / 文档页 118 / 拖链", [
    field("travel", "行程", "mm", 800, ["mm", "m"]),
    field("bendRadius", "弯曲半径", "mm", 75, ["mm", "m"]),
    field("cableCount", "线缆数量", "pcs", 8),
    field("fillRate", "填充率", "ratio", 0.5),
    field("speed", "运行速度", "mm/s", 500, ["mm/s", "m/s"]),
  ]),
  module("sensor-rule-selector", "传感器规则选型", "传感器", "PDF P123 / 文档页 120 / 传感器", [
    field("objectCode", "检测对象", "code", 1),
    field("detectDistance", "检测距离", "mm", 20, ["mm", "m"]),
    field("responseTime", "响应时间", "ms", 10),
    field("environmentCode", "环境等级", "code", 1),
    field("spaceLimit", "安装空间", "mm", 30),
  ]),
  module("material-rule-selector", "材料规则选型", "材料", "PDF P135 / 文档页 132 / 材料", [
    field("strengthLevel", "强度需求", "MPa", 300),
    field("wearDemand", "耐磨需求", "score", 2),
    field("corrosionDemand", "耐腐蚀需求", "score", 1),
    field("weightSensitive", "轻量化需求", "score", 1),
    field("foodGradeDemand", "食品医药需求", "score", 0),
  ]),
  module("machining-rule-selector", "机加工规则选型", "机加工", "PDF P139 / 文档页 136 / 机加工", [
    field("tolerance", "公差需求", "mm", 0.05),
    field("batchQty", "批量", "pcs", 5),
    field("hardness", "材料硬度", "HB", 180),
    field("complexity", "结构复杂度", "score", 1),
    field("surfaceDemand", "表面要求", "score", 1),
  ]),
  module("heat-surface-rule-selector", "热处理&表面处理规则选型", "热处理&表面处理", "PDF P141 / 文档页 138 / 热处理&表面处理", [
    field("hardnessTarget", "目标硬度", "HRC", 35),
    field("wearDemand", "耐磨需求", "score", 1),
    field("corrosionDemand", "防腐需求", "score", 1),
    field("appearanceDemand", "外观需求", "score", 1),
    field("deformationRisk", "变形敏感", "score", 1),
  ]),
  module("hardware-rule-selector", "常用五金件规则选型", "常用五金件", "PDF P146 / 文档页 143 / 常用五金件", [
    field("load", "载荷", "N", 500),
    field("vibrationLevel", "振动等级", "score", 1),
    field("adjustFrequency", "调节频率", "score", 1),
    field("spaceLimit", "安装空间", "mm", 20),
    field("disassemblyNeed", "拆装需求", "score", 1),
  ]),
];

export function ruleResultForModule(moduleId: string): CalculationResult | null {
  return moduleId === "robot-rule-selector" ? robotResult : null;
}

const robotResult = {
  moduleId: "robot-rule-selector",
  moduleName: "机器人规则选型",
  formulaVersion: "robot-rule-selector@0.1.0",
  summary: "建议 SCARA 或小型六轴机器人，设计负载 7.500 kg",
  conclusion: "规则选型结果可作为厂家样本库筛选条件，最终型号需复核负载曲线、惯量和节拍。",
  steps: [
    step("问题1 负载余量", "payload * K", "5.000 * 1.500", "7.500", "kg", "PDF P67 / 文档页 64 / 机器人"),
    step("问题2 臂展", "reach", "600.000 mm", "600.000", "mm", "PDF P67 / 文档页 64 / 机器人"),
    step("问题3 节拍", "cycleTime", "2.000 s", "2.000", "s", "PDF P67 / 文档页 64 / 机器人"),
    step("问题4 精度", "precision", "0.050 mm", "0.050", "mm", "PDF P67 / 文档页 64 / 机器人"),
  ],
  rules: [
    rule("robot-type", "推荐类型", "SCARA 或小型六轴机器人", "负载 7.500 kg，臂展 600.000 mm", "low"),
    rule("robot-speed", "节拍判断", "常规搬运/装配机器人可进入样本筛选。", "节拍 2.000 s", "low"),
    rule("robot-precision", "精度风险", "精度需求处于常规初筛范围。", "重复精度 0.050 mm", "low"),
  ],
  risks: [{ level: "success", message: "未发现基础计算风险。", fieldId: null, source: "PDF P67 / 文档页 64 / 机器人" }],
  requirements: [
    { id: "designPayload", label: "设计负载", value: 7.5, unit: "kg" },
    { id: "reach", label: "臂展", value: 600, unit: "mm" },
    { id: "cycleTime", label: "节拍", value: 2, unit: "s" },
  ],
  sourcePages: ["PDF P67 / 文档页 64 / 机器人"],
  inputSnapshot: {},
  defaultsSnapshot: {},
} satisfies CalculationResult;

function module(id: string, name: string, sourceChapter: string, source: string, fields: ModuleDefinition["fields"]): ModuleDefinition {
  return {
    id,
    name,
    category: "规则选型",
    description: `${sourceChapter}工况规则选型`,
    sourceChapter,
    sourcePage: source,
    fields: fields.map((item) => ({ ...item, source })),
  };
}

function field(id: string, label: string, unit: string, defaultValue: number, unitOptions = [unit]) {
  return { id, label, unit, unitOptions, required: true, min: 0, defaultValue, helper: label, source: "" };
}

function step(label: string, formula: string, substitution: string, result: string, unit: string, source: string) {
  return { label, formula, substitution, result, unit, source };
}

function rule(id: string, label: string, recommendation: string, basis: string, risk: string) {
  return { id, label, recommendation, basis, risk, source: "PDF P67 / 文档页 64 / 机器人" };
}
