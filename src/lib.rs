pub mod db;
pub mod error_handler;
/// This is the main library of the project. It contains the modules that are used to build the project.
///
/// # Modules
///
/// * `three_grams` - Contains the three-grams module.
/// * `error_handler` - Contains the error handler module.
/// * `db` - Contains the database module.
pub mod three_grams;

/// Parses the varying indexes from the query.
///
/// # Arguments
///
/// * `vary` - The varying indexes.
///
/// # Returns
///
/// A `Result` containing the `Vec<i32>` if the indexes are valid, otherwise a `String` with the error message.
///
/// # Errors
///
/// If the indexes are invalid, a `String` with the error message will be returned.
pub fn parse_varying_indexes(vary: &str) -> Result<Vec<i32>, String> {
    let indexes_str = vary.split(",").collect::<Vec<&str>>();
    let mut indexes: Vec<i32> = vec![];

    for index in indexes_str {
        match index.parse::<i32>() {
            Ok(index) => indexes.push(index),
            Err(_) => return Err("Invalid index".to_string()),
        }
    }

    for index in &indexes {
        if *index < 1 || *index > 3 {
            return Err("Invalid index".to_string());
        }
    }

    Ok(indexes)
}

/// Parses the amount from the query.
///
/// # Arguments
///
/// * `amount` - The amount.
///
/// # Returns
///
/// A `Result` containing the `i32` if the amount is valid, otherwise a `String` with the error message.
///
/// # Errors
///
/// If the amount is invalid, a `String` with the error message will be returned.
pub fn parse_amount(amount: &str) -> Result<i32, String> {
    match amount.parse::<i32>() {
        Ok(amount) => Ok(amount),
        Err(_) => Err("Invalid amount".to_string()),
    }
}
