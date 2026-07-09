import type { CalculationResult, ModuleDefinition } from "../domain/calculation";

const SOURCE = "工程规则库";

export const ruleModules: ModuleDefinition[] = [
  module("robot-rule-selector", "机器人规则选型", "机器人", [
    field("payload", "负载", "kg", 5),
    field("reach", "臂展", "mm", 600, ["mm", "m"]),
    field("cycleTime", "节拍", "s", 2, ["s", "min"]),
    field("precision", "重复精度", "mm", 0.05),
    field("applicationCode", "应用场景", "code", 1),
  ]),
  module("cable-chain-rule-selector", "拖链规则选型", "拖链", [
    field("travel", "行程", "mm", 800, ["mm", "m"]),
    field("bendRadius", "弯曲半径", "mm", 75, ["mm", "m"]),
    field("cableCount", "线缆数量", "pcs", 8),
    field("fillRate", "填充率", "ratio", 0.5),
    field("speed", "运行速度", "mm/s", 500, ["mm/s", "m/s"]),
  ]),
  module("sensor-rule-selector", "传感器规则选型", "传感器", [
    field("objectCode", "检测对象", "code", 1),
    field("detectDistance", "检测距离", "mm", 20, ["mm", "m"]),
    field("responseTime", "响应时间", "ms", 10),
    field("environmentCode", "环境等级", "code", 1),
    field("spaceLimit", "安装空间", "mm", 30),
  ]),
  module("material-rule-selector", "材料规则选型", "材料", [
    field("strengthLevel", "强度需求", "MPa", 300),
    field("wearDemand", "耐磨需求", "score", 2),
    field("corrosionDemand", "耐腐蚀需求", "score", 1),
    field("weightSensitive", "轻量化需求", "score", 1),
    field("foodGradeDemand", "食品医药需求", "score", 0),
  ]),
  module("machining-rule-selector", "机加工规则选型", "机加工", [
    field("tolerance", "公差需求", "mm", 0.05),
    field("batchQty", "批量", "pcs", 5),
    field("hardness", "材料硬度", "HB", 180),
    field("complexity", "结构复杂度", "score", 1),
    field("surfaceDemand", "表面要求", "score", 1),
  ]),
  module("heat-surface-rule-selector", "热处理&表面处理规则选型", "热处理&表面处理", [
    field("hardnessTarget", "目标硬度", "HRC", 35),
    field("wearDemand", "耐磨需求", "score", 1),
    field("corrosionDemand", "防腐需求", "score", 1),
    field("appearanceDemand", "外观需求", "score", 1),
    field("deformationRisk", "变形敏感", "score", 1),
  ]),
  module("hardware-rule-selector", "常用五金件规则选型", "常用五金件", [
    field("load", "载荷", "N", 500),
    field("vibrationLevel", "振动等级", "score", 1),
    field("adjustFrequency", "调节频率", "score", 1),
    field("spaceLimit", "安装空间", "mm", 20),
    field("disassemblyNeed", "拆装需求", "score", 1),
  ]),
];

