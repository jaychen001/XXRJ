use super::models::ModuleDefinition;

pub const TIMING_BELT_MODULE_ID: &str = crate::modules::transmission::timing_belt::MODULE_ID;

pub fn module_definitions() -> Vec<ModuleDefinition> {
    let mut definitions = crate::modules::module_definitions();
    definitions.extend([
        planned_module("cylinder", "气缸", "气动", "气动执行元件"),
        planned_module("vacuum", "真空吸附", "气动", "气动执行元件"),
        planned_module("solenoid-valve", "电磁阀", "气动", "气动控制（调速阀）"),
    ]);
    definitions
}

pub fn get_module_definition(module_id: &str) -> Option<ModuleDefinition> {
    module_definitions()
        .into_iter()
        .find(|definition| definition.id == module_id)
}

fn planned_module(id: &str, name: &str, category: &str, chapter: &str) -> ModuleDefinition {
    ModuleDefinition {
        id: id.to_string(),
        name: name.to_string(),
        category: category.to_string(),
        description: "已进入模块清单，计算公式按后续章节包落地。".to_string(),
        source_chapter: chapter.to_string(),
        source_page: format!("根目录 PDF / {chapter}"),
        fields: Vec::new(),
    }
}
