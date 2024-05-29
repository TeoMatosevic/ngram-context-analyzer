use crate::{
    db::{
        get_n_gram_string, QueryError, QueryFactory, GET_ALL_VARYING_1, GET_ALL_VARYING_2_1,
        GET_ALL_VARYING_2_2, GET_ALL_VARYING_3_1, GET_ALL_VARYING_3_3,
    },
    n_grams::solver::parse_text_to_sentences,
};
use actix_web::web::BytesMut;
use futures::stream::StreamExt;
use scylla::{statement::Consistency, Session};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::{mpsc, Arc},
};

/// Represents a query builder.
///
/// # Fields
///
/// * `query` - The query.
/// * `static_params` - The static parameters.
/// * `varying_params` - The varying parameters.
#[derive(Deserialize, Serialize)]
pub struct QueryBuilder {
    pub query: String,
    pub static_params: Vec<String>,
    pub varying_params: Vec<String>,
}

/// Represents a solver with a confusion set.
///
/// # Fields
///
/// * `confusion_set` - The confusion set.
/// * `text` - The text.
///
/// # Methods
///
/// * `new` - Creates a new `SolverWithConfusionSet`.
/// * `find_queries` - Finds the queries.
#[derive(Deserialize, Serialize)]
pub struct SolverWithConfusionSet {
    pub confusion_set: Vec<Vec<String>>,
    pub text: String,
}

impl SolverWithConfusionSet {
    /// Creates a new `SolverWithConfusionSet`.
    ///
    /// # Arguments
    ///
    /// * `bytes` - The bytes.
    ///
    /// # Returns
    ///
    /// A `Result` containing the `SolverWithConfusionSet` if the bytes can be deserialized, otherwise a `String` with the error message.
    ///
    /// # Errors
    ///
    /// If the bytes can not be deserialized, a `String` with the error message will be returned.
    pub fn new(bytes: BytesMut) -> Result<Self, String> {
        let obj = match serde_json::from_slice::<SolverWithConfusionSet>(&bytes) {
            Ok(obj) => obj,
            Err(err) => {
                return Err(err.to_string());
            }
        };

        Ok(obj)
    }

    /// Finds the queries.
    ///
    /// # Returns
    ///
    /// A `HashMap` containing the queries.
    pub fn find_queries(&self) -> HashMap<String, Vec<QueryBuilder>> {
        let sentences = parse_text_to_sentences(&self.text);
        let mut queries = HashMap::new();

        for sentence in &sentences {
            let words: Vec<&str> = sentence.split_whitespace().collect();
            for confusion_set in &self.confusion_set {
                for word in confusion_set {
                    if sentence.contains(word) {
                        process_word_in_sentence(word, &words, confusion_set, &mut queries);
                    }
                }
            }
        }

        queries
    }
}

/// Processes the word in the sentence.
///
/// # Arguments
///
/// * `word` - The word.
/// * `words` - The words.
/// * `confusion_set` - The confusion set.
/// * `queries` - The queries.
fn process_word_in_sentence(
    word: &str,
    words: &[&str],
    confusion_set: &[String],
    queries: &mut HashMap<String, Vec<QueryBuilder>>,
) {
    for (j, &w) in words.iter().enumerate() {
        if w == word {
            let context = extract_context(j, words);

            if queries.get(&context).is_some() {
                continue;
            }

            let mut q = Vec::new();

            let query_str = GET_ALL_VARYING_1.to_string()
                + "("
                + "?, ".repeat(confusion_set.len() - 1).as_str()
                + "?)";

            q.push(QueryBuilder {
                query: query_str,
                static_params: vec![],
                varying_params: confusion_set.to_vec(),
            });

            if j >= 1 {
                add_to_query(
                    GET_ALL_VARYING_2_2,
                    &words[j - 1..=j],
                    &confusion_set,
                    &mut q,
                    1,
                );
            }
            if j + 1 < words.len() {
                add_to_query(
                    GET_ALL_VARYING_2_1,
                    &words[j..=j + 1],
                    &confusion_set,
                    &mut q,
                    0,
                );
            }

            if j >= 2 {
                add_to_query(
                    GET_ALL_VARYING_3_3,
                    &words[j - 2..=j],
                    &confusion_set,
                    &mut q,
                    2,
                );
            }
            if j + 2 < words.len() {
                add_to_query(
                    GET_ALL_VARYING_3_1,
                    &words[j..=j + 2],
                    &confusion_set,
                    &mut q,
                    0,
                );
            }

            queries.insert(context, q);
        }
    }
}

/// Extracts the context.
///
/// # Arguments
///
/// * `index` - The index.
/// * `words` - The words.
///
/// # Returns
///
/// The context.
fn extract_context(index: usize, words: &[&str]) -> String {
    let mut context = String::new();
    for i in (index.saturating_sub(2))..=(index + 2).min(words.len() - 1) {
        context.push_str(words[i]);
        context.push(' ');
    }
    context = context.trim().to_string();
    context
}

