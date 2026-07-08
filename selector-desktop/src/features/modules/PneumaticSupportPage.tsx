import { Calculator, FileText, ListChecks } from "lucide-react";
import type { CoverageItem } from "../../domain/coverage";
import "./pneumatic-support-page.css";

interface PneumaticSupportPageProps {
  item: CoverageItem;
}

interface ChapterProfile {
  calculations: string[];
  rules: string[];
  knowledge: string[];
  source: string;
}

const CHAPTER_PROFILES: Record<string, ChapterProfile> = {
  "pneumatic-actuator": {
    calculations: ["气缸", "手指气缸", "滑台气缸", "真空吸附"],
    rules: ["负载率修正", "夹持力", "滑台推力", "吸盘面积"],
    knowledge: ["气动执行元件章节入口", "气缸、手指气缸、滑台气缸和真空吸附初筛", "样本库导入后匹配缸径、行程和吸盘型号"],
    source: "PDF P69 / 文档页 66 / 气动执行元件",
  },
  "pneumatic-control": {
    calculations: ["电磁阀"],
    rules: ["调速阀方式", "阀口径初筛", "耗气量", "动作频率"],
    knowledge: ["气动控制章节入口", "调速阀、节流、排气和气路控制判断", "电磁阀流量参数待样本库导入后匹配"],
    source: "PDF P88 / 文档页 85 / 气动控制",
  },
  "linear-guide": {
    calculations: ["直线导轨"],
    rules: ["滑块数量", "载荷余量", "偏载力矩", "安装姿态"],
    knowledge: ["直线导轨章节入口", "负载、滑块数量、静/动载荷初筛", "导轨规格和力矩额定值待样本库导入后匹配"],
    source: "PDF P103 / 文档页 100 / 直线导轨",
  },
  "linear-bearing": {
    calculations: ["直线轴承"],
    rules: ["载荷余量", "速度指标", "类型判断", "轴径"],
    knowledge: ["直线轴承章节入口", "轴承类型、负载、速度和精度判断", "额定载荷待样本库导入后匹配"],
    source: "PDF P104 / 文档页 101 / 直线轴承",
  },
  "rolling-bearing": {
    calculations: ["滚动轴承"],
    rules: ["等效动载荷", "寿命小时", "动额定载荷", "样册匹配需求"],
    knowledge: ["滚动轴承章节入口", "径向/轴向载荷、转速、寿命和系数计算", "C 值、内径和极限转速待样本库导入后匹配"],
    source: "PDF P109 / 文档页 106 / 滚动轴承",
  },
  coupling: {
    calculations: ["联轴器"],
    rules: ["类型建议", "设计扭矩", "扭转需求指标", "偏差补偿"],
    knowledge: ["联轴器章节入口", "扭矩、惯量、偏差补偿和适用场景判断", "额定扭矩和允许偏差待样本库导入后匹配"],
    source: "PDF P116 / 文档页 113 / 联轴器",
  },
};

export function PneumaticSupportPage({ item }: PneumaticSupportPageProps) {
  const profile = CHAPTER_PROFILES[item.id] ?? fallbackProfile(item);

  return (
    <section className="pneumatic-entry" aria-labelledby="pneumatic-entry-title">
      <header>
        <div>
          <h3 id="pneumatic-entry-title">{item.chapter} · 气动与支撑入口</h3>
          <p>{item.requirement}</p>
        </div>
        <span>{profile.source}</span>
      </header>
      <div className="pneumatic-entry__grid">
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
    <article className="pneumatic-entry__column">
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
