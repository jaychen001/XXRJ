import type { CalculationResult, ModuleDefinition } from "../domain/calculation";

const SOURCE = "工程公式库";

export const pneumaticActuatorModules: ModuleDefinition[] = [
  implementedModule("pneumatic-gripper-sizing", "手指气缸", "夹持力和手指力矩校核", [
    field("partMass", "工件质量", "kg", 1),
    field("frictionCoefficient", "夹持摩擦系数", "ratio", 0.3),
    field("jawCount", "夹爪数量", "pcs", 2),
    field("jawArm", "夹持力臂", "mm", 30, ["mm", "m"]),
    field("acceleration", "搬运加速度", "m/s²", 2),
    field("orientationFactor", "姿态重力系数", "ratio", 1),
    field("externalForce", "外部扰动力", "N", 0),
    field("candidateGripForce", "候选单爪有效夹持力", "N", 80),
    field("allowableFingerMoment", "候选允许手指力矩", "Nm", 3),
  ]),
  implementedModule("pneumatic-slide-table-sizing", "滑台气缸", "推力、偏载力矩和缓冲动能校核", [
    field("loadMass", "负载质量", "kg", 5),
    field("guideFriction", "导向摩擦系数", "ratio", 0.15),
    field("acceleration", "加速度", "m/s²", 2),
    field("stroke", "行程", "mm", 100, ["mm", "m"]),
    field("moveTime", "移动时间", "s", 0.5, ["s", "min"]),
    field("loadRateLimit", "负载率上限", "ratio", 0.5),
    field("externalForce", "外部阻力", "N", 0),
    field("verticalLoadFactor", "垂直负载系数", "ratio", 0),
    field("candidateRatedThrust", "候选额定推力", "N", 300),
    field("loadCenterOffset", "负载偏心距", "mm", 40, ["mm", "m"]),
    field("allowableMoment", "候选允许力矩", "Nm", 8),
    field("allowableKineticEnergy", "候选允许动能", "J", 0.5),
  ]),
  implementedModule("pneumatic-rotary-actuator-sizing", "旋转气缸", "扭矩、旋转动能和缓冲风险校核", [
    field("loadInertia", "负载惯量", "kg·m²", 0.01),
    field("rotationAngle", "旋转角度", "deg", 90),
    field("rotationTime", "旋转时间", "s", 0.5, ["s", "min"]),
    field("externalTorque", "外部阻力矩", "Nm", 0.5),
    field("torqueLoadRate", "扭矩负载率上限", "ratio", 0.5),
    field("candidateTorque", "候选额定扭矩", "Nm", 5),
    field("allowableKineticEnergy", "候选允许动能", "J", 0.2),
  ]),
];

export function pneumaticActuatorResultForModule(moduleId: string): CalculationResult | null {
  if (moduleId === "pneumatic-gripper-sizing") {
    return gripperResult;
  }
  if (moduleId === "pneumatic-slide-table-sizing") {
    return slideTableResult;
  }
  if (moduleId === "pneumatic-rotary-actuator-sizing") {
    return rotaryResult;
  }
  return null;
}

const gripperResult = result(
  "pneumatic-gripper-sizing",
  "手指气缸",
  "pneumatic-gripper-sizing@0.2.0",
  "单爪夹持力 29.517 N，夹持力余量 2.710",
  "候选手指需提供不小于 29.517 N 的单爪有效夹持力，并满足 0.885 Nm 手指力矩。",
  [
    step("夹持负载", "Fload = m * (g * kg + a) + Fext", "1.000*(9.80665*1.000+2.000)+0.000", "11.807", "N"),
    step("单爪夹持力", "Fjaw = Fload * K / (μ * n)", "11.807*1.500 / (0.300*2.000)", "29.517", "N"),
    step("总夹持力", "Ftotal = Fjaw * n", "29.517*2.000", "59.033", "N"),
    step("夹持力矩", "M = Fjaw * L", "29.517*0.030", "0.885", "Nm"),
    step("夹持力余量", "Sforce = Frated / Fjaw", "80.000 / 29.517", "2.710", "ratio"),
    step("手指力矩余量", "Smoment = Mrated / M", "3.000 / 0.885", "3.388", "ratio"),
  ],
  [
    { id: "holdingForce", label: "总夹持力", value: 59.033, unit: "N" },
    { id: "forcePerJaw", label: "单爪夹持力", value: 29.517, unit: "N" },
    { id: "gripTorque", label: "夹持力矩", value: 0.885, unit: "Nm" },
    { id: "forceMargin", label: "夹持力余量", value: 2.71, unit: "ratio" },
    { id: "momentMargin", label: "手指力矩余量", value: 3.388, unit: "ratio" },
  ],
);

