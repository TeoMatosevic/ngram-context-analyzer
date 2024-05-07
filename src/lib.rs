use std::collections::HashMap;
use std::sync::Arc;

pub mod db;
pub mod error_handler;

/// This module contains the n-grams of the application.
///
/// # Modules
///
/// * `three_grams` - Contains the three-grams.
/// * `two_grams` - Contains the two-grams.
pub mod n_grams;

/// Represents the application data.
///
/// # Fields
///
/// * `scy_session` - The ScyllaDB session.
///
/// This struct is used to store the application data.
pub struct AppData {
    pub scy_session: Arc<scylla::Session>,
}

/// Parses the varying indexes from the query.
///
/// # Arguments
///
/// * `vary` - The varying indexes.
/// * `validate` - The validation function.
///
/// # Returns
///
/// A `Result` containing the `Vec<i32>` if the indexes are valid, otherwise a `String` with the error message.
///
/// # Errors
///
/// If the indexes are invalid, a `String` with the error message will be returned
pub fn parse_varying_indexes(
    vary: &str,
    validate: fn(&Vec<i32>) -> Result<(), String>,
) -> Result<Vec<i32>, String> {
    let indexes_str = vary.split(",").collect::<Vec<&str>>();
    let mut indexes: Vec<i32> = vec![];

    for index in indexes_str {
        match index.parse::<i32>() {
            Ok(index) => indexes.push(index),
            Err(_) => return Err("Invalid index".to_string()),
        }
    }

    match validate(&indexes) {
        Ok(_) => Ok(indexes),
        Err(err) => Err(err),
    }
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

/// Parses the query parameters.
pub trait ParseQueryParams: Sized {
    /// Creates a new instance of the struct from the given query.
    ///
    /// # Arguments
    ///
    /// * `query` - The query.
    ///
    /// # Returns
    ///
    /// A `Result` containing the struct if the query is valid, otherwise a `String` with the error message.
    ///
    /// # Errors
    ///
    /// If the query is invalid, a `String` with the error message will be returned.
    fn from(query: &HashMap<String, String>) -> Result<Self, String>;
}

/// Parses the query parameters.
///
/// # Arguments
///
/// * `query` - The query.
///
/// # Returns
///
/// A `Result` containing the struct if the query is valid, otherwise a `String` with the error message.
pub fn parse<T: ParseQueryParams>(query: &HashMap<String, String>) -> Result<T, String> {
    T::from(query)
}

#[cfg(test)]
mod tests {
    use super::*;

    const MOCK_VALIDATION_FN: fn(&Vec<i32>) -> Result<(), String> = |_indexes: &Vec<i32>| Ok(());

    #[test]
    fn test_parse_varying_indexes() {
        let indexes = parse_varying_indexes("1,2,3", MOCK_VALIDATION_FN);

        assert_eq!(indexes, Ok(vec![1, 2, 3]));
    }

    #[test]
    fn test_parse_varying_indexes_invalid_index() {
        let indexes = parse_varying_indexes("1,2,3a", MOCK_VALIDATION_FN);

        assert_eq!(indexes.is_err(), true);
    }

    #[test]
    fn test_parse_amount() {
        let amount = parse_amount("1");

        assert_eq!(amount, Ok(1));
    }

    #[test]
    fn test_parse_amount_invalid_amount() {
        let amount = parse_amount("1a");

        assert_eq!(amount.is_err(), true);
    }
}
