use crate::{
    db::{QueryError, QueryFactory},
    n_grams::Queryable,
};
use futures::stream::StreamExt;
use scylla::{statement::Consistency, Session};
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
        let query = match input.get_query(Some(*index)) {
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

        let mut row_stream = match query.execute_one(s, input).await {
            Ok(rows) => rows.into_typed::<(String, i32)>(),
            Err(err) => match err {
                QueryError::ScyllaError => return Err("Can not execute query".to_string()),
                QueryError::NotFound => return Err("Word not found".to_string()),
            },
        };

        let mut result: Vec<WordFreqPair> = vec![];

        while let Some(rows) = row_stream.next().await {
            let (word, frequency) = rows.unwrap();
            result.push(WordFreqPair::new(word, frequency));
        }

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

#[cfg(test)]
mod tests {
    use super::super::super::db::init;
    use super::*;
    use crate::n_grams::three_grams::model::ThreeGramInput;
    use std::collections::HashMap;

    #[test]
    fn test_new() {
        let word_freq_pair = WordFreqPair::new("word".to_string(), 1);

        assert_eq!(word_freq_pair.word, "word");
        assert_eq!(word_freq_pair.frequency, 1);
    }

    #[tokio::test]
    async fn test_from() {
        let session = init().await.unwrap();

        let mut query_map = HashMap::new();

        query_map.insert("word1".to_string(), "ja".to_string());
        query_map.insert("word2".to_string(), "sam".to_string());
        query_map.insert("word3".to_string(), "gledao".to_string());
        query_map.insert("vary".to_string(), "1,2".to_string());
        query_map.insert("amount".to_string(), "50".to_string());

        let input = ThreeGramInput::from(&query_map).unwrap();

        let result = WordFreqPair::from(Arc::clone(&session), &1, &input).await;

        assert!(result.is_ok());
        let result = result.unwrap();

        assert!(result.len() >= 50);
    }

    #[test]
    fn test_find() {
        let word_freq_pair1 = WordFreqPair::new("word".to_string(), 1);
        let word_freq_pair2 = WordFreqPair::new("word2".to_string(), 2);
        let word_freq_pair3 = WordFreqPair::new("word3".to_string(), 3);
        let pairs = vec![word_freq_pair1, word_freq_pair2, word_freq_pair3];

        let result = WordFreqPair::find(&pairs, "word");

        assert!(result.is_some());
        assert_eq!(result.unwrap().word, "word");
    }
}
