use rusqlite::Connection;

use super::models::CaseFilter;
use super::repository::CaseRepository;
use crate::engine;

#[test]
fn repository_saves_duplicates_reruns_and_deletes_case() {
    let connection = Connection::open_in_memory().expect("database");
    connection
        .execute_batch(include_str!("../../migrations/0001_init.sql"))
        .expect("schema");
    let repository = CaseRepository::new(&connection);
    let request = test_request();
    let module = engine::get_module_definition(&request.module_id).expect("module");
    let result = engine::calculate_request(&request).expect("calculation");
    let saved = repository
        .save_case("同步带测试", "", &module, &request, &result)
        .expect("save");
    assert_eq!(
        repository
            .list_cases(&CaseFilter::default())
            .expect("list")
            .len(),
        1
    );

    let renamed = repository
        .update_case(&saved.id, "同步带更新", "已复核")
        .expect("update")
        .expect("case exists");
    assert_eq!(renamed.name, "同步带更新");

    let filtered = repository
        .list_cases(&CaseFilter {
            query: Some("更新".to_string()),
            module_id: Some(module.id.clone()),
            created_from: None,
            created_to: None,
        })
        .expect("filtered list");
    assert_eq!(filtered.len(), 1);

    let duplicated = repository
        .duplicate_case(&saved.id)
        .expect("duplicate")
        .expect("case exists");
    assert_ne!(duplicated.id, saved.id);

    let mut copy_request = request.clone();
    set_input(&mut copy_request, "targetSpeed", 400.0, "mm/s");
    let copy_result = engine::calculate_request(&copy_request).expect("copy calculation");
    repository
        .append_run(&duplicated.id, &module, &copy_request, &copy_result)
        .expect("append copy run")
        .expect("copy still exists");

    let (_, original_request, _) = repository
        .load_case_payload(&saved.id)
        .expect("load original")
        .expect("original exists");
    let (_, copied_request, loaded_result) = repository
        .load_case_payload(&duplicated.id)
        .expect("load copy")
        .expect("copy exists");
    assert_eq!(field_value(&original_request, "targetSpeed"), 500.0);
    assert_eq!(field_value(&copied_request, "targetSpeed"), 400.0);
    assert_eq!(loaded_result.module_id, result.module_id);

    let rerun = repository
        .append_run(&saved.id, &module, &request, &result)
        .expect("rerun")
        .expect("case still exists");
    assert_eq!(rerun.id, saved.id);
    assert!(repository.delete_case(&saved.id).expect("delete"));
}

fn test_request() -> crate::engine::models::CalculationRequest {
    crate::engine::models::CalculationRequest {
        module_id: "timing-belt-basic".to_string(),
        safety_factor: Some(1.5),
        safety_factor_confirmed: true,
        fields: vec![
            input("loadMass", 5.0, "kg"),
            input("frictionCoefficient", 0.1, "ratio"),
            input("targetSpeed", 500.0, "mm/s"),
            input("accelerationTime", 0.3, "s"),
            input("pulleyTeeth", 20.0, "teeth"),
            input("toothPitch", 5.0, "mm"),
            input("efficiency", 0.9, "ratio"),
        ],
    }
}

fn input(id: &str, value: f64, unit: &str) -> crate::engine::models::FieldInput {
    crate::engine::models::FieldInput {
        id: id.to_string(),
        value,
        unit: unit.to_string(),
    }
}

fn set_input(
    request: &mut crate::engine::models::CalculationRequest,
    id: &str,
    value: f64,
    unit: &str,
) {
    let input = request
        .fields
        .iter_mut()
        .find(|field| field.id == id)
        .expect("field exists");
    input.value = value;
    input.unit = unit.to_string();
}

fn field_value(request: &crate::engine::models::CalculationRequest, id: &str) -> f64 {
    request
        .fields
        .iter()
        .find(|field| field.id == id)
        .map(|field| field.value)
        .expect("field exists")
}
