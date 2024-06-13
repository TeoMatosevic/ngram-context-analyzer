use crate::{
    db::{QueryError, QueryFactory},
    n_grams::{word_freq_pair::WordFreqPair, Printable, Queryable},
};
use futures::stream::StreamExt;
use scylla::{statement::Consistency, Session};
use serde::{Deserialize, Serialize};
use std::{
    sync::{mpsc, Arc},
    thread,
};
use tokio;

/// Represents a varying n-gram.
///
/// # Fields
///
/// * `index` - The index of the word.
/// * `word` - The word.
/// * `solutions` - The solutions of the word.
///
/// # Methods
///
/// * `new` - Creates a new `VaryingNGram`.
/// * `find_freq` - Finds the frequency of the word in the given vector of `VaryingNGram`.
#[derive(Serialize, Deserialize)]
pub struct VaryingNGram {
    pub index: i32,
    pub word: String,
    pub solutions: Vec<WordFreqPair>,
}

impl VaryingNGram {
    /// Creates a new `VaryingNGram`.
    ///
    /// # Arguments
    ///
    /// * `index` - The index of the word.
    /// * `word` - The word.
    /// * `solutions` - The solutions of the word.
    ///
    /// # Returns
    ///
    /// A `VaryingNGram`.
    pub fn new(index: &i32, word: String, solutions: Vec<WordFreqPair>) -> VaryingNGram {
        VaryingNGram {
            index: *index,
            word,
            solutions,
        }
    }

