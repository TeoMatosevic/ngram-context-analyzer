use super::super::word_freq_pair::DEFAULT_AMOUNT_OF_WORD_FREQ_PAIRS;
use crate::{
    db::{
        GET_BY_FIRST_AND_SECOND_3, GET_BY_FIRST_AND_THIRD_3, GET_BY_SECOND_AND_THIRD_3, GET_FREQ_3,
    },
    n_grams::{Printable, Queryable},
    parse_amount, parse_varying_indexes, ParseQueryParams,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents the three gram that is given as input.
///
/// # Fields
///
/// * `word1` - The first word of the three-gram.
/// * `word2` - The second word of the three-gram.
/// * `word3` - The third word of the three-gram.
///
/// # Implements
///
/// * `Queryable` - Provides methods to query the database.
/// * `Printable` - Provides method for printing.
#[derive(Serialize, Deserialize, Clone)]
pub struct ThreeGramInput {
    pub word1: String,
    pub word2: String,
    pub word3: String,
}

impl ThreeGramInput {
    /// Creates a new `ThreeGramInput` from the given query.
    ///
    /// # Arguments
    ///
    /// * `query` - The query that contains the three-gram.
    ///
    /// # Returns
    ///
    /// A `Result` containing the `ThreeGramInput` if the query is valid, otherwise a `String` with the error message.
    pub fn from(query: &HashMap<String, String>) -> Result<ThreeGramInput, String> {
        let word1 = match query.get("word1") {
            Some(word1) => word1,
            None => return Err("word1 is required".to_string()),
        };

        let word2 = match query.get("word2") {
            Some(word2) => word2,
            None => return Err("word2 is required".to_string()),
        };

        let word3 = match query.get("word3") {
            Some(word3) => word3,
            None => return Err("word3 is required".to_string()),
        };

        Ok(ThreeGramInput {
            word1: word1.to_string(),
            word2: word2.to_string(),
            word3: word3.to_string(),
        })
    }
}

impl Queryable for ThreeGramInput {
    fn to_vec(&self) -> Vec<&str> {
        vec![&self.word1, &self.word2, &self.word3]
    }

    fn get_query(&self, index: Option<i32>) -> Result<&str, String> {
        match index {
            Some(index) => match index {
                1 => Ok(GET_BY_SECOND_AND_THIRD_3),
                2 => Ok(GET_BY_FIRST_AND_THIRD_3),
                3 => Ok(GET_BY_FIRST_AND_SECOND_3),
                _ => Err("Invalid index".to_string()),
            },
            None => Ok(GET_FREQ_3),
        }
    }

    fn get_input(&self, index: i32) -> Result<Vec<&String>, String> {
        match index {
            1 => Ok(vec![&self.word2, &self.word3]),
            2 => Ok(vec![&self.word1, &self.word3]),
            3 => Ok(vec![&self.word1, &self.word2]),
            _ => Err("Invalid index".to_string()),
        }
    }

    fn get_word(&self, index: i32) -> Result<&String, String> {
        match index {
            1 => Ok(&self.word1),
            2 => Ok(&self.word2),
            3 => Ok(&self.word3),
            _ => Err("Invalid index".to_string()),
        }
    }
}

impl Printable for ThreeGramInput {
    fn print(&self) -> String {
        format!("{} {} {}", self.word1, self.word2, self.word3)
    }
}

/// Represents the query parameters for the three-gram.
///
/// # Fields
///
/// * `three_gram` - The three-gram.
/// * `varying_indexes` - The indexes that are varying.
/// * `amount` - The amount of words to return.
///
/// # Implements
///
/// * `ParseQueryParams` - Parses the query parameters.
pub struct ThreeGramQueryParams {
    pub three_gram: ThreeGramInput,
    pub varying_indexes: Option<Vec<i32>>,
    pub amount: i32,
}

impl ParseQueryParams for ThreeGramQueryParams {
    fn from(query: &HashMap<String, String>) -> Result<Self, String> {
        let three_gram = match ThreeGramInput::from(query) {
            Ok(three_gram) => three_gram,
            Err(err) => return Err(err),
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

        Ok(ThreeGramQueryParams {
            three_gram,
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
        if *index < 1 || *index > 3 {
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
