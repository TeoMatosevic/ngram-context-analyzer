use scylla::{
    prepared_statement::PreparedStatement, serialize::row::SerializeRow, statement::Consistency,
    transport::iterator::RowIterator, Session, SessionBuilder,
};
use std::sync::Arc;

pub static GET_FREQ_3: &str =
    "SELECT freq FROM n_grams.three_grams_1_2_pk WHERE word_1 = ? AND word_2 = ? AND word_3 = ?";

pub static GET_ALL_3: &str =
    "SELECT * FROM n_grams.three_grams_1_2_pk where word_1 = ? AND word_2 = ? AND word_3 = ?";

pub static GET_ALL_VARYING_3_3: &str =
    "SELECT word_3, freq FROM n_grams.three_grams_1_2_pk WHERE word_1 = ? AND word_2 = ? AND word_3 IN ";

pub static GET_ALL_VARYING_3_1: &str =
    "SELECT word_1, freq FROM n_grams.three_grams_2_3_pk WHERE word_2 = ? AND word_3 = ? AND word_1 IN ";

pub static GET_BY_SECOND_AND_THIRD_3: &str =
    "SELECT word_1, freq FROM n_grams.three_grams_2_3_pk WHERE word_2 = ? AND word_3 = ?";

pub static GET_BY_FIRST_AND_THIRD_3: &str =
    "SELECT word_2, freq FROM n_grams.three_grams_1_3_pk WHERE word_1 = ? AND word_3 = ?";

pub static GET_BY_FIRST_AND_SECOND_3: &str =
    "SELECT word_3, freq FROM n_grams.three_grams_1_2_pk WHERE word_1 = ? AND word_2 = ?";

pub static GET_FREQ_2: &str =
    "SELECT freq FROM n_grams.two_grams_1_pk WHERE word_1 = ? AND word_2 = ?";

pub static GET_ALL_2: &str = "SELECT * FROM n_grams.two_grams_1_pk WHERE word_1 = ? AND word_2 = ?";

pub static GET_ALL_VARYING_2_2: &str =
    "SELECT word_2, freq FROM n_grams.two_grams_1_pk WHERE word_1 = ? AND word_2 IN ";

pub static GET_ALL_VARYING_2_1: &str =
    "SELECT word_1, freq FROM n_grams.two_grams_2_pk WHERE word_2 = ? AND word_1 IN ";

pub static GET_BY_SECOND_2: &str =
    "SELECT word_1, freq FROM n_grams.two_grams_2_pk WHERE word_2 = ?";

pub static GET_BY_FIRST_2: &str =
    "SELECT word_2, freq FROM n_grams.two_grams_1_pk WHERE word_1 = ?";

pub static GET_ALL_1: &str = "SELECT word, freq FROM n_grams.one_grams WHERE word = ?";

pub static GET_ALL_VARYING_1: &str = "SELECT * FROM n_grams.one_grams WHERE word IN ";

/// Represents the error that can occur when querying the database.
///
/// # Fields
///
/// * `ScyllaError` - The error that occurs when querying the database.
/// * `NotFound` - The error that occurs when the query result is not found.
pub enum QueryError {
    ScyllaError,
    NotFound,
}

/// Initializes the ScyllaDB session.
///
/// # Returns
///
/// A `Result` containing the `Session` if the connection is successful, otherwise a `&'static str` with the error message.
///
/// # Errors
///
/// If the connection to ScyllaDB can not be established, a `&'static str` with the error message will be returned.
pub async fn init() -> Result<Arc<Session>, &'static str> {
    let uri = std::env::var("SCYLLA_URI").unwrap_or_else(|_| "127.0.0.1:9042".to_string());

    let session = match SessionBuilder::new().known_node(uri).build().await {
        Ok(session) => session,
        Err(_) => return Err("Failed to connect to ScyllaDB"),
    };

    let session = Arc::new(session);

    Ok(session)
}

/// Represents the query factory.
///
/// # Fields
///
/// * `preapred_query` - The prepared query.
pub struct QueryFactory {
    prepared_query: PreparedStatement,
}

impl QueryFactory {
    /// Builds the query factory.
    ///
    /// # Arguments
    ///
    /// * `session` - The ScyllaDB session.
    /// * `query` - The query to be executed.
    /// * `consistency` - The consistency level.
    ///
    /// # Returns
    ///
    /// A `Result` containing the `QueryFactory` if the preparation is successful, otherwise a `&'static str` with the error message.
    ///
    /// # Errors
    ///
    /// If the query can not be prepared, a `&'static str` with the error message will be returned.
    pub async fn build(
        session: Arc<Session>,
        query: &str,
        consistency: Consistency,
    ) -> Result<Self, &'static str> {
        let mut prepared_query = match session.prepare(query).await {
            Ok(prepared_query) => prepared_query,
            Err(_) => return Err("Failed to prepare query"),
        };

        prepared_query.set_consistency(consistency);

        Ok(QueryFactory { prepared_query })
    }

    /// Executes the query.
    ///
    /// # Arguments
    ///
    /// * `session` - The ScyllaDB session.
    /// * `params` - The query parameters.
    ///
    /// # Returns
    ///
    /// A `Result` containing the `RowIterator` if the query is successful, otherwise a `QueryError`.
    ///
    /// # Errors
    ///
    /// If the query can not be executed, a `QueryError` will be returned.
    /// If the query does not return any results, a `QueryError` will be returned.
    pub async fn execute_one(
        &self,
        session: Arc<Session>,
        params: impl SerializeRow,
    ) -> Result<RowIterator, QueryError> {
        let query = PreparedStatement::clone(&self.prepared_query);
        let rows_stream = match session.execute_iter(query, params).await {
            Ok(rows_stream) => rows_stream,
            Err(_) => return Err(QueryError::ScyllaError),
        };

        Ok(rows_stream)
    }
}

/// Gets the n-gram string.
///
/// # Arguments
///
/// * `query` - The query.
/// * `static_params` - The static parameters.
/// * `varying_param` - The varying parameter.
///
/// # Returns
///
/// The n-gram string.
///
/// # Panics
///
/// If the query is invalid, a panic will occur.
pub fn get_n_gram_string(query: &str, static_params: &Vec<&str>, varying_param: &str) -> String {
    if query.starts_with("SELECT word_3, freq FROM n_grams.three_grams_1_2_pk WHERE word_1 = ? AND word_2 = ? AND word_3 IN") {
        let first = static_params.get(0).unwrap();
        let second = static_params.get(1).unwrap();
        format!("{} {} {}", first, second, varying_param)
    } else if query.starts_with("SELECT word_1, freq FROM n_grams.three_grams_2_3_pk WHERE word_2 = ? AND word_3 = ? AND word_1 IN") {
        let second = static_params.get(0).unwrap();
        let third = static_params.get(1).unwrap();
        format!("{} {} {}", varying_param, second, third)
    } else if query.starts_with("SELECT word_2, freq FROM n_grams.two_grams_1_pk WHERE word_1 = ? AND word_2 IN") {
        let first = static_params.get(0).unwrap();
        format!("{} {}", first, varying_param)
    } else if query.starts_with("SELECT word_1, freq FROM n_grams.two_grams_2_pk WHERE word_2 = ? AND word_1 IN") {
        let second = static_params.get(0).unwrap();
        format!("{} {}", varying_param, second)
    } else if query.starts_with("SELECT * FROM n_grams.one_grams WHERE word IN") {
        format!("{}", varying_param)
    } else {
        panic!("Invalid query: {}", query);
    }
}