    /// Finds the frequency of the word in the given vector of `VaryingNGram`.
    ///
    /// # Arguments
    ///
    /// * `vary` - The vector of `VaryingNGram`.
    /// * `word` - The word.
    ///
    /// # Returns
    ///
    /// A `Result` containing the frequency of the word if the word is found, otherwise a `String` with the error message.
    fn find_freq(vary: &VaryingNGram, word: &String) -> Result<i32, String> {
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
/// * `vary` - The varying n-grams.
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
    pub vary: Vec<VaryingNGram>,
}

impl VaryingQueryResult {
    /// Gets the query result with one n-gram.
    ///
    /// # Arguments
    ///
    /// * `session` - The ScyllaDB session.
    /// * `input` - Generic input that implements `Queryable`.
    ///
    /// # Returns
    ///
    /// A `Result` containing the `VaryingQueryResult` if the query is successful, otherwise a `String` with the error message.
    ///
    /// # Errors
    ///
    /// If the query can not be executed, a `String` with the error message will be returned.
    pub async fn get_one<T>(session: Arc<Session>, input: T) -> Result<Self, String>
    where
        T: Queryable + Printable + Clone + Send + Sync + 'static,
    {
        let query = match input.get_query(None) {
            Ok(query) => query,
            Err(err) => return Err(err.to_string()),
        };
        let consistency = Consistency::One;

        let start_time = std::time::Instant::now();

        let s = Arc::clone(&session);

        let query = match QueryFactory::build(s, query, consistency).await {
            Ok(query) => query,
            Err(err) => return Err(err.to_string()),
        };

        let s = Arc::clone(&session);

        let mut row_stream = match query.execute_one(s, input.to_vec()).await {
            Ok(rows) => rows.into_typed::<(i32,)>(),
            Err(err) => match err {
                QueryError::ScyllaError => return Err("Can not execute query".to_string()),
                QueryError::NotFound => {
                    let end_time = format!("{} ms", start_time.elapsed().as_millis());
                    return Ok(VaryingQueryResult {
                        time_taken: end_time,
                        n_gram_length: input.print().split_whitespace().count() as i32,
                        provided_n_gram: input.print(),
                        provided_n_gram_frequency: 0,
                        varying_indexes: vec![],
                        vary: vec![],
                    });
                }
            },
        };

        let mut provided_n_gram_frequency = 0;

        if let Some(row) = row_stream.next().await {
            let (freq,) = row.unwrap();
            provided_n_gram_frequency = freq;
        }

        let end_time = format!("{} ms", start_time.elapsed().as_millis());
        return Ok(VaryingQueryResult {
            time_taken: end_time,
            n_gram_length: input.print().split_whitespace().count() as i32,
            provided_n_gram: input.print(),
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
    /// * `input` - Generic input that implements `Queryable`.
    /// * `varying_indexed` - The varying indexes.
    /// * `amount` - The amount of word freq pairs to return.
    ///
    /// # Returns
    ///
    /// A `Result` containing the `VaryingQueryResult` if the query is successful, otherwise a `String` with the error message.
    pub async fn get_varying<T>(
        session: Arc<Session>,
        input: T,
        varying_indexed: Vec<i32>,
        amount: i32,
    ) -> Result<VaryingQueryResult, String>
    where
        T: Queryable + Printable + Clone + Send + Sync + 'static,
    {
        let mut vary: Vec<VaryingNGram> = vec![];
        let vary_indexes_copy = varying_indexed.clone();

        let start_time = std::time::Instant::now();
        let (tx, rx) = mpsc::channel();

        let mut handlers = vec![];

        for index in &varying_indexed {
            let s = Arc::clone(&session);
            let index = *index;
            let i = input.clone();
            let tx_clone = tx.clone();

            // not the best approach
            // should use tokio::spawn
            let handle = thread::spawn(move || {
                let rt = tokio::runtime::Builder::new_multi_thread()
                    .enable_all()
                    .build()
                    .unwrap();
                rt.block_on(process(s, &i, index, tx_clone)).unwrap();
            });
            handlers.push(handle);
        }

        for handle in handlers {
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
                        if let Ok(freq) = VaryingNGram::find_freq(&varying, &word) {
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
            n_gram_length: input.print().split_whitespace().count() as i32,
            provided_n_gram: input.print(),
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
/// * `input` - Generic input that implements `Queryable`.
/// * `index` - The index of the word.
/// * `tx` - The sender.
///
/// # Returns
///
/// A `Result` containing unit if the query is successful, otherwise a `String` with the error message.
///
/// # Errors
///
/// If the query can not be executed, a `String` with the error message will be returned.
async fn process<T>(
    session: Arc<Session>,
    input: &T,
    index: i32,
    tx: mpsc::Sender<Result<VaryingNGram, String>>,
) -> Result<(), std::io::Error>
where
    T: Queryable + Printable + Clone + Send + Sync + 'static,
{
    let s = Arc::clone(&session);
    let solutions = WordFreqPair::from(s, &index, input).await;
    let solutions = match solutions {
        Ok(solutions) => solutions,
        Err(err) => {
            tx.send(Err(err)).unwrap();
            return Ok(());
        }
    };

    let word = match input.get_word(index) {
        Ok(word) => word,
        Err(err) => {
            tx.send(Err(err)).unwrap();
            return Ok(());
        }
    };

    let varying = VaryingNGram::new(&index, word.to_string(), solutions);
    tx.send(Ok(varying)).unwrap();

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_freq() {
        let vary = VaryingNGram {
            index: 1,
            word: "hello".to_string(),
            solutions: vec![
                WordFreqPair::new("hello".to_string(), 1),
                WordFreqPair::new("world".to_string(), 2),
            ],
        };
        let word = "hello".to_string();
        assert_eq!(VaryingNGram::find_freq(&vary, &word), Ok(1));
    }

    #[test]
    fn test_find_freq_fail() {
        let vary = VaryingNGram {
            index: 1,
            word: "hello".to_string(),
            solutions: vec![
                WordFreqPair::new("hello".to_string(), 1),
                WordFreqPair::new("world".to_string(), 2),
            ],
        };
        let word = "test".to_string();
        assert_eq!(
            VaryingNGram::find_freq(&vary, &word),
            Err("No pair found".to_string())
        );
    }
}
