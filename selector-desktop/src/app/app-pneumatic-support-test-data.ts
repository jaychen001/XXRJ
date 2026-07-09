import type { CalculationResult, ModuleDefinition } from "../domain/calculation";
import {
  pneumaticActuatorModules,
  pneumaticActuatorResultForModule,
} from "./app-pneumatic-actuator-test-data";

export const pneumaticSupportModules: ModuleDefinition[] = [
  implementedModule("pneumatic-cylinder-sizing", "气缸", "气动", "气动执行元件", "气缸输出力、负载率和缸径初筛", [
    field("loadMass", "负载质量", "kg", 8, "PDF P69 / 文档页 66 / 气动执行元件"),
    field("frictionCoefficient", "摩擦系数", "ratio", 0.2, "PDF P69 / 文档页 66 / 气动执行元件"),
    field("acceleration", "加速度", "m/s²", 1.5, "PDF P69 / 文档页 66 / 气动执行元件"),
    field("workingPressure", "工作压力", "MPa", 0.5, "PDF P69 / 文档页 66 / 气动执行元件"),
    field("loadRateLimit", "负载率上限", "ratio", 0.5, "PDF P69 / 文档页 66 / 气动执行元件"),
    field("mechanicalEfficiency", "机械效率", "ratio", 0.9, "PDF P69 / 文档页 66 / 气动执行元件"),
  ]),
  ...pneumaticActuatorModules,
  implementedModule("vacuum-suction-sizing", "真空吸附", "气动", "气动执行元件", "吸附力、吸盘面积和直径", [
    field("workpieceMass", "工件质量", "kg", 2, "PDF P98 / 文档页 95 / 真空吸附"),
    field("acceleration", "搬运加速度", "m/s²", 2, "PDF P98 / 文档页 95 / 真空吸附"),
    field("vacuumPressure", "有效真空压力", "kPa", 60, "PDF P98 / 文档页 95 / 真空吸附"),
    field("cupCount", "吸盘数量", "pcs", 4, "PDF P98 / 文档页 95 / 真空吸附"),
    field("leakageFactor", "泄漏修正", "ratio", 0.8, "PDF P98 / 文档页 95 / 真空吸附"),
  ]),
  implementedModule("pneumatic-flow-control", "电磁阀", "气动", "气动控制（调速阀）", "耗气量、调速阀和阀口径初筛", [
    field("cylinderCount", "气缸数量", "pcs", 1, "工程公式库 / 气动控制"),
    field("cylinderBore", "气缸缸径", "mm", 32, "工程公式库 / 气动控制", ["mm", "m"]),
    field("rodDiameter", "活塞杆直径", "mm", 12, "工程公式库 / 气动控制", ["mm", "m"]),
    field("stroke", "行程", "mm", 100, "工程公式库 / 气动控制", ["mm", "m"]),
    field("tubeInnerDiameter", "管路内径", "mm", 6, "工程公式库 / 气动控制", ["mm", "m"]),
    field("tubeLength", "管路长度", "mm", 1000, "工程公式库 / 气动控制", ["mm", "m"]),
    field("cycleTime", "循环时间", "s", 2, "工程公式库 / 气动控制", ["s", "min"]),
    field("workingPressure", "工作压力", "MPa", 0.5, "工程公式库 / 气动控制"),
    field("actuationFrequency", "动作频率", "cycle/min", 20, "工程公式库 / 气动控制"),
    field("valveRatedFlow", "阀额定流量", "L/min", 200, "工程公式库 / 气动控制"),
  ]),
  implementedModule("linear-guide-sizing", "直线导轨", "支撑导向", "直线导轨", "滑块载荷、载荷余量和偏载力矩", [
    field("loadMass", "负载质量", "kg", 20, "PDF P103 / 文档页 100 / 直线导轨"),
    field("sliderCount", "滑块数量", "pcs", 4, "PDF P103 / 文档页 100 / 直线导轨"),
    field("dynamicLoadRating", "单滑块动额定载荷", "N", 5000, "PDF P103 / 文档页 100 / 直线导轨"),
    field("impactFactor", "冲击系数", "ratio", 1.5, "PDF P103 / 文档页 100 / 直线导轨"),
    field("offsetDistance", "偏载距离", "mm", 50, "PDF P103 / 文档页 100 / 直线导轨", ["mm", "m"]),
  ]),
  implementedModule("linear-bearing-selector", "直线轴承", "支撑导向", "直线轴承", "载荷、速度和类型判断", [
    field("radialLoad", "总径向载荷", "N", 200, "工程公式库 / 直线轴承"),
    field("bearingCount", "轴承数量", "pcs", 2, "工程公式库 / 直线轴承"),
    field("loadDirectionFactor", "方向系数", "ratio", 1, "工程公式库 / 直线轴承"),
    field("impactFactor", "冲击系数", "ratio", 1.5, "工程公式库 / 直线轴承"),
    field("shaftDiameter", "轴径", "mm", 20, "工程公式库 / 直线轴承"),
    field("travelSpeed", "运行速度", "mm/s", 300, "工程公式库 / 直线轴承", ["mm/s", "m/s"]),
    field("requiredTravelLife", "目标行走寿命", "km", 5000, "工程公式库 / 直线轴承"),
    field("loadRating", "额定动载荷 C", "N", 1000, "工程公式库 / 直线轴承"),
  ]),
  implementedModule("rolling-bearing-life", "滚动轴承", "支撑导向", "滚动轴承", "等效载荷、寿命和样册参数匹配", [
    field("radialLoad", "径向载荷", "N", 500, "工程公式库 / 滚动轴承"),
    field("axialLoad", "轴向载荷", "N", 100, "工程公式库 / 滚动轴承"),
    field("xFactor", "径向系数 X", "ratio", 1, "工程公式库 / 滚动轴承"),
    field("yFactor", "轴向系数 Y", "ratio", 0.6, "工程公式库 / 滚动轴承"),
    field("shaftSpeed", "转速", "rpm", 600, "工程公式库 / 滚动轴承", ["rpm", "rps"]),
    field("requiredLifeHours", "目标寿命", "h", 10000, "工程公式库 / 滚动轴承"),
    field("dynamicLoadRating", "动额定载荷 C", "N", 3000, "工程公式库 / 滚动轴承"),
    field("staticLoadRating", "静额定载荷 C0", "N", 2000, "工程公式库 / 滚动轴承"),
    field("lifeExponent", "寿命指数", "ratio", 3, "工程公式库 / 滚动轴承"),
    field("applicationFactor", "工况系数", "ratio", 1.2, "工程公式库 / 滚动轴承"),
  ]),
  implementedModule("coupling-selector", "联轴器", "连接件", "联轴器", "扭矩、惯量和偏差补偿判断", [
    field("motorTorque", "电机额定扭矩", "Nm", 2, "工程公式库 / 联轴器"),
    field("peakTorque", "峰值扭矩", "Nm", 4, "工程公式库 / 联轴器"),
    field("ratedTorque", "联轴器额定扭矩", "Nm", 10, "工程公式库 / 联轴器"),
    field("shockFactor", "冲击系数", "ratio", 1.5, "工程公式库 / 联轴器"),
    field("temperatureFactor", "温度修正系数", "ratio", 1, "工程公式库 / 联轴器"),
    field("shaftSpeed", "轴转速", "rpm", 1500, "工程公式库 / 联轴器", ["rpm", "rps"]),
    field("inertiaRatio", "负载惯量比", "ratio", 3, "工程公式库 / 联轴器"),
    field("parallelOffset", "平行偏差", "mm", 0.05, "工程公式库 / 联轴器"),
    field("angularOffset", "角向偏差", "deg", 0.5, "工程公式库 / 联轴器"),
    field("axialOffset", "轴向位移", "mm", 0.2, "工程公式库 / 联轴器"),
  ]),
];

