import type {
  CalculationRequest,
  CalculationResult,
  ModuleDefinition,
} from "../domain/calculation";
import { mechanicalModules, mechanicalResultForModule } from "./app-mechanical-test-data";
import {
  pneumaticSupportModules,
  pneumaticSupportResultForModule,
} from "./app-pneumatic-support-test-data";
import { ruleModules, ruleResultForModule } from "./app-rule-test-data";

export const calculationResult = {
  moduleId: "timing-belt-basic",
  moduleName: "同步带基础计算",
  formulaVersion: "timing-belt-basic@0.2.0",
  summary: "输出扭矩 0.351 Nm，需求转速 300.000 rpm",
  conclusion: "按安全系数 1.500 计算，驱动端至少需要 0.351 Nm、300.000 rpm。",
  steps: [
    step("摩擦力", "Ff = m * g * μ", "5 * 9.80665 * 0.1", "4.903", "N", "PDF P34 / 文档页 31 / 同步带"),
    step("加速力", "Fa = m * a", "5 * 1.667", "8.333", "N", "PDF P34 / 文档页 31 / 同步带"),
    step("输出扭矩", "T = F * Dp / 2", "22.061 * 0.032 / 2", "0.351", "Nm", "PDF P34 / 文档页 31 / 同步带"),
    step("需求转速", "n = v / (z * p) * 60", "0.500 / 0.100 * 60", "300.000", "rpm", "PDF P34 / 文档页 31 / 同步带"),
  ],
  rules: [
    rule("timing-belt-speed", "速度区间", "同步带传动可进入型号匹配", "目标速度 500.000 mm/s，基础阈值 2000 mm/s", "low", "PDF P34 / 文档页 31 / 同步带"),
  ],
  risks: [risk("success", "未发现基础速度、效率或安全系数风险。", null, "PDF P34 / 文档页 31 / 同步带")],
  requirements: [
    { id: "beltForce", label: "等效推力", value: 22.061, unit: "N" },
    { id: "outputTorque", label: "输出扭矩", value: 0.351, unit: "Nm" },
    { id: "requiredSpeed", label: "需求转速", value: 300, unit: "rpm" },
    { id: "power", label: "估算功率", value: 11.031, unit: "W" },
  ],
  sourcePages: ["PDF P34 / 文档页 31 / 同步带"],
  inputSnapshot: {},
  defaultsSnapshot: {},
} satisfies CalculationResult;

export const calculationRequest = {
  moduleId: "timing-belt-basic",
  safetyFactor: 1.5,
  safetyFactorConfirmed: true,
  fields: [
    { id: "loadMass", value: 5, unit: "kg" },
    { id: "frictionCoefficient", value: 0.1, unit: "ratio" },
    { id: "targetSpeed", value: 500, unit: "mm/s" },
    { id: "accelerationTime", value: 0.3, unit: "s" },
    { id: "externalForce", value: 0, unit: "N" },
    { id: "verticalLoadFactor", value: 0, unit: "ratio" },
    { id: "pulleyTeeth", value: 20, unit: "teeth" },
    { id: "toothPitch", value: 5, unit: "mm" },
    { id: "efficiency", value: 0.9, unit: "ratio" },
  ],
} satisfies CalculationRequest;

export type CalculationRequestFixture = CalculationRequest;

