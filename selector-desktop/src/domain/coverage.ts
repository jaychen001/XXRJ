export type ImplementationShape =
  | "计算向导 + 规则选型"
  | "规则选型 + 计算向导"
  | "规则选型";

export type CoverageStatus = "planned" | "partial" | "done" | "blocked";

export interface CoverageItem {
  id: string;
  chapter: string;
  shape: ImplementationShape;
  requirement: string;
  status: CoverageStatus;
  source: string;
  entryCount: number;
  catalogPage?: string | null;
  catalogExcerpt?: string;
  notes?: string;
}

export const COVERAGE_ITEMS: CoverageItem[] = [
  {
    id: "motor",
    chapter: "电机篇",
    shape: "计算向导 + 规则选型",
    requirement: "通用电机、伺服、步进、调速/普通电机相关计算",
    status: "done",
    source: "PDF P4 / 文档页 1 / 电机篇",
    entryCount: 2,
  },
  {
    id: "ball-screw",
    chapter: "丝杆篇",
    shape: "计算向导 + 规则选型",
    requirement: "滚珠丝杠、梯形丝杠、导程、效率、惯量、力矩",
    status: "done",
    source: "PDF P25 / 文档页 22 / 丝杆篇",
    entryCount: 1,
  },
  {
    id: "timing-belt",
    chapter: "同步带",
    shape: "计算向导 + 规则选型",
    requirement: "同步轮同步带、分辨率、速度、扭矩、齿形判断",
    status: "done",
    source: "PDF P34 / 文档页 31 / 同步带",
    entryCount: 1,
  },
  {
    id: "v-belt",
    chapter: "V 带",
    shape: "规则选型 + 计算向导",
    requirement: "V 带类型、效率、速度/负载相关判断",
    status: "planned",
    source: "PDF 目录 / V 带",
    entryCount: 0,
  },
  {
    id: "gear",
    chapter: "齿轮",
    shape: "计算向导 + 规则选型",
    requirement: "模数、齿数、中心距、减速比和适用场景",
    status: "planned",
    source: "PDF 目录 / 齿轮",
    entryCount: 0,
  },
  {
    id: "chain",
    chapter: "链条",
    shape: "规则选型 + 计算向导",
    requirement: "节距、链号、中心距、速度、负载",
    status: "planned",
    source: "PDF 目录 / 链条",
    entryCount: 0,
  },
  {
    id: "reducer",
    chapter: "减速机",
    shape: "计算向导 + 规则选型",
    requirement: "减速比、输出转速、输出扭矩、效率、类型选择",
    status: "done",
    source: "PDF P54 / 文档页 51 / 减速机",
    entryCount: 1,
  },
  {
    id: "linear-module",
    chapter: "直线模组",
    shape: "规则选型 + 计算向导",
    requirement: "丝杆、同步带、齿轮齿条、气动模组适用判断",
    status: "done",
    source: "PDF P57 / 文档页 54 / 直线模组",
    entryCount: 1,
  },
  {
    id: "cam-indexer",
    chapter: "凸轮分割器",
    shape: "计算向导 + 规则选型",
    requirement: "工位数、节拍、转速、扭矩/惯量、驱动选择",
    status: "planned",
    source: "PDF 目录 / 凸轮分割器",
    entryCount: 0,
  },
  {
    id: "brake-clutch",
    chapter: "制动器/离合器",
    shape: "规则选型 + 计算向导",
    requirement: "类型选择、扭矩/响应需求、适用工况",
    status: "planned",
    source: "PDF 目录 / 制动器/离合器",
    entryCount: 0,
  },
  {
    id: "robot",
    chapter: "机器人",
    shape: "规则选型",
    requirement: "负载、臂展、节拍、精度、应用场景",
    status: "planned",
    source: "PDF 目录 / 机器人",
    entryCount: 0,
  },
  {
    id: "pneumatic-actuator",
    chapter: "气动执行元件",
    shape: "计算向导 + 规则选型",
    requirement: "气缸、手指气缸、滑台气缸、夹持力、负载率",
    status: "planned",
    source: "PDF 目录 / 气动执行元件",
    entryCount: 0,
  },
  {
    id: "pneumatic-control",
    chapter: "气动控制（调速阀）",
    shape: "规则选型",
    requirement: "调速阀、节流、排气、气路控制适用判断",
    status: "planned",
    source: "PDF 目录 / 气动控制",
    entryCount: 0,
  },
  {
    id: "linear-guide",
    chapter: "直线导轨",
    shape: "计算向导 + 规则选型",
    requirement: "负载、安装姿态、滑块数量、静/动载荷初筛",
    status: "planned",
    source: "PDF 目录 / 直线导轨",
    entryCount: 0,
  },
  {
    id: "linear-bearing",
    chapter: "直线轴承",
    shape: "规则选型 + 计算向导",
    requirement: "轴承类型、负载、速度、精度和适用场景",
    status: "planned",
    source: "PDF 目录 / 直线轴承",
    entryCount: 0,
  },
  {
    id: "rolling-bearing",
    chapter: "滚动轴承",
    shape: "计算向导 + 规则选型",
    requirement: "径向/轴向载荷、转速、寿命/系数、样册参数匹配",
    status: "planned",
    source: "PDF 目录 / 滚动轴承",
    entryCount: 0,
  },
  {
    id: "coupling",
    chapter: "联轴器",
    shape: "规则选型 + 计算向导",
    requirement: "类型选择、扭矩、惯量、偏差补偿、适用场景",
    status: "planned",
    source: "PDF 目录 / 联轴器",
    entryCount: 0,
  },
  {
    id: "cable-chain",
    chapter: "拖链",
    shape: "规则选型",
    requirement: "行程、弯曲半径、线缆数量、安装空间、速度适配",
    status: "planned",
    source: "PDF 目录 / 拖链",
    entryCount: 0,
  },
  {
    id: "sensor",
    chapter: "传感器",
    shape: "规则选型",
    requirement: "类型、检测对象、距离、环境、安装方式",
    status: "planned",
    source: "PDF 目录 / 传感器",
    entryCount: 0,
  },
  {
    id: "material",
    chapter: "材料",
    shape: "规则选型",
    requirement: "强度、耐磨、耐腐蚀、食品医药等适用判断",
    status: "planned",
    source: "PDF 目录 / 材料",
    entryCount: 0,
  },
  {
    id: "machining",
    chapter: "机加工",
    shape: "规则选型",
    requirement: "加工方式、精度、成本、材料适配和注意事项",
    status: "planned",
    source: "PDF 目录 / 机加工",
    entryCount: 0,
  },
  {
    id: "heat-surface",
    chapter: "热处理&表面处理",
    shape: "规则选型",
    requirement: "硬度、耐磨、防腐、外观需求",
    status: "planned",
    source: "PDF 目录 / 热处理&表面处理",
    entryCount: 0,
  },
  {
    id: "hardware",
    chapter: "常用五金件",
    shape: "规则选型",
    requirement: "紧固件/五金件类型、规格、适用场景",
    status: "planned",
    source: "PDF 目录 / 常用五金件",
    entryCount: 0,
  },
];
