use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

use rusqlite::Connection;

use super::field_mapping::{build_models, validate_confirm_request};
use super::import_job;
use super::models::{
    ConfirmVendorImportRequest, NewVendorLibrary, RecommendationRequest, RecommendationRequirement,
    VendorImportPreviewRequest,
};
use super::recommendation::recommend_models;
use super::repository::{next_id, VendorRepository};

#[test]
fn csv_preview_requires_confirmed_mapping_before_import_and_recommendation() {
    let sample_path = write_sample_csv();
    let preview = import_job::build_preview(&VendorImportPreviewRequest {
        source_file: sample_path.to_string_lossy().to_string(),
        source_format: "csv".to_string(),
    })
    .expect("csv preview");
    assert_eq!(preview.rows.len(), 2);
    assert_eq!(preview.failed_rows.len(), 0);
    assert_eq!(preview.rows[0].model_name, "SV-400");
    assert!(preview.confidence > 0.8);

    let mut request = ConfirmVendorImportRequest {
        library_name: "伺服 CSV 样本".to_string(),
        component_type: "伺服/步进电机".to_string(),
        version_name: "v1".to_string(),
        confirmed: false,
        mappings: preview.suggested_mappings.clone(),
        preview,
    };
    assert!(validate_confirm_request(&request).is_err());
    request.confirmed = true;
    validate_confirm_request(&request).expect("confirmed mapping");

    let connection = Connection::open_in_memory().expect("database");
    connection
        .execute_batch(include_str!("../../migrations/0001_init.sql"))
        .expect("schema");
    let repository = VendorRepository::new(&connection);
    let library_id = next_id("vendor-library");
    repository
        .insert_library(&NewVendorLibrary {
            id: library_id.clone(),
            name: request.library_name.clone(),
            component_type: request.component_type.clone(),
            source_file: request.preview.source_file.clone(),
            source_format: request.preview.source_format.clone(),
            version_name: request.version_name.clone(),
        })
        .expect("library");
    let models = build_models(&request, &library_id);
    assert_eq!(models.len(), 2);
    repository.insert_models(&models).expect("models");

    let listed_models = repository
        .list_models(None, true, None)
        .expect("list models");
    let candidates = recommend_models(
        &RecommendationRequest {
            module_id: "timing-belt-basic".to_string(),
            component_type: None,
            limit: Some(5),
            requirements: vec![
                requirement("outputTorque", "输出扭矩", 0.351, "Nm"),
                requirement("requiredSpeed", "需求转速", 300.0, "rpm"),
            ],
        },
        listed_models,
    );
    assert_eq!(candidates.len(), 2);
    assert_eq!(candidates[0].model.model_name, "SV-400");
    assert_eq!(candidates[0].failed_rules.len(), 0);

    fs::remove_file(sample_path).ok();
}

fn write_sample_csv() -> std::path::PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or(0);
    let path = std::env::temp_dir().join(format!("vendor-sample-{stamp}.csv"));
    fs::write(
        &path,
        "型号,品牌,系列,额定扭矩(Nm),额定转速(rpm)\nSV-400,ACME,SV,0.5,3000\nSV-750,ACME,SV,0.9,3000\n",
    )
    .expect("write csv");
    path
}

fn requirement(
    id: &str,
    label: &str,
    value: f64,
    unit: &str,
) -> RecommendationRequirement {
    RecommendationRequirement {
        id: id.to_string(),
        label: label.to_string(),
        value,
        unit: unit.to_string(),
    }
}
