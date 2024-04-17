use crate::db::{
    QueryError, QueryFactory, GET_BY_ALL, GET_BY_FIRST_AND_SECOND, GET_BY_FIRST_AND_THIRD,
    GET_BY_SECOND_AND_THIRD,
};
use actix_web::web;
use scylla::{statement::Consistency, IntoTypedRows, Session};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
#[derive(Serialize, Deserialize)]
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
    pub fn from(query: &web::Query<HashMap<String, String>>) -> Result<Self, String> {
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
        session: &Session,
        index: &i32,
        three_gram: &ThreeGramInput,
    ) -> Result<Vec<WordFreqPair>, String> {
        let query = match index {
            1 => GET_BY_SECOND_AND_THIRD,
            2 => GET_BY_FIRST_AND_THIRD,
            3 => GET_BY_FIRST_AND_SECOND,
            _ => return Err("Invalid index".to_string()),
        };
        let consistency = Consistency::One;

        let query = match QueryFactory::build(session, query, consistency).await {
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

        let rows = match query.execute_one(session, input).await {
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

    pub fn find<'a>(pairs: &'a Vec<WordFreqPair>, word: &str) -> Option<&'a WordFreqPair> {
        pairs.iter().find(|pair| pair.word == word)
    }
}

/// Represents a varying three-gram.
///
/// # Fields
///
/// * `index` - The index of the word.
/// * `word` - The word.
/// * `solutions` - The solutions of the word.
///
/// # Methods
///
/// * `new` - Creates a new `VaryingThreeGram`.
#[derive(Serialize, Deserialize)]
pub struct VaryingThreeGram {
    pub index: i32,
    pub word: String,
    pub solutions: Vec<WordFreqPair>,
}

impl VaryingThreeGram {
    /// Creates a new `VaryingThreeGram`.
    ///
    /// # Arguments
    ///
    /// * `index` - The index of the word.
    /// * `word` - The word.
    ///
    /// # Returns
    ///
    /// A `VaryingThreeGram`.
    pub fn new(index: &i32, word: String, solutions: Vec<WordFreqPair>) -> VaryingThreeGram {
        VaryingThreeGram {
            index: *index,
            word,
            solutions,
        }
    }
}

/// Represents the varying query result.
///
/// # Fields
///
/// * `time_taken` - The time taken to execute the query.
/// * `n_gram_length` - The length of the n-gram.
/// * `provided_n_gram` - The provided n-gram.
/// * `provided_n_gram_frequency` - The frequency of the provided n-gram.
/// * `varying_indexes` - The varying indexes.
/// * `vary` - The varying three-grams.
///
/// # Methods
///
/// * `get_one` - Gets the query result with one n-gram.
/// * `get_varying` - Gets the query result with varying n-grams.
#[derive(Serialize, Deserialize)]
pub struct VaryingQueryResult {
    pub time_taken: String,
    pub n_gram_length: i32,
    pub provided_n_gram: String,
    pub provided_n_gram_frequency: i32,
    pub varying_indexes: Vec<i32>,
    pub vary: Vec<VaryingThreeGram>,
}

