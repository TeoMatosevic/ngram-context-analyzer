use crate::{
    db::{
        get_n_gram_string, QueryError, QueryFactory, GET_ALL_VARYING_1, GET_ALL_VARYING_2_1,
        GET_ALL_VARYING_2_2, GET_ALL_VARYING_3_1, GET_ALL_VARYING_3_3,
    },
    n_grams::solver::parse_text_to_sentences,
};
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

/// Represents a text extractor.
///
/// # Fields
///
/// * `text` - The text.
#[derive(Deserialize, Serialize)]
pub struct TextExtractor {
    pub text: String,
}

/// Represents the queries.
///
/// # Fields
///
/// * `queries` - The queries.
/// * `word` - The word.
pub struct Queries {
    pub queries: Vec<QueryBuilder>,
    pub word: String,
}

impl SolverWithConfusionSet {
    /// Creates a new `SolverWithConfusionSet`.
    ///
    /// # Arguments
    ///
    /// * `text` - The text.
    /// * `confusion_set` - The confusion set.
    ///
    /// # Returns
    ///
    /// A `Result` containing the `SolverWithConfusionSet` if the confusion set is not empty, otherwise a `String` with the error message.
    pub fn new(text: String, confusion_set: &Vec<Vec<String>>) -> Result<Self, String> {
        if confusion_set.is_empty() {
            return Err("Confusion set is empty".to_string());
        }

        Ok(Self {
            confusion_set: confusion_set.clone(),
            text,
        })
    }

    /// Finds the queries.
    ///
    /// # Returns
    ///
    /// A `HashMap` containing the queries.
    pub fn find_queries(&self) -> HashMap<String, Queries> {
        let sentences = parse_text_to_sentences(&self.text);
        let mut queries = HashMap::new();

        for sentence in &sentences {
            let words: Vec<&str> = sentence.split_whitespace().collect();
            for confusion_set in &self.confusion_set {
                for word in confusion_set {
                    if sentence.to_lowercase().contains(word) {
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
    queries: &mut HashMap<String, Queries>,
) {
    for (j, &w) in words.iter().enumerate() {
        if w.to_lowercase() == word.to_lowercase() {
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
                if words[j - 1] != words[j - 1].to_lowercase() {
                    let lowercase_word = words[j - 1].to_lowercase();
                    add_to_query(
                        GET_ALL_VARYING_2_2,
                        &vec![&lowercase_word, words[j]],
                        &confusion_set,
                        &mut q,
                        1,
                    );
                }
                add_to_query(
                    GET_ALL_VARYING_2_2,
                    &words[j - 1..=j],
                    &confusion_set,
                    &mut q,
                    1,
                );
            }
            if j + 1 < words.len() {
                if words[j + 1] != words[j + 1].to_lowercase() {
                    let lowercase_word = words[j + 1].to_lowercase();
                    add_to_query(
                        GET_ALL_VARYING_2_1,
                        &vec![words[j], &lowercase_word],
                        &confusion_set,
                        &mut q,
                        0,
                    );
                }
                add_to_query(
                    GET_ALL_VARYING_2_1,
                    &words[j..=j + 1],
                    &confusion_set,
                    &mut q,
                    0,
                );
            }

            if j >= 2 {
                if words[j - 2] != words[j - 2].to_lowercase()
                    || words[j - 1] != words[j - 1].to_lowercase()
                {
                    let lowercase_word1 = words[j - 2].to_lowercase();
                    let lowercase_word2 = words[j - 1].to_lowercase();
                    add_to_query(
                        GET_ALL_VARYING_3_3,
                        &vec![&lowercase_word1, &lowercase_word2, words[j]],
                        &confusion_set,
                        &mut q,
                        2,
                    );
                }
                add_to_query(
                    GET_ALL_VARYING_3_3,
                    &words[j - 2..=j],
                    &confusion_set,
                    &mut q,
                    2,
                );
            }
            if j + 2 < words.len() {
                if words[j + 1] != words[j + 1].to_lowercase()
                    || words[j + 2] != words[j + 2].to_lowercase()
                {
                    let lowercase_word1 = words[j + 1].to_lowercase();
                    let lowercase_word2 = words[j + 2].to_lowercase();
                    add_to_query(
                        GET_ALL_VARYING_3_1,
                        &vec![words[j], &lowercase_word1, &lowercase_word2],
                        &confusion_set,
                        &mut q,
                        0,
                    );
                }
                add_to_query(
                    GET_ALL_VARYING_3_1,
                    &words[j..=j + 2],
                    &confusion_set,
                    &mut q,
                    0,
                );
            }

            let result = Queries {
                queries: q,
                word: word.to_string(),
            };

            queries.insert(context, result);
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
/// * `length` - The length.
#[derive(Deserialize, Serialize)]
pub struct QueryResult {
    pub input: String,
    pub frequency: i32,
    pub length: i32,
}

/// Represents a sentence result.
///
/// # Fields
///
/// * `sentence` - The sentence.
/// * `word` - The word.
/// * `results` - The results.
#[derive(Deserialize, Serialize)]
pub struct SentenceResult {
    pub sentence: String,
    pub word: String,
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
    queries: HashMap<String, Queries>,
    session: Arc<Session>,
) -> TimedSentenceResults {
    let mut sentence_results: Vec<SentenceResult> = vec![];
    let (tx, rx) = mpsc::channel();
    let mut handlers = vec![];

    let start = std::time::Instant::now();

    for (key, value) in queries {
        sentence_results.push(SentenceResult {
            sentence: key.clone(),
            results: vec![],
            word: value.word,
        });
        for v in value.queries {
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

            handlers.push(handle);
        }
    }

    for handle in handlers {
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
        let input = get_n_gram_string(query, &static_values, word.as_str());
        words_received.push(word.clone());
        tx.send((
            key.clone(),
            QueryResult {
                input: input.clone(),
                frequency: freq,
                length: input.split_whitespace().count() as i32,
            },
        ))
        .unwrap();
    }

    for word in varying_values {
        if !words_received.contains(&word.to_string()) {
            let input = get_n_gram_string(query, &static_values, word);
            tx.send((
                key.clone(),
                QueryResult {
                    input: input.clone(),
                    frequency: 0,
                    length: input.split_whitespace().count() as i32,
                },
            ))
            .unwrap();
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_context() {
        let words = vec!["Krleža", "sve", "oduševio", "svojim", "dijelom"];

        let context = extract_context(4, &words);
        assert_eq!(context, "oduševio svojim dijelom");
    }
}
