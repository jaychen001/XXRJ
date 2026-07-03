import type { CaseDetailRecord, CaseRecord } from "../../domain/calculation";

interface CaseDetailPanelProps {
  selectedCase: CaseRecord | null;
  selectedDetail: CaseDetailRecord | null;
  editName: string;
  editNotes: string;
  onEditNameChange: (value: string) => void;
  onEditNotesChange: (value: string) => void;
  onSave: () => void;
  onOpenCalculation: () => void;
}

export function CaseDetailPanel({
  selectedCase,
  selectedDetail,
  editName,
  editNotes,
  onEditNameChange,
  onEditNotesChange,
  onSave,
  onOpenCalculation,
}: CaseDetailPanelProps) {
  return (
    <aside className="case-detail">
      <h2>{selectedCase?.name ?? "未选择案例"}</h2>
      <label>
        案例名称
        <input
          aria-label="详情案例名称"
          disabled={!selectedCase}
          value={editName}
          onChange={(event) => onEditNameChange(event.target.value)}
        />
      </label>
      <label>
        备注
        <textarea
          aria-label="详情备注"
          disabled={!selectedCase}
          value={editNotes}
          onChange={(event) => onEditNotesChange(event.target.value)}
        />
      </label>
      <div className="case-detail__actions">
        <button className="secondary-button" type="button" disabled={!selectedCase} onClick={onSave}>
          保存修改
        </button>
        <button
          className="secondary-button"
          type="button"
          disabled={!selectedCase}
          onClick={onOpenCalculation}
        >
          修改参数
        </button>
      </div>
      <dl>
        <div>
          <dt>模块</dt>
          <dd>{selectedCase?.moduleName ?? "-"}</dd>
        </div>
        <div>
          <dt>结果</dt>
          <dd>{selectedCase?.resultSummary ?? "-"}</dd>
        </div>
        <div>
          <dt>输入快照</dt>
          <dd>{formatInputs(selectedDetail)}</dd>
        </div>
        <div>
          <dt>风险</dt>
          <dd>{formatRisks(selectedDetail, selectedCase)}</dd>
        </div>
        <div>
          <dt>推荐型号</dt>
          <dd>样本库推荐接入后显示候选型号</dd>
        </div>
      </dl>
    </aside>
  );
}

function formatInputs(detail: CaseDetailRecord | null): string {
  if (!detail) {
    return "-";
  }
  const fields = detail.request.fields.map(
    (field) => `${fieldLabel(field.id)}=${field.value}${field.unit}`,
  );
  fields.push(`安全系数=${detail.request.safetyFactor ?? "-"}K`);
  return fields.join(" / ");
}

function formatRisks(detail: CaseDetailRecord | null, selectedCase: CaseRecord | null): string {
  if (!detail) {
    return selectedCase ? `${selectedCase.riskCount} 项` : "-";
  }
  return detail.result.risks.map((risk) => risk.message).join(" / ") || "无风险";
}

function fieldLabel(id: string): string {
  const labels: Record<string, string> = {
    loadMass: "负载质量",
    frictionCoefficient: "摩擦系数",
    targetSpeed: "目标速度",
    accelerationTime: "加速时间",
    pulleyTeeth: "同步轮齿数",
    toothPitch: "齿距",
    efficiency: "传动效率",
  };
  return labels[id] ?? id;
}