const ruleResults: Record<string, CalculationResult> = {
  "robot-rule-selector": result("robot-rule-selector", "机器人规则选型", "建议 SCARA 或小型六轴机器人，设计负载 7.500 kg", [
    step("问题1 负载余量", "payload * K", "5.000 * 1.500", "7.500", "kg"),
    step("问题2 臂展", "reach", "600.000 mm", "600.000", "mm"),
    step("问题3 节拍", "cycleTime", "2.000 s", "2.000", "s"),
    step("问题4 精度", "precision", "0.050 mm", "0.050", "mm"),
  ], [
    rule("robot-type", "推荐类型", "SCARA 或小型六轴机器人", "负载 7.500 kg，臂展 600.000 mm", "low"),
    rule("robot-speed", "节拍判断", "常规搬运/装配机器人可进入样本筛选。", "节拍 2.000 s", "low"),
    rule("robot-precision", "精度风险", "精度需求处于常规初筛范围。", "重复精度 0.050 mm", "low"),
  ], reqs([["designPayload", "设计负载", 7.5, "kg"], ["reach", "臂展", 600, "mm"], ["cycleTime", "节拍", 2, "s"]])),
  "cable-chain-rule-selector": result("cable-chain-rule-selector", "拖链规则选型", "建议 常规封闭或桥式拖链，安装长度估算 635.619 mm", [
    step("问题1 行程", "travel", "800.000 mm", "800.000", "mm"),
    step("问题2 弯曲半径", "R", "75.000 mm", "75.000", "mm"),
    step("问题3 填充余量", "0.6 / fillRate", "0.6 / 0.500", "1.200", "ratio"),
    step("安装长度估算", "L = S/2 + πR", "800.000/2 + π*75.000", "635.619", "mm"),
  ], [
    rule("chain-type", "推荐类型", "常规封闭或桥式拖链", "行程 800.000 mm，速度 500.000 mm/s", "low"),
    rule("chain-fill", "填充率", "填充率可进入常规初筛。", "填充率 0.500", "low"),
    rule("chain-cables", "线缆管理", "多线缆建议加隔片并按粗线缆弯曲半径选 R。", "线缆数量 8.000", "low"),
  ], reqs([["installLength", "安装长度估算", 635.619, "mm"], ["bendRadius", "弯曲半径", 75, "mm"], ["fillMargin", "填充余量", 1.2, "ratio"]])),
  "sensor-rule-selector": result("sensor-rule-selector", "传感器规则选型", "建议 接近开关，距离余量 30.000 mm", [
    step("问题1 检测对象", "objectCode", "1.000", "1.000", "code"),
    step("问题2 检测距离", "distance * K", "20.000*1.500", "30.000", "mm"),
    step("问题3 响应时间", "responseTime", "10.000 ms", "10.000", "ms"),
    step("问题4 安装空间", "spaceLimit", "30.000 mm", "30.000", "mm"),
  ], [
    rule("sensor-type", "推荐类型", "接近开关", "对象代码 1.000，距离 20.000 mm", "low"),
    rule("sensor-response", "响应风险", "响应时间可进入常规模块初筛。", "响应 10.000 ms", "low"),
    rule("sensor-environment", "环境适配", "普通环境可按距离和对象筛选。", "环境代码 1.000", "low"),
  ], reqs([["detectDistance", "检测距离余量", 30, "mm"], ["responseTime", "响应时间", 10, "ms"], ["spaceLimit", "安装空间", 30, "mm"]])),
  "material-rule-selector": result("material-rule-selector", "材料规则选型", "建议 45 钢调质或耐磨工程塑料，设计强度 450.000 MPa", [
    step("问题1 强度", "σd = σ * K", "300.000*1.500", "450.000", "MPa"),
    step("问题2 耐磨", "wearDemand", "2.000", "2.000", "score"),
    step("问题3 耐腐蚀", "corrosionDemand", "1.000", "1.000", "score"),
    step("问题4 轻量化", "weightSensitive", "1.000", "1.000", "score"),
  ], [
    rule("material-type", "推荐材料", "45 钢调质或耐磨工程塑料", "强度 450.000 MPa，耐磨 2.000", "low"),
    rule("material-corrosion", "防腐风险", "普通防腐需求可按成本和加工性筛选。", "耐腐蚀 1.000", "low"),
    rule("material-process", "加工适配", "材料选择需同步复核机加工、热处理和表面处理可行性。", "食品医药 0.000", "low"),
  ], reqs([["designStrength", "设计强度", 450, "MPa"], ["wearDemand", "耐磨需求", 2, "score"], ["corrosionDemand", "耐腐蚀需求", 1, "score"]])),
  "machining-rule-selector": result("machining-rule-selector", "机加工规则选型", "建议 车铣常规加工，精度指标 30.000", [
    step("问题1 公差", "1/tolerance * K", "1/0.050*1.500", "30.000", "index"),
    step("问题2 批量", "batchQty", "5.000", "5.000", "pcs"),
    step("问题3 硬度", "hardness", "180.000 HB", "180.000", "HB"),
    step("问题4 复杂度", "complexity", "1.000", "1.000", "score"),
  ], [
    rule("machining-process", "加工方式", "车铣常规加工", "公差 0.050 mm，复杂度 1.000", "low"),
    rule("machining-cost", "成本风险", "批量加工可评估工装和工序合并。", "批量 5.000 pcs", "low"),
    rule("machining-quality", "质量注意", "普通表面按常规粗糙度和去毛刺控制。", "表面要求 1.000", "low"),
  ], reqs([["precisionIndex", "精度指标", 30, "index"], ["batchQty", "批量", 5, "pcs"], ["hardness", "材料硬度", 180, "HB"]])),
  "heat-surface-rule-selector": result("heat-surface-rule-selector", "热处理&表面处理规则选型", "建议 调质、发黑或常规防锈，处理需求指标 6.500", [
    step("问题1 硬度", "hardnessTarget", "35.000 HRC", "35.000", "HRC"),
    step("问题2 耐磨", "wearDemand", "1.000", "1.000", "score"),
    step("问题3 防腐", "corrosionDemand", "1.000", "1.000", "score"),
    step("问题4 外观", "appearanceDemand", "1.000", "1.000", "score"),
  ], [
    rule("treatment-type", "推荐处理", "调质、发黑或常规防锈", "硬度 35.000 HRC，防腐 1.000", "low"),
    rule("treatment-deformation", "变形风险", "变形风险可按常规工艺控制。", "变形敏感 1.000", "low"),
    rule("treatment-appearance", "外观注意", "普通外观按功能防护优先。", "外观需求 1.000", "low"),
  ], reqs([["processScore", "处理需求指标", 6.5, "score"], ["hardnessTarget", "目标硬度", 35, "HRC"], ["corrosionDemand", "防腐需求", 1, "score"]])),
  "hardware-rule-selector": result("hardware-rule-selector", "常用五金件规则选型", "建议 常规内六角螺钉、销钉和垫圈，设计载荷 750.000 N", [
    step("问题1 设计载荷", "F = load * K", "500.000*1.500", "750.000", "N"),
    step("问题2 振动等级", "vibrationLevel", "1.000", "1.000", "score"),
    step("问题3 调节频率", "adjustFrequency", "1.000", "1.000", "score"),
    step("问题4 安装空间", "spaceLimit", "20.000 mm", "20.000", "mm"),
  ], [
    rule("hardware-type", "推荐五金件", "常规内六角螺钉、销钉和垫圈", "载荷 750.000 N，空间 20.000 mm", "low"),
    rule("hardware-lock", "防松风险", "低振动场景可按常规紧固件初筛。", "振动 1.000", "low"),
    rule("hardware-adjust", "调节维护", "低频调节可用常规螺钉和定位件。", "调节频率 1.000", "low"),
  ], reqs([["designLoad", "设计载荷", 750, "N"], ["spaceLimit", "安装空间", 20, "mm"], ["disassemblyNeed", "拆装需求", 1, "score"]])),
};

