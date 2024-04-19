mod vary_three_gram;
mod word_freq_pair;

use crate::parse_varying_indexes;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub use vary_three_gram::VaryingQueryResult;

/// Represents a three-gram.
///
/// # Fields
///
/// * `word1` - The first word of the three-gram.
/// * `word2` - The second word of the three-gram.
/// * `word3` - The third word of the three-gram.
/// * `freq` - The frequency of the three-gram.
#[derive(Serialize, Deserialize)]
pub struct ThreeGram {
    pub word1: String,
    pub word2: String,
    pub word3: String,
    pub freq: i32,
}

/// Represents the three gram that is given as input.
///
/// # Fields
///
/// * `word1` - The first word of the three-gram.
/// * `word2` - The second word of the three-gram.
/// * `word3` - The third word of the three-gram.
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

/// Represents the HTTP query input.
///
/// # Fields
///
/// * `three_gram` - The three-gram.
/// * `varying_indexes` - The varying indexes.
///
/// # Methods
///
/// * `from` - Creates a `HttpQueryInput` from the given query.
pub struct HttpQueryInput {
    pub three_gram: ThreeGramInput,
    pub varying_indexes: Option<Vec<i32>>,
}

impl HttpQueryInput {
    /// Creates a `HttpQueryInput` from the given query.
    ///
    /// # Arguments
    ///
    /// * `query` - The query that contains the three-gram.
    ///
    /// # Returns
    ///
    /// A `Result` containing the `HttpQueryInput` if the query is valid, otherwise a `String` with the error message.
    pub fn from(query: &HashMap<String, String>) -> Result<HttpQueryInput, String> {
        let three_gram = match ThreeGramInput::from(query) {
            Ok(three_gram) => three_gram,
            Err(err) => return Err(err),
        };

        let varying_indexes = match query.get("vary") {
            Some(vary) => match parse_varying_indexes(vary) {
                Ok(indexes) => Some(indexes),
                Err(err) => return Err(err),
            },
            None => None,
        };

        Ok(HttpQueryInput {
            three_gram,
            varying_indexes,
        })
    }
}
