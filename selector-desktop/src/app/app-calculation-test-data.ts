export const calculationResult = {
  moduleId: "timing-belt-basic",
  moduleName: "同步带基础计算",
  formulaVersion: "timing-belt-basic@0.1.0",
  summary: "输出扭矩 0.221 Nm，需求转速 300.000 rpm",
  conclusion: "按安全系数 1.500 计算，驱动端至少需要 0.221 Nm、300.000 rpm。",
  steps: [
    {
      label: "摩擦力",
      formula: "Ff = m * g * μ",
      substitution: "5 * 9.80665 * 0.1",
      result: "4.903",
      unit: "N",
      source: "根目录 PDF / 同步带匹配页",
    },
  ],
  rules: [
    {
      id: "timing-belt-speed",
      label: "速度区间",
      recommendation: "同步带传动可进入型号匹配",
      basis: "目标速度 500.000 mm/s，基础阈值 2000 mm/s",
      risk: "low",
      source: "根目录 PDF / 同步带匹配页",
    },
  ],
  risks: [
    {
      level: "success",
      message: "未发现基础速度、效率或安全系数风险。",
      fieldId: null,
      source: "根目录 PDF / 同步带匹配页",
    },
  ],
  requirements: [{ id: "outputTorque", label: "输出扭矩", value: 0.221, unit: "Nm" }],
  sourcePages: ["根目录 PDF / 同步带匹配页"],
  inputSnapshot: {},
  defaultsSnapshot: {},
};

export const calculationRequest = {
  moduleId: "timing-belt-basic",
  safetyFactor: 1.5,
  safetyFactorConfirmed: true,
  fields: [
    { id: "loadMass", value: 5, unit: "kg" },
    { id: "frictionCoefficient", value: 0.1, unit: "ratio" },
    { id: "targetSpeed", value: 500, unit: "mm/s" },
    { id: "accelerationTime", value: 0.3, unit: "s" },
    { id: "pulleyTeeth", value: 20, unit: "teeth" },
    { id: "toothPitch", value: 5, unit: "mm" },
    { id: "efficiency", value: 0.9, unit: "ratio" },
  ],
};

export type CalculationRequestFixture = typeof calculationRequest;

export const calculationModules = [
  {
    id: "timing-belt-basic",
    name: "同步带基础计算",
    category: "传动",
    description: "同步带负载到扭矩和转速计算",
    sourceChapter: "同步带",
    sourcePage: "根目录 PDF / 同步带匹配页",
    fields: [
      field("loadMass", "负载质量", "kg", 5),
      field("frictionCoefficient", "摩擦系数", "ratio", 0.1),
      field("targetSpeed", "目标速度", "mm/s", 500, ["mm/s", "m/s"]),
      field("accelerationTime", "加速时间", "s", 0.3, ["s", "min"]),
      field("pulleyTeeth", "同步轮齿数", "teeth", 20),
      field("toothPitch", "齿距", "mm", 5, ["mm", "m"]),
      field("efficiency", "传动效率", "ratio", 0.9),
    ],
  },
  plannedModule("cylinder", "气缸", "气动执行元件"),
  plannedModule("vacuum", "真空吸附", "气动执行元件"),
  plannedModule("solenoid-valve", "电磁阀", "气动控制（调速阀）"),
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

function field(id: string, label: string, unit: string, defaultValue: number, unitOptions = [unit]) {
  return {
    id,
    label,
    unit,
    unitOptions,
    required: true,
    min: 0,
    defaultValue,
    helper: label,
    source: "根目录 PDF / 同步带",
  };
}

function plannedModule(id: string, name: string, sourceChapter: string) {
  return {
    id,
    name,
    category: "规划",
    description: "后续章节包实现",
    sourceChapter,
    sourcePage: `根目录 PDF / ${sourceChapter}`,
    fields: [],
  };
}