export function ruleResultForModule(moduleId: string): CalculationResult | null {
  return ruleResults[moduleId] ?? null;
}

function module(id: string, name: string, sourceChapter: string, fields: ModuleDefinition["fields"]): ModuleDefinition {
  return { id, name, category: "规则选型", description: `${sourceChapter}工况规则选型`, sourceChapter, sourcePage: SOURCE, fields: fields.map((item) => ({ ...item, source: SOURCE })) };
}

function field(id: string, label: string, unit: string, defaultValue: number, unitOptions = [unit]) {
  return { id, label, unit, unitOptions, required: true, min: 0, defaultValue, helper: label, source: SOURCE };
}

function result(moduleId: string, moduleName: string, summary: string, steps: CalculationResult["steps"], rules: CalculationResult["rules"], requirements: CalculationResult["requirements"]): CalculationResult {
  return {
    moduleId,
    moduleName,
    formulaVersion: `${moduleId}@0.1.0`,
    summary,
    conclusion: "规则选型结果用于工程初筛，最终型号需结合厂家样本、安装空间和现场工况复核。",
    steps,
    rules,
    risks: [{ level: "success", message: "未发现基础规则风险。", fieldId: null, source: SOURCE }],
    requirements,
    sourcePages: [SOURCE],
    inputSnapshot: {},
    defaultsSnapshot: {},
  };
}

function step(label: string, formula: string, substitution: string, resultValue: string, unit: string) {
  return { label, formula, substitution, result: resultValue, unit, source: SOURCE };
}

function rule(id: string, label: string, recommendation: string, basis: string, risk: string) {
  return { id, label, recommendation, basis, risk, source: SOURCE };
}

function reqs(items: Array<[string, string, number, string]>): CalculationResult["requirements"] {
  return items.map(([id, label, value, unit]) => ({ id, label, value, unit }));
}
