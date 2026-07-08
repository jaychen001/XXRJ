import { Calculator, FileText, ListChecks } from "lucide-react";
import type { CoverageItem } from "../../domain/coverage";
import "./drive-module-page.css";

interface DriveModulePageProps {
  item: CoverageItem;
}

interface ChapterProfile {
  calculations: string[];
  rules: string[];
  knowledge: string[];
  source: string;
}

const CHAPTER_PROFILES: Record<string, ChapterProfile> = {
  motor: {
    calculations: ["通用电机功率计算", "伺服/步进选型计算"],
    rules: ["功率余量", "惯量比", "单脉冲分辨率", "安全系数确认"],
    knowledge: ["电机篇目录入口", "通用电机、调速电机、普通电机初筛", "候选电机样本参数待导入后匹配"],
    source: "PDF P4 / 文档页 1 / 电机篇",
  },
  "ball-screw": {
    calculations: ["滚珠丝杠伺服计算"],
    rules: ["临界转速", "丝杠效率", "导程与转速匹配", "安全系数确认"],
    knowledge: ["丝杆篇目录入口", "导程、效率、惯量、力矩计算链路", "样本额定动载和支撑方式待导入后匹配"],
    source: "PDF P25 / 文档页 22 / 丝杆篇",
  },
  "timing-belt": {
    calculations: ["同步带基础计算"],
    rules: ["速度区间", "传动效率", "齿数齿距换算", "安全系数确认"],
    knowledge: ["同步带目录入口", "同步轮齿数、齿距、速度、扭矩计算链路", "齿形和带宽待样本库导入后匹配"],
    source: "PDF P34 / 文档页 31 / 同步带",
  },
  reducer: {
    calculations: ["减速机基础计算"],
    rules: ["减速比区间", "输入扭矩", "输出扭矩", "效率确认"],
    knowledge: ["减速机目录入口", "输出转速、输出扭矩、效率计算链路", "减速机类型和回程间隙待样本库匹配"],
    source: "PDF P54 / 文档页 51 / 减速机",
  },
  "linear-module": {
    calculations: ["直线模组选型判断"],
    rules: ["丝杆模组", "同步带模组", "常规直线模组", "精度与行程判断"],
    knowledge: ["直线模组目录入口", "行程、速度、精度、推力计算链路", "导轨和模组系列待样本库匹配"],
    source: "PDF P57 / 文档页 54 / 直线模组",
  },
};

export function DriveModulePage({ item }: DriveModulePageProps) {
  const profile = CHAPTER_PROFILES[item.id] ?? fallbackProfile(item);

  return (
    <section className="drive-module-entry" aria-labelledby="drive-module-entry-title">
      <header>
        <div>
          <h3 id="drive-module-entry-title">{item.chapter} · 计算与规则入口</h3>
          <p>{item.requirement}</p>
        </div>
        <span>{profile.source}</span>
      </header>
      <div className="drive-module-entry__grid">
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
    <article className="drive-module-entry__column">
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
