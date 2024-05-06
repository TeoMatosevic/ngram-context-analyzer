use super::Queryable;
use crate::db::{QueryError, QueryFactory};
use scylla::{statement::Consistency, IntoTypedRows, Session};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// The default amount of word frequency pairs to return.
pub static DEFAULT_AMOUNT_OF_WORD_FREQ_PAIRS: i32 = 50;

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
/// * `from` - Creates a `WordFreqPair` from the given session, index, and n-gram.
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

    /// Creates a `WordFreqPair` from the given session, index, and n-gram.
    ///
    /// # Arguments
    ///
    /// * `session` - The ScyllaDB session.
    /// * `index` - The index of the word.
    /// * `input` - Generic input that implements `Queryable`.
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
    pub async fn from<T>(
        session: Arc<Session>,
        index: &i32,
        input: &T,
    ) -> Result<Vec<WordFreqPair>, String>
    where
        T: Queryable,
    {
        let query = match input.get_query(*index) {
            Ok(query) => query,
            Err(err) => return Err(err),
        };
        let consistency = Consistency::One;

        let s = Arc::clone(&session);

        let query = match QueryFactory::build(s, query, consistency).await {
            Ok(query) => query,
            Err(err) => return Err(err.to_string()),
        };

        let input = match input.get_input(*index) {
            Ok(input) => input.iter().map(|s| s.as_str()).collect::<Vec<&str>>(),
            Err(err) => return Err(err),
        };

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
