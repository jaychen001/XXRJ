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
        <h2>计算对象</h2>
        <input
          aria-label="搜索计算对象"
          placeholder="搜索电机、气缸、丝杆..."
          value={searchQuery}
          onChange={(event) => onSearchQueryChange(event.target.value)}
        />
      </div>
      <div className="module-list__items">
        {filteredModules.map((module) => (
          <button
            className={`module-card${module.id === selectedModuleId ? " module-card--active" : ""}`}
            key={module.id}
            type="button"
            onClick={() => onSelectModule(module.id)}
          >
            <span>{module.category}</span>
            <strong>{module.name}</strong>
            <small>{module.fields.length > 0 ? "可计算" : "待补公式"}</small>
          </button>
        ))}
        {filteredModules.length === 0 ? <p>没有匹配的计算对象，请清空搜索。</p> : null}
      </div>
    </section>
  );
}
