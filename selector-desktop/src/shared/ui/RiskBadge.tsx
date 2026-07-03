import type { ReactNode } from "react";
import { AlertTriangle, CheckCircle2, CircleDashed, XCircle } from "lucide-react";
import "./risk-badge.css";

export type RiskTone = "success" | "warning" | "danger" | "neutral";

interface RiskBadgeProps {
  tone: RiskTone;
  children: ReactNode;
}

const ICONS = {
  success: CheckCircle2,
  warning: AlertTriangle,
  danger: XCircle,
  neutral: CircleDashed,
} satisfies Record<RiskTone, typeof CheckCircle2>;

export function RiskBadge({ tone, children }: RiskBadgeProps) {
  const Icon = ICONS[tone];

  return (
    <span className={`risk-badge risk-badge--${tone}`}>
      <Icon size={14} aria-hidden="true" />
      <span>{children}</span>
    </span>
  );
}
