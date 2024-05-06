use super::super::word_freq_pair::DEFAULT_AMOUNT_OF_WORD_FREQ_PAIRS;
use crate::{
    db::{GET_BY_FIRST_2, GET_BY_SECOND_2, GET_FREQ_2},
    n_grams::{Printable, Queryable},
    parse_amount, parse_varying_indexes, ParseQueryParams,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents a two-gram.
///
/// # Fields
///
/// * `word1` - The first word of the two-gram.
/// * `word2` - The second word of the two-gram.
///
/// # Implements
///
/// * `Queryable` - Provides methods to query the database.
/// * `Printable` - Provides method for printing.   
#[derive(Clone, Serialize, Deserialize)]
pub struct TwoGramInput {
    pub word1: String,
    pub word2: String,
}

impl TwoGramInput {
    /// Creates a new `TwoGramInput` from the given query.
    ///
    /// # Arguments
    ///
    /// * `query` - The query that contains the two-gram.
    ///
    /// # Returns
    ///
    /// A `Result` containing the `TwoGramInput` if the query is valid, otherwise a `String` with the error message.
    pub fn from(query: &HashMap<String, String>) -> Result<TwoGramInput, String> {
        let word1 = match query.get("word1") {
            Some(word1) => word1,
            None => return Err("word1 is required".to_string()),
        };

        let word2 = match query.get("word2") {
            Some(word2) => word2,
            None => return Err("word2 is required".to_string()),
        };

        Ok(TwoGramInput {
            word1: word1.to_string(),
            word2: word2.to_string(),
        })
    }
}

impl Queryable for TwoGramInput {
    fn to_vec(&self) -> Vec<&str> {
        vec![&self.word1, &self.word2]
    }

    fn get_query(&self, index: Option<i32>) -> Result<&str, String> {
        match index {
            Some(index) => match index {
                1 => Ok(GET_BY_SECOND_2),
                2 => Ok(GET_BY_FIRST_2),
                _ => Err("Invalid index".to_string()),
            },
            None => Ok(GET_FREQ_2),
        }
    }

    fn get_input(&self, index: i32) -> Result<Vec<&String>, String> {
        match index {
            1 => Ok(vec![&self.word2]),
            2 => Ok(vec![&self.word1]),
            _ => Err("Invalid index".to_string()),
        }
    }

    fn get_word(&self, index: i32) -> Result<&String, String> {
        match index {
            1 => Ok(&self.word1),
            2 => Ok(&self.word2),
            _ => Err("Invalid index".to_string()),
        }
    }
}

impl Printable for TwoGramInput {
    fn print(&self) -> String {
        format!("{} {}", self.word1, self.word2)
    }
}

/// Represents the query parameters for a two-gram.
///
/// # Fields
///
/// * `two_gram` - The two-gram.
/// * `varying_indexes` - The indexes that will vary.
/// * `amount` - The amount of word frequency pairs to return.
///
/// # Implements
///
/// * `ParseQueryParams` - Provides methods to parse the query parameters.
pub struct TwoGramQueryParams {
    pub two_gram: TwoGramInput,
    pub varying_indexes: Option<Vec<i32>>,
    pub amount: i32,
}

impl ParseQueryParams for TwoGramQueryParams {
    fn from(query: &HashMap<String, String>) -> Result<TwoGramQueryParams, String> {
        let two_gram = match TwoGramInput::from(query) {
            Ok(two_gram) => two_gram,
            Err(error) => return Err(error),
        };

        let varying_indexes = match query.get("vary") {
            Some(vary) => match parse_varying_indexes(vary, validate) {
                Ok(indexes) => Some(indexes),
                Err(err) => return Err(err),
            },
            None => None,
        };

        let amount = match query.get("amount") {
            Some(amount) => match parse_amount(amount) {
                Ok(amount) => amount,
                Err(err) => return Err(err),
            },
            None => DEFAULT_AMOUNT_OF_WORD_FREQ_PAIRS,
        };

        Ok(TwoGramQueryParams {
            two_gram,
            varying_indexes,
            amount,
        })
    }
}

/// Validates the indexes.
///
/// # Arguments
///
/// * `indexes` - The indexes to validate.
///
/// # Returns
///
/// A `Result` containing `()` if the indexes are valid, otherwise a `String` with the error message.
fn validate(indexes: &Vec<i32>) -> Result<(), String> {
    let mut new = vec![];
    for index in indexes {
        if *index < 1 || *index > 2 {
            return Err("Invalid index".to_string());
        }
        if new.contains(index) {
            return Err("Invalid index".to_string());
        }
        new.push(*index);
    }
    if new.len() != indexes.len() {
        return Err("Invalid index".to_string());
    }
    Ok(())
}
