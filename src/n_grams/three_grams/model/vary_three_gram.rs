use super::{word_freq_pair::WordFreqPair, ThreeGram, ThreeGramInput};
use crate::db::{QueryError, QueryFactory, GET_BY_ALL};
use scylla::{statement::Consistency, IntoTypedRows, Session};
use serde::{Deserialize, Serialize};
use std::{
    sync::{mpsc, Arc},
    thread,
};
use tokio;

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
/// * `find_freq` - Finds the frequency of the word in the given vector of `VaryingThreeGram`.
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
    /// * `solutions` - The solutions of the word.
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

    /// Finds the frequency of the word in the given vector of `VaryingThreeGram`.
    ///
    /// # Arguments
    ///
    /// * `vary` - The vector of `VaryingThreeGram`.
    /// * `word` - The word.
    ///
    /// # Returns
    ///
    /// A `Result` containing the frequency of the word if the word is found, otherwise a `String` with the error message.
    fn find_freq(vary: &VaryingThreeGram, word: &String) -> Result<i32, String> {
        let pair = WordFreqPair::find(&vary.solutions, word);
        match pair {
            Some(pair) => return Ok(pair.frequency),
            None => return Err("No pair found".to_string()),
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
    ///
    /// # Errors
    ///
    /// If the query can not be executed, a `String` with the error message will be returned.
    pub async fn get_one(session: Arc<Session>, input: ThreeGramInput) -> Result<Self, String> {
        let query = GET_BY_ALL;
        let consistency = Consistency::One;

        let start_time = std::time::Instant::now();

        let s = Arc::clone(&session);

        let query = match QueryFactory::build(s, query, consistency).await {
            Ok(query) => query,
            Err(err) => return Err(err.to_string()),
        };

        let s = Arc::clone(&session);

        let rows = match query
            .execute_one(s, vec![&input.word1, &input.word2, &input.word3])
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
    /// * `amount` - The amount of word freq pairs to return.
    ///
    /// # Returns
    ///
    /// A `Result` containing the `VaryingQueryResult` if the query is successful, otherwise a `String` with the error message.
    pub async fn get_varying(
        session: Arc<Session>,
        input: ThreeGramInput,
        varying_indexed: Vec<i32>,
        amount: i32,
    ) -> Result<VaryingQueryResult, String> {
        let mut vary: Vec<VaryingThreeGram> = vec![];
        let vary_indexes_copy = varying_indexed.clone();

        let start_time = std::time::Instant::now();
        let (tx, rx) = mpsc::channel();

        let mut handles = vec![];

        for index in &varying_indexed {
            let s = Arc::clone(&session);
            let index = *index;
            let i = input.clone();
            let tx_clone = tx.clone();

            let handle = thread::spawn(move || {
                let rt = tokio::runtime::Builder::new_multi_thread()
                    .enable_all()
                    .build()
                    .unwrap();
                rt.block_on(process(s, &i, index, tx_clone)).unwrap();
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        drop(tx);

        let mut i = 0;
        let mut provided_n_gram_frequency = 0;

        for received in rx {
            match received {
                Ok(mut varying) => {
                    if i == 0 {
                        let word = varying.word.clone();
                        if let Ok(freq) = VaryingThreeGram::find_freq(&varying, &word) {
                            provided_n_gram_frequency = freq;
                        }
                        i += 1;
                    }
                    if amount >= 0 {
                        varying.solutions.truncate(amount as usize);
                    }
                    vary.push(varying);
                }
                Err(err) => return Err(err),
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

/// Processes the query.
///
/// # Arguments
///
/// * `session` - The ScyllaDB session.
/// * `input` - The three-gram input.
/// * `index` - The index of the word.
/// * `tx` - The sender.
///
/// # Returns
///
/// A `Result` containing the `VaryingThreeGram` if the query is successful, otherwise a `String` with the error message.
///
/// # Errors
///
/// If the query can not be executed, a `String` with the error message will be returned.
async fn process(
    session: Arc<Session>,
    input: &ThreeGramInput,
    index: i32,
    tx: mpsc::Sender<Result<VaryingThreeGram, String>>,
) -> Result<(), std::io::Error> {
    let s = Arc::clone(&session);
    let solutions = WordFreqPair::from(s, &index, input).await;
    let solutions = match solutions {
        Ok(solutions) => solutions,
        Err(err) => {
            tx.send(Err(err)).unwrap();
            return Ok(());
        }
    };

    let word = match index {
        1 => &input.word1,
        2 => &input.word2,
        3 => &input.word3,
        _ => {
            tx.send(Err("Invalid index".to_string())).unwrap();
            return Ok(());
        }
    };

    let varying = VaryingThreeGram::new(&index, word.to_string(), solutions);
    tx.send(Ok(varying)).unwrap();

    Ok(())
}
