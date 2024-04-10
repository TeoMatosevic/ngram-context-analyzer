use crate::db::{QueryError, QueryFactory};
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
    pub fn from(query: web::Query<HashMap<String, String>>) -> Result<Self, String> {
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

/// Represents the result of a query.
///
/// # Fields
///
/// * `result` - The result of the query.
///
/// If the query is successful, the result will contain a `Vec` of `ThreeGram` objects, otherwise it will be `None`.
#[derive(Serialize, Deserialize)]
pub struct QueryResult {
    pub result: Option<Vec<ThreeGram>>,
}

impl QueryResult {
    /// Gets the three-gram from the database.
    ///
    /// # Arguments
    ///
    /// * `session` - The ScyllaDB session.
    /// * `input` - The input that contains the three-gram.
    ///
    /// # Returns
    ///
    /// A `Result` containing the `QueryResult` if the query is successful, otherwise a `String` with the error message.
    ///
    /// # Errors
    ///
    /// If the query can not be executed, a `String` with the error message will be returned.
    /// If the query does not return any results, a `QueryResult` with `None` will be returned.
    pub async fn get_one(session: &Session, input: ThreeGramInput) -> Result<Self, String> {
        let query = "SELECT * FROM n_grams.three_grams_1_2_pk WHERE word_1 = ? AND word_2 = ? AND word_3 = ?";
        let consistency = Consistency::One;

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
                QueryError::NotFound => return Ok(QueryResult { result: None }),
            },
        };

        let mut result = Vec::new();

        for row in rows.into_typed::<(String, String, String, i32)>() {
            let (word1, word2, word3, freq) = match row {
                Ok(row) => row,
                Err(err) => return Err(err.to_string()),
            };
            result.push(ThreeGram {
                word1,
                word2,
                word3,
                freq,
            });
        }
        return Ok(QueryResult {
            result: Some(result),
        });
    }
}
