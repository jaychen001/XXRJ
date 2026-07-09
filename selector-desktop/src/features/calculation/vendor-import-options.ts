import type { CalculationResult } from "../../domain/calculation";
import type { FieldMapping } from "../../domain/vendor";

const FIELD_OPTIONS = [
  ["modelName", "型号"],
  ["brand", "品牌"],
  ["series", "系列"],
  ["outputTorque", "扭矩/力矩"],
  ["requiredSpeed", "转速"],
  ["power", "功率"],
  ["load", "推力/载荷"],
  ["bore", "缸径/直径"],
  ["stroke", "行程"],
  ["flowRate", "流量"],
  ["vacuumPressure", "真空压力"],
  ["dynamicLoadRating", "动额定载荷"],
  ["staticLoadRating", "静额定载荷"],
  ["allowableMoment", "允许力矩"],
  ["kineticEnergy", "允许动能"],
  ["ratedLife", "寿命"],
] as const;

const KNOWN_SOURCE_FORMATS = ["pdf", "csv", "tsv", "xlsx", "xls", "xlsm"] as const;

export function optionsFor(value: string): Array<[string, string]> {
  const options = FIELD_OPTIONS.map(([optionValue, label]) => [optionValue, label] as [string, string]);
  return options.some(([option]) => option === value) ? options : [[value, value], ...options];
}

export function replaceMapping(
  mappings: FieldMapping[],
  index: number,
  key: keyof FieldMapping,
  value: string,
): FieldMapping[] {
  return mappings.map((mapping, currentIndex) =>
    currentIndex === index ? { ...mapping, [key]: value } : mapping,
  );
}

export function formatFromPath(path: string): string {
  const fileName = path.split(/[\\/]/).at(-1) ?? "";
  const extension = fileName.split(".").at(-1)?.toLowerCase() ?? "";
  return KNOWN_SOURCE_FORMATS.some((format) => format === extension) ? extension : "";
}

export function inferVendorComponentType(result: CalculationResult): string {
  if (result.moduleId.includes("timing-belt")) return "同步轮同步带";
  if (result.moduleId.includes("v-belt")) return "V 带";
  if (result.moduleId.includes("chain")) return "链条";
  if (result.moduleId.includes("gear")) return "齿轮";
  if (result.moduleId.includes("reducer")) return "减速机";
  if (result.moduleId.includes("linear-module")) return "直线模组";
  if (result.moduleId.includes("ball-screw")) return "滚珠丝杠";
  if (result.moduleId.includes("linear-bearing")) return "直线轴承";
  if (result.moduleId.includes("linear-guide")) return "直线导轨";
  if (result.moduleId.includes("rolling-bearing")) return "滚动轴承";
  if (result.moduleId.includes("coupling")) return "联轴器";
  if (result.moduleId.includes("brake-clutch")) return "制动器/离合器";
  if (result.moduleId.includes("indexer")) return "分割器";
  if (result.moduleId.includes("rotary-actuator")) return "旋转气缸";
  if (result.moduleId.includes("slide-table")) return "滑台气缸";
  if (result.moduleId.includes("gripper")) return "手指气缸";
  if (result.moduleId.includes("cylinder")) return "气缸";
  if (result.moduleId.includes("vacuum")) return "真空";
  if (result.moduleId.includes("flow-control")) return "电磁阀";
  if (result.moduleId.includes("servo") || result.moduleId.includes("stepper")) return "伺服/步进电机";
  if (result.moduleId.includes("motor")) return "普通电机";
  return result.moduleName;
}
