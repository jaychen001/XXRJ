import { Calculator, FileText, ListChecks } from "lucide-react";
import type { CoverageItem } from "../../domain/coverage";
import "./mechanical-transmission-page.css";

interface MechanicalTransmissionPageProps {
  item: CoverageItem;
}

interface ChapterProfile {
  calculations: string[];
  rules: string[];
  knowledge: string[];
  source: string;
}

const CHAPTER_PROFILES: Record<string, ChapterProfile> = {
  "v-belt": {
    calculations: ["V 带选型计算"],
    rules: ["带型建议", "带速区间", "工况系数", "传动效率"],
    knowledge: ["V 带章节入口", "带轮直径、转速、功率和效率判断", "带型和根数待样本库导入后匹配"],
    source: "PDF P40 / 文档页 37 / V 带",
  },
  gear: {
    calculations: ["齿轮参数计算"],
    rules: ["齿数风险", "齿宽比例", "模数", "中心距"],
    knowledge: ["齿轮章节入口", "模数、齿数、中心距、减速比计算链路", "材料和强度校核待样本库导入后匹配"],
    source: "PDF P44 / 文档页 41 / 齿轮",
  },
  chain: {
    calculations: ["链条选型计算"],
    rules: ["小链轮齿数", "链速判断", "链节数", "传动比"],
    knowledge: ["链条章节入口", "节距、链号、中心距、速度、负载判断", "润滑和链型待样本库导入后匹配"],
    source: "PDF P49 / 文档页 46 / 链条",
  },
  "cam-indexer": {
    calculations: ["凸轮分割器选型计算"],
    rules: ["工位数", "冲击风险", "输出转速", "设计扭矩"],
    knowledge: ["凸轮分割器章节入口", "工位数、节拍、转速、扭矩/惯量计算链路", "分割器型号待样本库导入后匹配"],
    source: "PDF P59 / 文档页 56 / 凸轮分割器",
  },
  "brake-clutch": {
    calculations: ["制动器/离合器选型"],
    rules: ["类型建议", "热负荷", "响应时间", "设计扭矩"],
    knowledge: ["制动器/离合器章节入口", "扭矩、响应需求、适用工况判断", "热容量和寿命待样本库导入后匹配"],
    source: "PDF P65 / 文档页 62 / 制动器/离合器",
  },
};

export function MechanicalTransmissionPage({ item }: MechanicalTransmissionPageProps) {
  const profile = CHAPTER_PROFILES[item.id] ?? fallbackProfile(item);

  return (
    <section className="mechanical-entry" aria-labelledby="mechanical-entry-title">
      <header>
        <div>
          <h3 id="mechanical-entry-title">{item.chapter} · 机械传动入口</h3>
          <p>{item.requirement}</p>
        </div>
        <span>{profile.source}</span>
      </header>
      <div className="mechanical-entry__grid">
        <EntryColumn icon={Calculator} title="计算项" items={profile.calculations} />
        <EntryColumn icon={ListChecks} title="规则项" items={profile.rules} />
        <EntryColumn icon={FileText} title="知识引用" items={profile.knowledge} />
      </div>
    </section>
  );
}

interface EntryColumnProps {
  icon: typeof Calculator;
  title: string;
  items: string[];
}

function EntryColumn({ icon: Icon, title, items }: EntryColumnProps) {
  return (
    <article className="mechanical-entry__column">
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

function fallbackProfile(item: CoverageItem): ChapterProfile {
  return {
    calculations: ["后续章节包实现"],
    rules: [item.shape],
    knowledge: [item.source, item.notes ?? item.requirement],
    source: item.source,
  };
}
