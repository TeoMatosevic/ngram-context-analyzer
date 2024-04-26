use super::ThreeGramInput;
use crate::db::{
    QueryError, QueryFactory, GET_BY_FIRST_AND_SECOND, GET_BY_FIRST_AND_THIRD,
    GET_BY_SECOND_AND_THIRD,
};
use scylla::{statement::Consistency, IntoTypedRows, Session};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Represents a word and its frequency.
///
/// # Fields
///
/// * `word` - The word.
/// * `frequency` - The frequency of the word.
///
/// # Methods
///
/// * `new` - Creates a new `WordFreqPair`.
/// * `from` - Creates a `WordFreqPair` from the given session, index, and three-gram.
/// * `find` - Finds the word in the given vector of `WordFreqPair`.
#[derive(Serialize, Deserialize)]
pub struct WordFreqPair {
    pub word: String,
    pub frequency: i32,
}

impl WordFreqPair {
    /// Creates a new `WordFreqPair`.
    ///
    /// # Arguments
    ///
    /// * `word` - The word.
    /// * `frequency` - The frequency of the word.
    ///
    /// # Returns
    ///
    /// A `WordFreqPair`.
    pub fn new(word: String, frequency: i32) -> WordFreqPair {
        WordFreqPair { word, frequency }
    }

    /// Creates a `WordFreqPair` from the given session, index, and three-gram.
    ///
    /// # Arguments
    ///
    /// * `session` - The ScyllaDB session.
    /// * `index` - The index of the word.
    /// * `three_gram` - The three-gram.
    /// * `amount` - The amount of words to return.
    ///
    /// # Returns
    ///
    /// A `Result` containing the `Vec` of `WordFreqPair` if the query is successful, otherwise a `String` with the error message.
    ///
    /// # Errors
    ///
    /// If the query can not be executed, a `String` with the error message will be returned.
    /// If the word is not found, a `String` with the error message will be returned.
    /// If the index is invalid, a `String` with the error message will be returned.
    pub async fn from(
        session: Arc<Session>,
        index: &i32,
        three_gram: &ThreeGramInput,
        amount: i32,
    ) -> Result<Vec<WordFreqPair>, String> {
        let query = match index {
            1 => GET_BY_SECOND_AND_THIRD,
            2 => GET_BY_FIRST_AND_THIRD,
            3 => GET_BY_FIRST_AND_SECOND,
            _ => return Err("Invalid index".to_string()),
        };
        let consistency = Consistency::One;

        let s = Arc::clone(&session);

        let query = match QueryFactory::build(s, query, consistency).await {
            Ok(query) => query,
            Err(err) => return Err(err.to_string()),
        };

        let input = match index {
            1 => vec![&three_gram.word2, &three_gram.word3],
            2 => vec![&three_gram.word1, &three_gram.word3],
            3 => vec![&three_gram.word1, &three_gram.word2],
            _ => return Err("Invalid index".to_string()),
        }
        .iter()
        .map(|s| s.as_str())
        .collect::<Vec<&str>>();

        let s = Arc::clone(&session);

        let rows = match query.execute_one(s, input).await {
            Ok(rows) => rows,
            Err(err) => match err {
                QueryError::ScyllaError => return Err("Can not execute query".to_string()),
                QueryError::NotFound => return Err("Word not found".to_string()),
            },
        };

        let mut result: Vec<WordFreqPair> = rows
            .into_typed::<(String, i32)>()
            .map(|row| {
                let (word, freq) = row.unwrap();
                WordFreqPair::new(word, freq)
            })
            .collect();

        result.sort_by(|a, b| b.frequency.cmp(&a.frequency));

        if amount >= 0 {
            result.truncate(amount as usize);
        }

        Ok(result)
    }

    /// Finds the word in the given vector of `WordFreqPair`.
    ///
    /// # Arguments
    ///
    /// * `pairs` - The vector of `WordFreqPair`.
    /// * `word` - The word to find.
    ///
    /// # Returns
    ///
    /// An `Option` containing the `WordFreqPair` if the word is found, otherwise `None`.
    pub fn find<'a>(pairs: &'a Vec<WordFreqPair>, word: &str) -> Option<&'a WordFreqPair> {
        pairs.iter().find(|pair| pair.word == word)
    }
}
