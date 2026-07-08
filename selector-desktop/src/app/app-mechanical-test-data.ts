import type { CalculationResult, ModuleDefinition } from "../domain/calculation";

export const mechanicalModules: ModuleDefinition[] = [
  implementedModule("v-belt-selector", "V 带选型计算", "传动", "V 带", "V 带速度、功率和带型判断", [
    field("transmitPower", "传递功率", "kW", 0.75, "PDF P40 / 文档页 37 / V 带", ["kW", "W"]),
    field("smallPulleyDiameter", "小带轮直径", "mm", 100, "PDF P40 / 文档页 37 / V 带", ["mm", "m"]),
    field("smallPulleySpeed", "小带轮转速", "rpm", 1450, "PDF P40 / 文档页 37 / V 带", ["rpm", "rps"]),
    field("serviceFactor", "工况系数", "ratio", 1.3, "PDF P40 / 文档页 37 / V 带"),
    field("beltEfficiency", "传动效率", "ratio", 0.95, "PDF P40 / 文档页 37 / V 带"),
  ]),
  implementedModule("gear-basic", "齿轮参数计算", "传动", "齿轮", "模数、齿数、中心距和减速比计算", [
    field("module", "模数", "mm", 2, "PDF P44 / 文档页 41 / 齿轮"),
    field("driveTeeth", "主动齿数", "teeth", 20, "PDF P44 / 文档页 41 / 齿轮"),
    field("drivenTeeth", "从动齿数", "teeth", 60, "PDF P44 / 文档页 41 / 齿轮"),
    field("faceWidth", "齿宽", "mm", 20, "PDF P44 / 文档页 41 / 齿轮"),
  ]),
  implementedModule("chain-selector", "链条选型计算", "传动", "链条", "节距、齿数、中心距和链速判断", [
    field("pitch", "节距", "mm", 12.7, "PDF P49 / 文档页 46 / 链条", ["mm", "m"]),
    field("smallSprocketTeeth", "小链轮齿数", "teeth", 18, "PDF P49 / 文档页 46 / 链条"),
    field("largeSprocketTeeth", "大链轮齿数", "teeth", 36, "PDF P49 / 文档页 46 / 链条"),
    field("centerDistance", "中心距", "mm", 500, "PDF P49 / 文档页 46 / 链条", ["mm", "m"]),
    field("sprocketSpeed", "小链轮转速", "rpm", 200, "PDF P49 / 文档页 46 / 链条", ["rpm", "rps"]),
  ]),
  implementedModule("cam-indexer-sizing", "凸轮分割器选型计算", "间歇传动", "凸轮分割器", "工位、节拍、惯量和驱动扭矩", [
    field("stationCount", "工位数", "station", 8, "PDF P59 / 文档页 56 / 凸轮分割器"),
    field("cycleTime", "循环时间", "s", 2, "PDF P59 / 文档页 56 / 凸轮分割器", ["s", "min"]),
    field("indexAngle", "分割角度", "deg", 45, "PDF P59 / 文档页 56 / 凸轮分割器"),
    field("tableInertia", "负载惯量", "kg·m²", 0.05, "PDF P59 / 文档页 56 / 凸轮分割器"),
    field("loadTorque", "外部负载扭矩", "Nm", 5, "PDF P59 / 文档页 56 / 凸轮分割器"),
    field("efficiency", "传动效率", "ratio", 0.8, "PDF P59 / 文档页 56 / 凸轮分割器"),
  ]),
  implementedModule("brake-clutch-selector", "制动器/离合器选型", "间歇传动", "制动器/离合器", "扭矩、响应和热负荷判断", [
    field("loadTorque", "负载扭矩", "Nm", 5, "PDF P65 / 文档页 62 / 制动器/离合器"),
    field("rotatingInertia", "旋转惯量", "kg·m²", 0.02, "PDF P65 / 文档页 62 / 制动器/离合器"),
    field("shaftSpeed", "轴转速", "rpm", 600, "PDF P65 / 文档页 62 / 制动器/离合器", ["rpm", "rps"]),
    field("stopTime", "停止时间", "s", 0.3, "PDF P65 / 文档页 62 / 制动器/离合器", ["s", "min"]),
    field("cyclesPerMinute", "动作频率", "cycle/min", 20, "PDF P65 / 文档页 62 / 制动器/离合器"),
    field("responseTime", "响应时间", "ms", 50, "PDF P65 / 文档页 62 / 制动器/离合器"),
  ]),
];

export function mechanicalResultForModule(moduleId: string): CalculationResult | null {
  return moduleId === "gear-basic" ? gearResult : null;
}

const gearResult = {
  moduleId: "gear-basic",
  moduleName: "齿轮参数计算",
  formulaVersion: "gear-basic@0.1.0",
  summary: "中心距 80.000 mm，减速比 3.000",
  conclusion: "按模数 2.000、齿数 20.000/60.000 计算，中心距为 80.000 mm。",
  steps: [
    step("主动分度圆", "d1 = m * z1", "2.000 * 20.000", "40.000", "mm", "PDF P44 / 文档页 41 / 齿轮"),
    step("从动分度圆", "d2 = m * z2", "2.000 * 60.000", "120.000", "mm", "PDF P44 / 文档页 41 / 齿轮"),
    step("中心距", "a = (d1 + d2) / 2", "(40.000 + 120.000) / 2", "80.000", "mm", "PDF P44 / 文档页 41 / 齿轮"),
    step("减速比", "i = z2 / z1", "60.000 / 20.000", "3.000", "ratio", "PDF P44 / 文档页 41 / 齿轮"),
  ],
  rules: [
    {
      id: "gear-undercut",
      label: "齿数风险",
      recommendation: "齿数可进入标准直齿轮初筛。",
      basis: "主动齿数 20.000",
      risk: "low",
      source: "PDF P44 / 文档页 41 / 齿轮",
    },
  ],
  risks: [{ level: "success", message: "未发现基础计算风险。", fieldId: null, source: "PDF P44 / 文档页 41 / 齿轮" }],
  requirements: [
    { id: "module", label: "模数", value: 2, unit: "mm" },
    { id: "centerDistance", label: "中心距", value: 80, unit: "mm" },
    { id: "gearRatio", label: "减速比", value: 3, unit: "ratio" },
  ],
  sourcePages: ["PDF P44 / 文档页 41 / 齿轮"],
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