/// Adds to the query.
///
/// # Arguments
///
/// * `query_str` - The query string.
/// * `window` - The window.
/// * `confusion_set` - The confusion set.
/// * `queries` - The queries.
/// * `index` - The index.
fn add_to_query(
    query_str: &str,
    window: &[&str],
    confusion_set: &[String],
    queries: &mut Vec<QueryBuilder>,
    index: usize,
) {
    let mut static_params: Vec<String> = window.to_vec().iter().map(|s| s.to_string()).collect();
    static_params.remove(index);

    let mut varying_params = Vec::new();

    for word in confusion_set {
        varying_params.push(word.clone());
    }

    let q = query_str.to_string() + "(" + "?, ".repeat(confusion_set.len() - 1).as_str() + "?)";

    queries.push(QueryBuilder {
        query: q,
        static_params,
        varying_params,
    });
}

/// Represents a query result.
///
/// # Fields
///
/// * `input` - The input.
/// * `frequency` - The frequency.
#[derive(Deserialize, Serialize)]
pub struct QueryResult {
    pub input: String,
    pub frequency: i32,
}

/// Represents a sentence result.
///
/// # Fields
///
/// * `sentence` - The sentence.
/// * `results` - The results.
#[derive(Deserialize, Serialize)]
pub struct SentenceResult {
    pub sentence: String,
    pub results: Vec<QueryResult>,
}

/// Represents the timed sentence results.
///
/// # Fields
///
/// * `time_taken` - The time taken.
/// * `results` - The results.
#[derive(Deserialize, Serialize)]
pub struct TimedSentenceResults {
    pub time_taken: String,
    pub results: Vec<SentenceResult>,
}

/// Executes the queries.
///
/// # Arguments
///
/// * `queries` - The queries.
/// * `session` - The session.
///
/// # Returns
///
/// The timed sentence results..
pub async fn execute_queries(
    queries: HashMap<String, Vec<QueryBuilder>>,
    session: Arc<Session>,
) -> TimedSentenceResults {
    let mut sentence_results: Vec<SentenceResult> = vec![];
    let (tx, rx) = mpsc::channel();
    let mut handles = vec![];

    let start = std::time::Instant::now();

    for (key, value) in queries {
        sentence_results.push(SentenceResult {
            sentence: key.clone(),
            results: vec![],
        });
        for v in value {
            let key = key.clone();
            let query = v.query.clone();
            let values = v.varying_params.clone();
            let static_params = v.static_params.clone();

            let s = Arc::clone(&session);

            let tx_clone = tx.clone();

            let handle = tokio::spawn(async move {
                process(
                    key,
                    s,
                    query.as_str(),
                    static_params.iter().map(|s| s.as_str()).collect(),
                    values.iter().map(|s| s.as_str()).collect(),
                    tx_clone,
                )
                .await
                .unwrap();
            });

            handles.push(handle);
        }
    }

    for handle in handles {
        handle.await.unwrap();
    }

    drop(tx);

    for result in rx {
        for sentence_result in &mut sentence_results {
            if sentence_result.sentence == result.0 {
                sentence_result.results.push(result.1);
                break;
            }
        }
    }

    let elapsed = start.elapsed().as_millis();

    TimedSentenceResults {
        time_taken: format!("{} ms", elapsed),
        results: sentence_results,
    }
}

/// Processes the query.
///
/// # Arguments
///
/// * `key` - The key.
/// * `session` - The session.
/// * `query` - The query.
/// * `static_values` - The static values.
/// * `varying_values` - The varying values.
/// * `tx` - The sender.
///
/// # Returns
///
/// A `Result` containing `()` if the query is successful, otherwise a `std::io::Error`.
async fn process(
    key: String,
    session: Arc<Session>,
    query: &str,
    static_values: Vec<&str>,
    varying_values: Vec<&str>,
    tx: mpsc::Sender<(String, QueryResult)>,
) -> Result<(), std::io::Error> {
    let s = Arc::clone(&session);

    let factory = match QueryFactory::build(s, query, Consistency::One).await {
        Ok(factory) => factory,
        Err(err) => {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                err.to_string(),
            ))
        }
    };

    let s = Arc::clone(&session);

    let all_values = static_values
        .iter()
        .chain(varying_values.iter())
        .collect::<Vec<&&str>>();

    let mut row_stream = match factory.execute_one(s, all_values).await {
        Ok(rows) => rows.into_typed::<(String, i32)>(),
        Err(err) => match err {
            QueryError::ScyllaError => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "Can not execute query",
                ))
            }
            QueryError::NotFound => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "Word not found",
                ))
            }
        },
    };

    let mut words_received = vec![];

    while let Some(rows) = row_stream.next().await {
        let (word, freq) = rows.unwrap();
        words_received.push(word.clone());
        tx.send((
            key.clone(),
            QueryResult {
                input: get_n_gram_string(query, &static_values, word.as_str()),
                frequency: freq,
            },
        ))
        .unwrap();
    }

    for word in varying_values {
        if !words_received.contains(&word.to_string()) {
            tx.send((
                key.clone(),
                QueryResult {
                    input: get_n_gram_string(query, &static_values, word),
                    frequency: 0,
                },
            ))
            .unwrap();
        }
    }

    Ok(())
}
