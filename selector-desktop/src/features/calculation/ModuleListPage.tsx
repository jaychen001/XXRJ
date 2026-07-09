import { Calculator, Search } from "lucide-react";
import type { ModuleDefinition } from "../../domain/calculation";

interface ModuleListPageProps {
  modules: ModuleDefinition[];
  selectedModuleId: string;
  searchQuery: string;
  onSearchQueryChange: (query: string) => void;
  onSelectModule: (moduleId: string) => void;
}

export function ModuleListPage({
  modules,
  selectedModuleId,
  searchQuery,
  onSearchQueryChange,
  onSelectModule,
}: ModuleListPageProps) {
  const normalizedQuery = searchQuery.trim().toLowerCase();
  const filteredModules = normalizedQuery
    ? modules.filter((module) =>
        `${module.name} ${module.category} ${module.description}`.toLowerCase().includes(
          normalizedQuery,
        ),
      )
    : modules;

  return (
    <section className="module-list" aria-label="计算对象列表">
      <div className="module-list__header">
        <div>
          <h2>计算对象</h2>
          <span>{filteredModules.length} 个匹配项</span>
        </div>
        <label className="module-search">
          <Search size={15} aria-hidden="true" />
          <input
            aria-label="搜索计算对象"
            placeholder="搜索电机、气缸、丝杆..."
            value={searchQuery}
            onChange={(event) => onSearchQueryChange(event.target.value)}
          />
        </label>
      </div>
      <div className="module-list__items">
        {filteredModules.map((module) => (
          <button
            className={`module-card${module.id === selectedModuleId ? " module-card--active" : ""}`}
            key={module.id}
            type="button"
            aria-current={module.id === selectedModuleId ? "true" : undefined}
            onClick={() => onSelectModule(module.id)}
          >
            <span className="module-card__icon">
              <Calculator size={16} aria-hidden="true" />
            </span>
            <span className="module-card__body">
              <span>{module.category}</span>
              <strong>{module.name}</strong>
              <em>{module.description}</em>
            </span>
            <small>{module.fields.length > 0 ? `${module.fields.length} 项参数` : "待补公式"}</small>
          </button>
        ))}
        {filteredModules.length === 0 ? <p>没有匹配的计算对象，请清空搜索。</p> : null}
      </div>
    </section>
  );
}
