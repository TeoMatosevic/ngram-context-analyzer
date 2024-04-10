use scylla::{
    frame::response::result::Row, prepared_statement::PreparedStatement, statement::Consistency,
    Session, SessionBuilder,
};

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
pub async fn init() -> Result<Session, &'static str> {
    let uri = std::env::var("SCYLLA_URI").unwrap_or_else(|_| "127.0.0.1:9042".to_string());

    let session = match SessionBuilder::new().known_node(uri).build().await {
        Ok(session) => session,
        Err(_) => return Err("Failed to connect to ScyllaDB"),
    };

    Ok(session)
}

/// Represents the query factory.
/// 
/// # Fields
/// 
/// * `preapred_query` - The prepared query.
pub struct QueryFactory {
    preapred_query: PreparedStatement,
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
        session: &Session,
        query: &str,
        consistency: Consistency,
    ) -> Result<Self, &'static str> {
        let mut prepared_query = match session.prepare(query).await {
            Ok(prepared_query) => prepared_query,
            Err(_) => return Err("Failed to prepare query"),
        };

        prepared_query.set_consistency(consistency);

        Ok(QueryFactory {
            preapred_query: prepared_query,
        })
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
    /// A `Result` containing the `Vec` of `Row` if the query is successful, otherwise a `QueryError`.
    /// 
    /// # Errors
    /// 
    /// If the query can not be executed, a `QueryError` will be returned.
    /// If the query does not return any results, a `QueryError` will be returned.
    pub async fn execute_one(
        &self,
        session: &Session,
        params: Vec<&str>,
    ) -> Result<Vec<Row>, QueryError> {
        let rows = match session.execute(&self.preapred_query, params).await {
            Ok(res) => res.rows,
            Err(_) => return Err(QueryError::ScyllaError),
        };

        if let Some(rows) = rows {
            Ok(rows)
        } else {
            Err(QueryError::NotFound)
        }
    }
}