export const calculationModules: ModuleDefinition[] = [
  implementedModule("timing-belt-basic", "同步带基础计算", "传动", "同步带", "同步带负载到扭矩和转速计算", [
    field("loadMass", "负载质量", "kg", 5, "PDF P34 / 文档页 31 / 同步带"),
    field("frictionCoefficient", "摩擦系数", "ratio", 0.1, "PDF P34 / 文档页 31 / 同步带"),
    field("targetSpeed", "目标速度", "mm/s", 500, "PDF P34 / 文档页 31 / 同步带", ["mm/s", "m/s"]),
    field("accelerationTime", "加速时间", "s", 0.3, "PDF P34 / 文档页 31 / 同步带", ["s", "min"]),
    field("externalForce", "外部阻力", "N", 0, "工程公式库 / 同步带"),
    field("verticalLoadFactor", "垂直负载系数", "ratio", 0, "工程公式库 / 同步带"),
    field("pulleyTeeth", "同步轮齿数", "teeth", 20, "PDF P34 / 文档页 31 / 同步带"),
    field("toothPitch", "齿距", "mm", 5, "PDF P34 / 文档页 31 / 同步带", ["mm", "m"]),
    field("efficiency", "传动效率", "ratio", 0.9, "PDF P34 / 文档页 31 / 同步带"),
  ]),
  implementedModule("ball-screw-servo", "滚珠丝杠伺服计算", "传动", "丝杆篇", "丝杠惯量、力矩和伺服转速", [
    field("loadMass", "负载质量", "kg", 10, "PDF P25 / 文档页 22 / 丝杆篇"),
    field("lead", "丝杠导程", "mm", 10, "PDF P25 / 文档页 22 / 丝杆篇", ["mm", "m"]),
    field("targetSpeed", "目标速度", "mm/s", 300, "PDF P25 / 文档页 22 / 丝杆篇", ["mm/s", "m/s"]),
    field("accelerationTime", "加速时间", "s", 0.2, "PDF P25 / 文档页 22 / 丝杆篇", ["s", "min"]),
    field("frictionCoefficient", "摩擦系数", "ratio", 0.05, "PDF P25 / 文档页 22 / 丝杆篇"),
    field("efficiency", "丝杠效率", "ratio", 0.9, "PDF P25 / 文档页 22 / 丝杆篇"),
  ]),
  ...mechanicalModules,
  implementedModule("reducer-basic", "减速机基础计算", "传动", "减速机", "减速比、输出扭矩和输入扭矩", [
    field("motorSpeed", "电机转速", "rpm", 1500, "PDF P54 / 文档页 51 / 减速机", ["rpm", "rps"]),
    field("outputSpeed", "输出转速", "rpm", 60, "PDF P54 / 文档页 51 / 减速机", ["rpm", "rps"]),
    field("loadTorque", "负载扭矩", "Nm", 20, "PDF P54 / 文档页 51 / 减速机"),
    field("efficiency", "减速机效率", "ratio", 0.85, "PDF P54 / 文档页 51 / 减速机"),
  ]),
  implementedModule("linear-module-selector", "直线模组选型判断", "传动", "直线模组", "丝杆、同步带或常规模组初筛", [
    field("loadMass", "负载质量", "kg", 8, "PDF P57 / 文档页 54 / 直线模组"),
    field("stroke", "行程", "mm", 600, "PDF P57 / 文档页 54 / 直线模组", ["mm", "m"]),
    field("targetSpeed", "目标速度", "mm/s", 500, "PDF P57 / 文档页 54 / 直线模组", ["mm/s", "m/s"]),
    field("accelerationTime", "加速时间", "s", 0.3, "PDF P57 / 文档页 54 / 直线模组", ["s", "min"]),
    field("positioningAccuracy", "定位精度", "mm", 0.05, "PDF P57 / 文档页 54 / 直线模组"),
    field("frictionCoefficient", "摩擦系数", "ratio", 0.1, "PDF P57 / 文档页 54 / 直线模组"),
  ]),
  implementedModule("general-motor-power", "通用电机功率计算", "驱动", "电机篇", "输送线功率、扭矩和转速估算", [
    field("loadMass", "负载质量", "kg", 20, "PDF P4 / 文档页 1 / 电机篇"),
    field("driveDiameter", "驱动直径", "mm", 80, "PDF P4 / 文档页 1 / 电机篇", ["mm", "m"]),
    field("lineSpeed", "线速度", "mm/s", 300, "PDF P4 / 文档页 1 / 电机篇", ["mm/s", "m/s"]),
    field("accelerationTime", "启动加速时间", "s", 1, "工程公式库 / 普通与调速电机", ["s", "min"]),
    field("frictionCoefficient", "摩擦系数", "ratio", 0.15, "PDF P4 / 文档页 1 / 电机篇"),
    field("externalForce", "外部阻力", "N", 0, "工程公式库 / 普通与调速电机"),
    field("verticalLoadFactor", "垂直负载系数", "ratio", 0, "工程公式库 / 普通与调速电机"),
    field("efficiency", "传动效率", "ratio", 0.85, "PDF P4 / 文档页 1 / 电机篇"),
  ]),
  implementedModule("servo-stepper-sizing", "伺服/步进选型计算", "驱动", "电机篇", "转速、力矩、惯量比和分辨率复核", [
    field("loadMass", "负载质量", "kg", 8, "PDF P4 / 文档页 1 / 电机篇 / 伺服步进"),
    field("travelPerRev", "每转位移", "mm", 20, "PDF P4 / 文档页 1 / 电机篇 / 伺服步进", ["mm", "m"]),
    field("targetSpeed", "目标速度", "mm/s", 500, "PDF P4 / 文档页 1 / 电机篇 / 伺服步进", ["mm/s", "m/s"]),
    field("accelerationTime", "加速时间", "s", 0.2, "PDF P4 / 文档页 1 / 电机篇 / 伺服步进", ["s", "min"]),
    field("frictionCoefficient", "摩擦系数", "ratio", 0.1, "PDF P4 / 文档页 1 / 电机篇 / 伺服步进"),
    field("efficiency", "传动效率", "ratio", 0.9, "PDF P4 / 文档页 1 / 电机篇 / 伺服步进"),
    field("motorInertia", "电机转子惯量", "kg·m²", 0.00002, "PDF P4 / 文档页 1 / 电机篇 / 伺服步进"),
    field("encoderResolution", "编码器分辨率", "pulse/rev", 10000, "PDF P4 / 文档页 1 / 电机篇 / 伺服步进"),
  ]),
  ...pneumaticSupportModules,
  ...ruleModules,
];