export function pneumaticSupportResultForModule(moduleId: string): CalculationResult | null {
  if (moduleId === "pneumatic-cylinder-sizing") {
    return cylinderResult;
  }
  return pneumaticActuatorResultForModule(moduleId);
}

const cylinderResult = {
  moduleId: "pneumatic-cylinder-sizing",
  moduleName: "气缸",
  formulaVersion: "pneumatic-cylinder-sizing@0.1.0",
  summary: "选型输出力 92.302 N，缸径需求 15.328 mm",
  conclusion: "按安全系数 1.500 和负载率 0.500 修正，气缸理论输出力至少 92.302 N。",
  steps: [
    step("摩擦力", "Ff = m * g * μ", "8.000 * 9.80665 * 0.200", "15.691", "N", "PDF P69 / 文档页 66 / 气动执行元件"),
    step("加速力", "Fa = m * a", "8.000 * 1.500", "12.000", "N", "PDF P69 / 文档页 66 / 气动执行元件"),
    step("负载率修正", "F = (Ff + Fa) * K / (η * λ)", "(15.691 + 12.000) * 1.500 / (0.900 * 0.500)", "92.302", "N", "PDF P69 / 文档页 66 / 气动执行元件"),
    step("缸径需求", "D = sqrt(4F / (πP))", "sqrt(4*92.302 / (π*0.500))", "15.328", "mm", "PDF P69 / 文档页 66 / 气动执行元件"),
  ],
  rules: [
    {
      id: "cylinder-bore",
      label: "缸径初筛",
      recommendation: "按计算缸径上取厂家标准缸径，并复核安装形式和缓冲。",
      basis: "需求缸径 15.328 mm",
      risk: "low",
      source: "PDF P69 / 文档页 66 / 气动执行元件",
    },
  ],
  risks: [{ level: "success", message: "未发现基础计算风险。", fieldId: null, source: "PDF P69 / 文档页 66 / 气动执行元件" }],
  requirements: [
    { id: "frictionForce", label: "摩擦力", value: 15.691, unit: "N" },
    { id: "accelerationForce", label: "加速力", value: 12, unit: "N" },
    { id: "outputForce", label: "选型输出力", value: 92.302, unit: "N" },
    { id: "boreDiameter", label: "缸径需求", value: 15.328, unit: "mm" },
  ],
  sourcePages: ["PDF P69 / 文档页 66 / 气动执行元件"],
  inputSnapshot: {},
  defaultsSnapshot: {},
} satisfies CalculationResult;

function implementedModule(
  id: string,
  name: string,
  category: string,
  sourceChapter: string,
  description: string,
  fields: ModuleDefinition["fields"],
): ModuleDefinition {
  return { id, name, category, description, sourceChapter, sourcePage: fields[0]?.source ?? sourceChapter, fields };
}

function field(id: string, label: string, unit: string, defaultValue: number, source: string, unitOptions = [unit]) {
  return { id, label, unit, unitOptions, required: true, min: 0, defaultValue, helper: label, source };
}

function step(label: string, formula: string, substitution: string, result: string, unit: string, source: string) {
  return { label, formula, substitution, result, unit, source };
}
