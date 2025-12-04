use crate::error::AppError;

pub fn validate_args(args: Vec<String>) -> Result<bool, AppError> {
    if args.len() < 2 {
        return Err(AppError::MissingFileArgument);
    }

    if args.len() > 2 && args[2] != "--batch" {
        return Err(AppError::WrongArgument(args[2].clone()));
    }

    if !&args[1].ends_with(".csv") {
        return Err(AppError::InvalidFileFormat);
    }

    Ok(true)
}