export const initialCaseRecord = {
  id: "case-1",
  name: "同步带测试",
  moduleId: "timing-belt-basic",
  moduleName: "同步带基础计算",
  notes: "",
  resultSummary: calculationResult.summary,
  riskCount: 1,
  createdAt: "2026-07-03 15:00:00",
  updatedAt: "2026-07-03 15:00:00",
};

export function calculationResultForModule(moduleId: string): CalculationResult {
  const mechanicalResult = mechanicalResultForModule(moduleId);
  if (mechanicalResult) {
    return mechanicalResult;
  }
  const pneumaticSupportResult = pneumaticSupportResultForModule(moduleId);
  if (pneumaticSupportResult) {
    return pneumaticSupportResult;
  }
  const ruleResult = ruleResultForModule(moduleId);
  if (ruleResult) {
    return ruleResult;
  }
  return moduleId === "ball-screw-servo" ? ballScrewResult : calculationResult;
}

const ballScrewResult = {
  ...calculationResult,
  moduleId: "ball-screw-servo",
  moduleName: "滚珠丝杠伺服计算",
  formulaVersion: "ball-screw-servo@0.1.0",
  summary: "总力矩 0.056 Nm，需求转速 1800.000 rpm",
  conclusion: "按安全系数 1.600 计算，丝杠伺服至少需要 0.056 Nm、1800.000 rpm。",
  steps: [
    step("直动惯量", "J = m * (L / 2π)^2", "10 * (0.010 / 2π)^2", "0.000", "kg·m²", "PDF P25 / 文档页 22 / 丝杆篇"),
    step("角加速度", "α = ω / t", "188.496 / 0.200", "942.478", "rad/s²", "PDF P25 / 文档页 22 / 丝杆篇"),
    step("加速力矩", "Ta = J * α * K / η", "0.000025 * 942.478 * 1.600 / 0.900", "0.042", "Nm", "PDF P25 / 文档页 22 / 丝杆篇"),
    step("匀速力矩", "Tc = Ff * L / 2π * K / η", "4.903 * 0.010 / 2π * 1.600 / 0.900", "0.014", "Nm", "PDF P25 / 文档页 22 / 丝杆篇"),
    step("总力矩", "T = Ta + Tc", "0.042 + 0.014", "0.056", "Nm", "PDF P25 / 文档页 22 / 丝杆篇"),
    step("需求转速", "n = v / L * 60", "0.300 / 0.010 * 60", "1800.000", "rpm", "PDF P25 / 文档页 22 / 丝杆篇"),
  ],
  requirements: [
    { id: "reflectedInertia", label: "直动惯量", value: 0.000025, unit: "kg·m²" },
    { id: "totalTorque", label: "总力矩", value: 0.056, unit: "Nm" },
    { id: "requiredSpeed", label: "需求转速", value: 1800, unit: "rpm" },
  ],
  sourcePages: ["PDF P25 / 文档页 22 / 丝杆篇"],
} satisfies CalculationResult;

function implementedModule(
  id: string,
  name: string,
  category: string,
  sourceChapter: string,
  description: string,
  fields: ModuleDefinition["fields"],
): ModuleDefinition {
  return {
    id,
    name,
    category,
    description,
    sourceChapter,
    sourcePage: fields[0]?.source ?? `根目录 PDF / ${sourceChapter}`,
    fields,
  };
}

function field(
  id: string,
  label: string,
  unit: string,
  defaultValue: number,
  source: string,
  unitOptions = [unit],
) {
  return {
    id,
    label,
    unit,
    unitOptions,
    required: true,
    min: 0,
    defaultValue,
    helper: label,
    source,
  };
}

function step(
  label: string,
  formula: string,
  substitution: string,
  result: string,
  unit: string,
  source: string,
) {
  return { label, formula, substitution, result, unit, source };
}

function rule(
  id: string,
  label: string,
  recommendation: string,
  basis: string,
  risk: string,
  source: string,
) {
  return { id, label, recommendation, basis, risk, source };
}

function risk(level: "success", message: string, fieldId: null, source: string) {
  return { level, message, fieldId, source };
}
