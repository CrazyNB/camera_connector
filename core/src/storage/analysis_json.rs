use crate::{TechnicalAssessmentPolicy, TechnicalDefectFlag};

use super::sqlite_data_error;

pub(super) fn technical_defect_flags_json(
    value: &[TechnicalDefectFlag],
) -> std::result::Result<String, rusqlite::Error> {
    serde_json::to_string(value).map_err(|error| sqlite_data_error(error.to_string()))
}

pub(super) fn technical_defect_flags_from_json(
    value: String,
) -> std::result::Result<Vec<TechnicalDefectFlag>, rusqlite::Error> {
    serde_json::from_str(&value).map_err(|error| sqlite_data_error(error.to_string()))
}

pub(super) fn technical_assessment_policy_json(
    value: Option<&TechnicalAssessmentPolicy>,
) -> std::result::Result<Option<String>, rusqlite::Error> {
    value
        .map(serde_json::to_string)
        .transpose()
        .map_err(|error| sqlite_data_error(error.to_string()))
}

pub(super) fn technical_assessment_policy_from_json(
    value: Option<String>,
) -> std::result::Result<Option<TechnicalAssessmentPolicy>, rusqlite::Error> {
    value
        .filter(|raw| !raw.trim().is_empty())
        .map(|raw| serde_json::from_str(&raw))
        .transpose()
        .map_err(|error| sqlite_data_error(error.to_string()))
}

pub(super) fn string_vec_json(value: &[String]) -> std::result::Result<String, rusqlite::Error> {
    serde_json::to_string(value).map_err(|error| sqlite_data_error(error.to_string()))
}

pub(super) fn string_vec_from_json(
    value: String,
) -> std::result::Result<Vec<String>, rusqlite::Error> {
    serde_json::from_str(&value).map_err(|error| sqlite_data_error(error.to_string()))
}