impl VaryingQueryResult {
    /// Gets the query result with one n-gram.
    ///
    /// # Arguments
    ///
    /// * `session` - The ScyllaDB session.
    /// * `input` - The three-gram input.
    ///
    /// # Returns
    ///
    /// A `Result` containing the `VaryingQueryResult` if the query is successful, otherwise a `String` with the error message.
    pub async fn get_one(session: &Session, input: ThreeGramInput) -> Result<Self, String> {
        let query = GET_BY_ALL;
        let consistency = Consistency::One;

        let start_time = std::time::Instant::now();

        let query = match QueryFactory::build(session, query, consistency).await {
            Ok(query) => query,
            Err(err) => return Err(err.to_string()),
        };

        let rows = match query
            .execute_one(session, vec![&input.word1, &input.word2, &input.word3])
            .await
        {
            Ok(rows) => rows,
            Err(err) => match err {
                QueryError::ScyllaError => return Err("Can not execute query".to_string()),
                QueryError::NotFound => {
                    let end_time = format!("{} ms", start_time.elapsed().as_millis());
                    return Ok(VaryingQueryResult {
                        time_taken: end_time,
                        n_gram_length: 3,
                        provided_n_gram: format!("{} {} {}", input.word1, input.word2, input.word3),
                        provided_n_gram_frequency: 0,
                        varying_indexes: vec![],
                        vary: vec![],
                    });
                }
            },
        };

        let mut result = Vec::new();
        let mut provided_n_gram_frequency = 0;

        for row in rows.into_typed::<(String, String, String, i32)>() {
            let (word1, word2, word3, freq) = match row {
                Ok(row) => row,
                Err(err) => return Err(err.to_string()),
            };
            provided_n_gram_frequency = freq;
            result.push(ThreeGram {
                word1,
                word2,
                word3,
                freq,
            });
        }
        let end_time = format!("{} ms", start_time.elapsed().as_millis());
        return Ok(VaryingQueryResult {
            time_taken: end_time,
            n_gram_length: 3,
            provided_n_gram: format!("{} {} {}", input.word1, input.word2, input.word3),
            provided_n_gram_frequency,
            varying_indexes: vec![],
            vary: vec![],
        });
    }

    /// Gets the query result with varying n-grams.
    ///
    /// # Arguments
    ///
    /// * `session` - The ScyllaDB session.
    /// * `input` - The three-gram input.
    /// * `varying_indexed` - The varying indexes.
    ///
    /// # Returns
    ///
    /// A `Result` containing the `VaryingQueryResult` if the query is successful, otherwise a `String` with the error message.
    pub async fn get_varying(
        session: &Session,
        input: &ThreeGramInput,
        varying_indexed: Vec<i32>,
    ) -> Result<VaryingQueryResult, String> {
        let mut vary: Vec<VaryingThreeGram> = vec![];
        let vary_indexes_copy = varying_indexed.clone();

        let start_time = std::time::Instant::now();

        for index in &varying_indexed {
            let solutions = WordFreqPair::from(session, &index, &input).await;
            let solutions = match solutions {
                Ok(solutions) => solutions,
                Err(err) => return Err(err),
            };
            let word = match index {
                1 => input.word1.clone(),
                2 => input.word2.clone(),
                3 => input.word3.clone(),
                _ => return Err("Invalid index".to_string()),
            };
            let varying = VaryingThreeGram::new(&index, word, solutions);
            vary.push(varying);
        }

        let mut provided_n_gram_frequency = 0;

        if varying_indexed.contains(&1) {
            for i in &vary {
                if i.index == 1 {
                    let pair = WordFreqPair::find(&i.solutions, &input.word1);
                    match pair {
                        Some(pair) => provided_n_gram_frequency = pair.frequency,
                        None => println!("No pair found"),
                    }
                }
            }
        } else if varying_indexed.contains(&2) {
            for i in &vary {
                if i.index == 2 {
                    let pair = WordFreqPair::find(&i.solutions, &input.word2);
                    match pair {
                        Some(pair) => provided_n_gram_frequency = pair.frequency,
                        None => println!("No pair found"),
                    }
                }
            }
        } else if varying_indexed.contains(&3) {
            for i in &vary {
                if i.index == 3 {
                    let pair = WordFreqPair::find(&i.solutions, &input.word3);
                    match pair {
                        Some(pair) => provided_n_gram_frequency = pair.frequency,
                        None => println!("No pair found"),
                    }
                }
            }
        }

        let end_time = format!("{} ms", start_time.elapsed().as_millis());
        Ok(VaryingQueryResult {
            time_taken: end_time,
            n_gram_length: 3,
            provided_n_gram: format!("{} {} {}", input.word1, input.word2, input.word3),
            provided_n_gram_frequency,
            varying_indexes: vary_indexes_copy,
            vary,
        })
    }
}
