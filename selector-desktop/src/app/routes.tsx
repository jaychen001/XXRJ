import type { LucideIcon } from "lucide-react";
import { Gauge } from "lucide-react";

export type AppRouteId = "calculation";

export interface AppRoute {
  id: AppRouteId;
  label: string;
  description: string;
  icon: LucideIcon;
}

export const APP_ROUTES: AppRoute[] = [
  {
    id: "calculation",
    label: "选型计算",
    description: "输入工况参数，输出公式过程和选型结论",
    icon: Gauge,
  },
];

export function getAppRoute(routeId: AppRouteId): AppRoute {
  return APP_ROUTES.find((route) => route.id === routeId) ?? APP_ROUTES[0];
}
