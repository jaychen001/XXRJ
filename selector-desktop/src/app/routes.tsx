import type { LucideIcon } from "lucide-react";
import {
  ClipboardList,
  Database,
  FileSearch,
  FileText,
  Gauge,
  Library,
  Search,
  Settings,
  TestTube2,
} from "lucide-react";

export type AppRouteId =
  | "coverage"
  | "calculation"
  | "cases"
  | "vendor"
  | "parameters"
  | "reports"
  | "knowledge"
  | "qa"
  | "settings";

export interface AppRoute {
  id: AppRouteId;
  label: string;
  description: string;
  icon: LucideIcon;
  priority: "P0" | "P1";
}

export const APP_ROUTES: AppRoute[] = [
  {
    id: "coverage",
    label: "PDF 覆盖矩阵",
    description: "23 章入口与实现状态",
    icon: ClipboardList,
    priority: "P0",
  },
  {
    id: "calculation",
    label: "选型计算",
    description: "工况输入与过程分析",
    icon: Gauge,
    priority: "P0",
  },
  {
    id: "cases",
    label: "案例库",
    description: "历史计算复用",
    icon: Library,
    priority: "P0",
  },
  {
    id: "vendor",
    label: "厂家样本库",
    description: "PDF / Excel / CSV 导入",
    icon: Database,
    priority: "P0",
  },
  {
    id: "parameters",
    label: "内部参数库",
    description: "经验参数维护",
    icon: FileText,
    priority: "P0",
  },
  {
    id: "reports",
    label: "报告导出",
    description: "PDF / Excel 报告",
    icon: FileSearch,
    priority: "P0",
  },
  {
    id: "knowledge",
    label: "知识检索",
    description: "根目录 PDF 来源查询",
    icon: Search,
    priority: "P1",
  },
  {
    id: "qa",
    label: "QA 覆盖检查",
    description: "覆盖、回归与打包验收",
    icon: TestTube2,
    priority: "P0",
  },
  {
    id: "settings",
    label: "设置",
    description: "数据路径和单位偏好",
    icon: Settings,
    priority: "P1",
  },
];

export function getAppRoute(routeId: AppRouteId): AppRoute {
  return APP_ROUTES.find((route) => route.id === routeId) ?? APP_ROUTES[0];
}
