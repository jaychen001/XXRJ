import { FileText, ListChecks, Route } from "lucide-react";
import type { CoverageItem } from "../../domain/coverage";
import "./rule-module-page.css";

interface RuleModulePageProps {
  item: CoverageItem;
}

interface RuleProfile {
  questions: string[];
  rules: string[];
  notes: string[];
  source: string;
}

const RULE_PROFILES: Record<string, RuleProfile> = {
  robot: {
    questions: ["负载", "臂展", "节拍", "精度", "应用场景"],
    rules: ["推荐类型", "节拍判断", "精度风险"],
    notes: ["负载曲线和末端惯量仍需样本复核", "来源 PDF P67 / 文档页 64"],
    source: "PDF P67 / 文档页 64 / 机器人",
  },
  "cable-chain": {
    questions: ["行程", "弯曲半径", "线缆数量", "填充率", "速度"],
    rules: ["推荐类型", "填充率", "线缆管理"],
    notes: ["内腔尺寸和最小弯曲半径需按最大线缆复核", "来源 PDF P121 / 文档页 118"],
    source: "PDF P121 / 文档页 118 / 拖链",
  },
  sensor: {
    questions: ["检测对象", "检测距离", "响应时间", "环境等级", "安装空间"],
    rules: ["推荐类型", "响应风险", "环境适配"],
    notes: ["透明、反光、油污环境要现场验证", "来源 PDF P123 / 文档页 120"],
    source: "PDF P123 / 文档页 120 / 传感器",
  },
  material: {
    questions: ["强度需求", "耐磨需求", "耐腐蚀需求", "轻量化", "食品医药"],
    rules: ["推荐材料", "防腐风险", "加工适配"],
    notes: ["材料建议需同步考虑加工和表面处理", "来源 PDF P135 / 文档页 132"],
    source: "PDF P135 / 文档页 132 / 材料",
  },
  machining: {
    questions: ["公差需求", "批量", "材料硬度", "结构复杂度", "表面要求"],
    rules: ["加工方式", "成本风险", "质量注意"],
    notes: ["精度、热处理变形和装夹基准要一起确认", "来源 PDF P139 / 文档页 136"],
    source: "PDF P139 / 文档页 136 / 机加工",
  },
  "heat-surface": {
    questions: ["目标硬度", "耐磨需求", "防腐需求", "外观需求", "变形敏感"],
    rules: ["推荐处理", "变形风险", "外观注意"],
    notes: ["热处理和表面处理要预留加工余量", "来源 PDF P141 / 文档页 138"],
    source: "PDF P141 / 文档页 138 / 热处理&表面处理",
  },
  hardware: {
    questions: ["载荷", "振动等级", "调节频率", "安装空间", "拆装需求"],
    rules: ["推荐五金件", "防松风险", "调节维护"],
    notes: ["规格需按螺纹、材料和安装工具空间复核", "来源 PDF P146 / 文档页 143"],
    source: "PDF P146 / 文档页 143 / 常用五金件",
  },
};

export function RuleModulePage({ item }: RuleModulePageProps) {
  const profile = RULE_PROFILES[item.id] ?? fallbackProfile(item);

  return (
    <section className="rule-entry" aria-labelledby="rule-entry-title">
      <header>
        <div>
          <h3 id="rule-entry-title">{item.chapter} · 规则选型入口</h3>
          <p>{item.requirement}</p>
        </div>
        <span>{profile.source}</span>
      </header>
      <div className="rule-entry__grid">
        <EntryColumn icon={Route} title="工况问题" items={profile.questions} />
        <EntryColumn icon={ListChecks} title="输出规则" items={profile.rules} />
        <EntryColumn icon={FileText} title="注意事项" items={profile.notes} />
      </div>
    </section>
  );
}

interface EntryColumnProps {
  icon: typeof Route;
  title: string;
  items: string[];
}

function EntryColumn({ icon: Icon, title, items }: EntryColumnProps) {
  return (
    <article className="rule-entry__column">
      <h4>
        <Icon size={16} aria-hidden="true" />
        {title}
      </h4>
      <ul>
        {items.map((item) => (
          <li key={item}>{item}</li>
        ))}
      </ul>
    </article>
  );
}

function fallbackProfile(item: CoverageItem): RuleProfile {
  return {
    questions: [item.requirement],
    rules: [item.shape],
    notes: [item.source, item.notes ?? item.requirement],
    source: item.source,
  };
}
