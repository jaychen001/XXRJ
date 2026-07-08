use super::models::ModuleDefinition;

#[cfg(test)]
pub const TIMING_BELT_MODULE_ID: &str = crate::modules::transmission::timing_belt::MODULE_ID;

pub fn module_definitions() -> Vec<ModuleDefinition> {
    crate::modules::module_definitions()
}

pub fn get_module_definition(module_id: &str) -> Option<ModuleDefinition> {
    module_definitions()
        .into_iter()
        .find(|definition| definition.id == module_id)
}
