use std::{collections::HashMap, sync::Arc};

/// This module contains functions that handle the database operations.
pub mod db;

/// This module contains the error handler.
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
/// * `confusion_set` - The confusion set.
/// * `number_of_ngrams` - The number of n-grams.
///
/// This struct is used to store the application data.
pub struct AppData {
    pub scy_session: Arc<scylla::Session>,
    pub confusion_set: Vec<Vec<String>>,
    pub number_of_ngrams: HashMap<i32, i64>,
}

/// Represents the form data.
///
/// # Fields
///
/// * `text` - The text.
///
/// This struct is used to store the form data.
#[derive(serde::Deserialize)]
pub struct FormData {
    pub text: String,
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

/// Parses the n from the query.
///
/// # Arguments
///
/// * `query` - The query.
///
/// # Returns
///
/// A `Result` containing the `i32` if the n is valid, otherwise a `String` with the error message.
pub fn parse_n(query: &HashMap<String, String>) -> Result<i32, String> {
    match query.get("n") {
        Some(n) => match n.parse::<i32>() {
            Ok(n) => Ok(n),
            Err(_) => Err("Invalid n".to_string()),
        },
        None => Err("Missing n".to_string()),
    }
}

/// Parses the confusion set.
///
/// # Arguments
///
/// * `contents` - The contents of the confusion set.
///
/// # Returns
///
/// A `Vec<Vec<String>>` containing the confusion set.
pub fn parse_confusion_set(contents: String) -> Vec<Vec<String>> {
    let mut confusion_set: Vec<Vec<String>> = vec![];

    for line in contents.lines() {
        let words = line.split_whitespace().map(|s| s.to_string()).collect();
        confusion_set.push(words);
    }

    confusion_set
}

/// Parses the number of n-grams.
///
/// # Arguments
///
/// * `contents` - The contents of the number of n-grams.
///
/// # Returns
///
/// A `HashMap<i32, i64>` containing the number of n-grams.
pub fn parse_number_of_ngrams(contents: String) -> HashMap<i32, i64> {
    let mut number_of_ngrams: HashMap<i32, i64> = HashMap::new();

    for line in contents.lines() {
        let parts = line.split_whitespace().collect::<Vec<&str>>();
        let n = parts[0].parse::<i32>().unwrap();
        let number = parts[1].parse::<i64>().unwrap();
        number_of_ngrams.insert(n, number);
    }

    let valid_n_values = vec![1, 2, 3];

    for n in number_of_ngrams.keys() {
        if !valid_n_values.contains(n) {
            panic!("Invalid ngram in number of ngrams file");
        }
    }

    number_of_ngrams
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
