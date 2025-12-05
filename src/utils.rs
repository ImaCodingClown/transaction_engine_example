use crate::error::AppError;

pub fn validate_file_path(file_path: &str) -> Result<String, AppError> {
    if !file_path.ends_with(".csv") {
        return Err(AppError::InvalidFileFormat);
    }

    Ok(file_path.to_string())
}
