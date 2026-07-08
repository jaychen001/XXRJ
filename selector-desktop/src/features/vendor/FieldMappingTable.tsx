import type { FieldMapping } from "../../domain/vendor";

const TARGET_FIELDS = [
  ["modelName", "型号"],
  ["brand", "品牌"],
  ["series", "系列"],
  ["outputTorque", "输出扭矩"],
  ["requiredSpeed", "需求转速"],
  ["power", "功率"],
  ["bore", "缸径"],
  ["stroke", "行程"],
  ["load", "负载/推力"],
  ["vacuumPressure", "真空压力"],
  ["flowRate", "流量"],
  ["dynamicLoad", "动额定载荷"],
  ["staticLoad", "静额定载荷"],
] as const;

interface FieldMappingTableProps {
  mappings: FieldMapping[];
  onMappingsChange: (mappings: FieldMapping[]) => void;
}

export function FieldMappingTable({ mappings, onMappingsChange }: FieldMappingTableProps) {
  function updateMapping(index: number, patch: Partial<FieldMapping>) {
    onMappingsChange(
      mappings.map((mapping, currentIndex) =>
        currentIndex === index ? { ...mapping, ...patch } : mapping,
      ),
    );
  }

  return (
    <div className="vendor-mapping">
      <h3>字段映射</h3>
      <table aria-label="厂家样本字段映射">
        <thead>
          <tr>
            <th>来源字段</th>
            <th>目标字段</th>
            <th>单位</th>
          </tr>
        </thead>
        <tbody>
          {mappings.map((mapping, index) => (
            <tr key={`${mapping.sourceField}-${index}`}>
              <td>{mapping.sourceField}</td>
              <td>
                <select
                  aria-label={`目标字段 ${mapping.sourceField}`}
                  value={mapping.targetField}
                  onChange={(event) =>
                    updateMapping(index, { targetField: event.currentTarget.value })
                  }
                >
                  {TARGET_FIELDS.map(([value, label]) => (
                    <option key={value} value={value}>
                      {label}
                    </option>
                  ))}
                </select>
              </td>
              <td>
                <input
                  aria-label={`单位 ${mapping.sourceField}`}
                  value={mapping.unit ?? ""}
                  onChange={(event) => updateMapping(index, { unit: event.currentTarget.value })}
                />
              </td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}