const slideTableResult = result(
  "pneumatic-slide-table-sizing",
  "滑台气缸",
  "pneumatic-slide-table-sizing@0.2.0",
  "推力需求 52.064 N，动能余量 5.000",
  "候选滑台气缸需提供不小于 52.064 N 推力，并满足 2.942 Nm 偏载力矩和 0.100 J 动能。",
  [
    step("导向摩擦力", "Ff = m * g * μ", "5.000*9.80665*0.150", "7.355", "N"),
    step("加速力", "Fa = m * a", "5.000*2.000", "10.000", "N"),
    step("垂直负载力", "Fg = m * g * kv", "5.000*9.80665*0.000", "0.000", "N"),
    step("推力需求", "F = (Ff + Fa + Fg + Fext) * K / λ", "(7.355+10.000+0.000+0.000)*1.500 / 0.500", "52.064", "N"),
    step("平均速度", "v = S / t", "0.100 / 0.500", "0.200", "m/s"),
    step("负载动能", "E = 0.5 * m * v²", "0.5*5.000*0.200²", "0.100", "J"),
    step("偏载力矩", "M = m * g * L * K", "5.000*9.80665*0.040*1.500", "2.942", "Nm"),
    step("推力余量", "Sforce = Frated / F", "300.000 / 52.064", "5.762", "ratio"),
    step("力矩余量", "Smoment = Mrated / M", "8.000 / 2.942", "2.720", "ratio"),
    step("动能余量", "Senergy = Erated / E", "0.500 / 0.100", "5.000", "ratio"),
  ],
  [
    { id: "requiredThrust", label: "推力需求", value: 52.064, unit: "N" },
    { id: "averageSpeed", label: "平均速度", value: 0.2, unit: "m/s" },
    { id: "kineticEnergy", label: "负载动能", value: 0.1, unit: "J" },
    { id: "loadMoment", label: "偏载力矩", value: 2.942, unit: "Nm" },
    { id: "thrustMargin", label: "推力余量", value: 5.762, unit: "ratio" },
    { id: "momentMargin", label: "力矩余量", value: 2.72, unit: "ratio" },
    { id: "energyMargin", label: "动能余量", value: 5, unit: "ratio" },
  ],
);

const rotaryResult = result(
  "pneumatic-rotary-actuator-sizing",
  "旋转气缸",
  "pneumatic-rotary-actuator-sizing@0.1.0",
  "需求扭矩 2.254 Nm，动能余量 1.013",
  "候选旋转气缸需提供不小于 2.254 Nm 扭矩，并满足 0.197 J 旋转动能。",
  [
    step("峰值角速度", "ωp = 2θ / t", "2*1.571 / 0.500", "6.283", "rad/s"),
    step("角加速度", "α = 4θ / t²", "4*1.571 / 0.500²", "25.133", "rad/s²"),
    step("惯量扭矩", "Tj = J * α", "0.010*25.133", "0.251", "Nm"),
    step("需求扭矩", "T = (Tj + Text) * K / λ", "(0.251+0.500)*1.500 / 0.500", "2.254", "Nm"),
    step("负载动能", "E = 0.5 * J * ωp²", "0.5*0.010*6.283²", "0.197", "J"),
    step("扭矩余量", "Storque = Trated / T", "5.000 / 2.254", "2.218", "ratio"),
    step("动能余量", "Senergy = Erated / E", "0.200 / 0.197", "1.013", "ratio"),
    step("最小旋转时间", "tmin = sqrt(2 * J * θ² / Erated)", "sqrt(2*0.010*1.571² / 0.200)", "0.497", "s"),
  ],
  [
    { id: "requiredTorque", label: "需求扭矩", value: 2.254, unit: "Nm" },
    { id: "kineticEnergy", label: "负载动能", value: 0.197, unit: "J" },
    { id: "torqueMargin", label: "扭矩余量", value: 2.218, unit: "ratio" },
    { id: "energyMargin", label: "动能余量", value: 1.013, unit: "ratio" },
    { id: "minimumRotationTime", label: "最小旋转时间", value: 0.497, unit: "s" },
  ],
);

function implementedModule(id: string, name: string, description: string, fields: ModuleDefinition["fields"]): ModuleDefinition {
  return { id, name, category: "气动", description, sourceChapter: "气动执行元件", sourcePage: SOURCE, fields };
}

function field(id: string, label: string, unit: string, defaultValue: number, unitOptions = [unit]) {
  return { id, label, unit, unitOptions, required: true, min: 0, defaultValue, helper: label, source: SOURCE };
}

function result(
  moduleId: string,
  moduleName: string,
  formulaVersion: string,
  summary: string,
  conclusion: string,
  steps: CalculationResult["steps"],
  requirements: CalculationResult["requirements"],
): CalculationResult {
  return {
    moduleId,
    moduleName,
    formulaVersion,
    summary,
    conclusion,
    steps,
    rules: [{ id: `${moduleId}-candidate`, label: "候选规格判断", recommendation: "可进入候选样本匹配。", basis: summary, risk: "low", source: SOURCE }],
    risks: [{ level: "success", message: "未发现基础计算风险。", fieldId: null, source: SOURCE }],
    requirements,
    sourcePages: [SOURCE],
    inputSnapshot: {},
    defaultsSnapshot: {},
  };
}

function step(label: string, formula: string, substitution: string, resultValue: string, unit: string) {
  return { label, formula, substitution, result: resultValue, unit, source: SOURCE };
}
