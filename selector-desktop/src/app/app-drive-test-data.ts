import type { CalculationResult, ModuleDefinition } from "../domain/calculation";

export const driveExtraModules: ModuleDefinition[] = [
  implementedModule("reducer-basic", "减速机基础计算", "传动", "减速机", "减速比、输出扭矩、输入扭矩和承载余量", [
    field("motorSpeed", "电机转速", "rpm", 1500, "工程公式库 / 减速机", ["rpm", "rps"]),
    field("outputSpeed", "输出转速", "rpm", 60, "工程公式库 / 减速机", ["rpm", "rps"]),
    field("loadTorque", "输出负载扭矩", "Nm", 20, "工程公式库 / 减速机"),
    field("serviceFactor", "工况系数", "ratio", 1.4, "工程公式库 / 减速机"),
    field("efficiency", "减速机效率", "ratio", 0.85, "工程公式库 / 减速机"),
    field("candidateRatedTorque", "候选额定输出扭矩", "Nm", 80, "工程公式库 / 减速机"),
    field("actualRadialLoad", "输出轴径向载荷", "N", 300, "工程公式库 / 减速机"),
    field("allowableRadialLoad", "允许径向载荷", "N", 900, "工程公式库 / 减速机"),
    field("actualAxialLoad", "输出轴轴向载荷", "N", 80, "工程公式库 / 减速机"),
    field("allowableAxialLoad", "允许轴向载荷", "N", 300, "工程公式库 / 减速机"),
    field("maxInputSpeed", "允许输入转速", "rpm", 3000, "工程公式库 / 减速机", ["rpm", "rps"]),
  ]),
  implementedModule("linear-module-selector", "直线模组选型判断", "传动", "直线模组", "推力、寿命、静载和定位精度初筛", [
    field("loadMass", "负载质量", "kg", 8, "工程公式库 / 直线模组"),
    field("stroke", "行程", "mm", 600, "工程公式库 / 直线模组", ["mm", "m"]),
    field("targetSpeed", "目标速度", "mm/s", 500, "工程公式库 / 直线模组", ["mm/s", "m/s"]),
    field("accelerationTime", "加速时间", "s", 0.3, "工程公式库 / 直线模组", ["s", "min"]),
    field("positioningAccuracy", "目标定位精度", "mm", 0.05, "工程公式库 / 直线模组"),
    field("candidateRepeatability", "候选重复定位精度", "mm", 0.02, "工程公式库 / 直线模组"),
    field("frictionCoefficient", "摩擦系数", "ratio", 0.1, "工程公式库 / 直线模组"),
    field("externalForce", "外部阻力", "N", 0, "工程公式库 / 直线模组"),
    field("verticalLoadFactor", "垂直负载系数", "ratio", 0, "工程公式库 / 直线模组"),
    field("driveEfficiency", "驱动效率", "ratio", 0.85, "工程公式库 / 直线模组"),
    field("candidateRatedThrust", "候选额定推力", "N", 600, "工程公式库 / 直线模组"),
    field("guideLoadFactor", "导向载荷系数", "ratio", 1.2, "工程公式库 / 直线模组"),
    field("dynamicLoadRating", "额定动载荷", "N", 4000, "工程公式库 / 直线模组"),
    field("staticLoadRating", "额定静载荷", "N", 8000, "工程公式库 / 直线模组"),
    field("targetTravelLife", "目标行走寿命", "km", 10000, "工程公式库 / 直线模组"),
  ]),
];

export function driveExtraResultForModule(moduleId: string): CalculationResult | null {
  if (moduleId === "reducer-basic") return reducerResult;
  if (moduleId === "linear-module-selector") return linearModuleResult;
  return null;
}

const reducerResult = result("reducer-basic", "减速机基础计算", "reducer-basic@0.2.0", "减速比 25.000，输出扭矩 42.000 Nm，扭矩余量 1.905", [
  step("减速比", "i = n1 / n2", "1500.000 / 60.000", "25.000", "ratio"),
  step("设计输出扭矩", "T2 = TL * Ka * K", "20.000 * 1.400 * 1.500", "42.000", "Nm"),
  step("输入扭矩", "T1 = T2 / i / η", "42.000 / 25.000 / 0.850", "1.976", "Nm"),
  step("扭矩余量", "M = Tr / T2", "80.000 / 42.000", "1.905", "ratio"),
], [
  { id: "reducerRatio", label: "减速比", value: 25, unit: "ratio" },
  { id: "outputTorque", label: "设计输出扭矩", value: 42, unit: "Nm" },
  { id: "inputTorque", label: "输入扭矩", value: 1.976, unit: "Nm" },
  { id: "torqueMargin", label: "扭矩余量", value: 1.905, unit: "ratio" },
]);

const linearModuleResult = result("linear-module-selector", "直线模组选型判断", "linear-module-selector@0.2.0", "推荐 滚珠丝杠模组，推力 37.374 N，寿命 1136017.449 km", [
  step("加速度", "a = v / t", "0.500 / 0.300", "1.667", "m/s²"),
  step("推力需求", "F = (Fa + Ff + Fg + Fe) * K / η", "(13.333 + 7.845 + 0.000 + 0.000) * 1.500 / 0.850", "37.374", "N"),
  step("额定寿命", "L = 50 * (C / P)^3", "50 * (4000.000 / 141.216)^3", "1136017.449", "km"),
  step("静载余量", "S0 = C0 / P", "8000.000 / 141.216", "56.651", "ratio"),
], [
  { id: "requiredThrust", label: "推力需求", value: 37.374, unit: "N" },
  { id: "thrustMargin", label: "推力余量", value: 16.054, unit: "ratio" },
  { id: "ratedLife", label: "额定寿命", value: 1136017.449, unit: "km" },
  { id: "staticMargin", label: "静载余量", value: 56.651, unit: "ratio" },
]);

function implementedModule(id: string, name: string, category: string, sourceChapter: string, description: string, fields: ModuleDefinition["fields"]): ModuleDefinition {
  return { id, name, category, description, sourceChapter, sourcePage: fields[0]?.source ?? sourceChapter, fields };
}

function field(id: string, label: string, unit: string, defaultValue: number, source: string, unitOptions = [unit]) {
  return { id, label, unit, unitOptions, required: true, min: 0, defaultValue, helper: label, source };
}

function result(moduleId: string, moduleName: string, formulaVersion: string, summary: string, steps: CalculationResult["steps"], requirements: CalculationResult["requirements"]) {
  return {
    moduleId,
    moduleName,
    formulaVersion,
    summary,
    conclusion: summary,
    steps,
    rules: [{ id: `${moduleId}-match`, label: "选型判断", recommendation: "可进入候选样本匹配。", basis: summary, risk: "low", source: "工程公式库" }],
    risks: [{ level: "success", message: "未发现基础计算风险。", fieldId: null, source: "工程公式库" }],
    requirements,
    sourcePages: ["工程公式库"],
    inputSnapshot: {},
    defaultsSnapshot: {},
  } satisfies CalculationResult;
}

function step(label: string, formula: string, substitution: string, result: string, unit: string) {
  return { label, formula, substitution, result, unit, source: "工程公式库" };
}
