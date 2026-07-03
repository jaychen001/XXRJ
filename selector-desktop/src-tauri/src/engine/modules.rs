use super::models::{ModuleDefinition, ModuleField};

pub const TIMING_BELT_MODULE_ID: &str = "timing-belt-basic";

pub fn module_definitions() -> Vec<ModuleDefinition> {
    vec![
        timing_belt_module(),
        planned_module("servo-motor", "伺服电机", "驱动", "电机篇"),
        planned_module("stepper-motor", "步进电机", "驱动", "电机篇"),
        planned_module("cylinder", "气缸", "气动", "气动执行元件"),
        planned_module("vacuum", "真空吸附", "气动", "气动执行元件"),
        planned_module("solenoid-valve", "电磁阀", "气动", "气动控制（调速阀）"),
    ]
}

pub fn get_module_definition(module_id: &str) -> Option<ModuleDefinition> {
    module_definitions()
        .into_iter()
        .find(|definition| definition.id == module_id)
}

fn timing_belt_module() -> ModuleDefinition {
    ModuleDefinition {
        id: TIMING_BELT_MODULE_ID.to_string(),
        name: "同步带基础计算".to_string(),
        category: "传动".to_string(),
        description: "用于验证负载、速度、同步轮参数到扭矩和转速的过程输出。".to_string(),
        source_chapter: "同步带".to_string(),
        source_page: "根目录 PDF / 同步带匹配页".to_string(),
        fields: vec![
            field("loadMass", "负载质量", "kg", 0.0, Some(5.0), "移动负载质量"),
            field(
                "frictionCoefficient",
                "摩擦系数",
                "ratio",
                0.0,
                Some(0.1),
                "导向面或机构摩擦系数",
            ),
            field_with_units(
                "targetSpeed",
                "目标速度",
                "mm/s",
                &["mm/s", "m/s"],
                0.0,
                Some(500.0),
                "机构目标线速度",
            ),
            field_with_units(
                "accelerationTime",
                "加速时间",
                "s",
                &["s", "min"],
                0.001,
                Some(0.3),
                "从静止到目标速度的时间",
            ),
            field(
                "pulleyTeeth",
                "同步轮齿数",
                "teeth",
                1.0,
                Some(20.0),
                "驱动轮齿数",
            ),
            field_with_units(
                "toothPitch",
                "齿距",
                "mm",
                &["mm", "m"],
                0.001,
                Some(5.0),
                "同步带齿距",
            ),
            field(
                "efficiency",
                "传动效率",
                "ratio",
                0.01,
                Some(0.9),
                "0-1 之间的小数",
            ),
        ],
    }
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

fn field(
    id: &str,
    label: &str,
    unit: &str,
    min: f64,
    default_value: Option<f64>,
    helper: &str,
) -> ModuleField {
    field_with_units(id, label, unit, &[unit], min, default_value, helper)
}

fn field_with_units(
    id: &str,
    label: &str,
    unit: &str,
    unit_options: &[&str],
    min: f64,
    default_value: Option<f64>,
    helper: &str,
) -> ModuleField {
    ModuleField {
        id: id.to_string(),
        label: label.to_string(),
        unit: unit.to_string(),
        unit_options: unit_options
            .iter()
            .map(|item| (*item).to_string())
            .collect(),
        required: true,
        min: Some(min),
        default_value,
        helper: helper.to_string(),
        source: "根目录 PDF / 同步带".to_string(),
    }
}
