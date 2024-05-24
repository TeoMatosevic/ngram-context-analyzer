use super::{
    three_grams::model::{validate as validate_indexes_3, ThreeGramInput},
    two_grams::model::{validate as validate_indexes_2, TwoGramInput},
    vary_n_gram::VaryingQueryResult,
    word_freq_pair::DEFAULT_AMOUNT_OF_WORD_FREQ_PAIRS,
    Printable, Queryable,
};
use crate::{error_handler::HttpError, parse_amount, parse_varying_indexes};
use actix_web::HttpResponse;
use scylla::Session;
use std::{collections::HashMap, sync::Arc};

/// supported n-grams
pub static SUPPORTED_N_GRAMS: [i32; 2] = [2, 3];

/// Query parameters for n-gram queries
///
/// # Fields
///
/// * `n_gram` - the n-gram to query
/// * `varying_indexes` - the indexes to vary
/// * `amount` - the amount of word frequency pairs to return
pub struct NgramQueryParams<T: Queryable> {
    pub n_gram: T,
    pub varying_indexes: Option<Vec<i32>>,
    pub amount: i32,
}

impl<T> NgramQueryParams<T>
where
    T: Queryable + Printable + Clone + Send + Sync + 'static,
{
    /// Execute the n-gram query
    ///
    /// # Arguments
    ///
    /// * `input` - the query parameters
    /// * `session` - the scylla session
    ///
    /// # Returns
    ///
    /// * `HttpResponse` - the response
    pub async fn execute(
        input: NgramQueryParams<T>,
        session: Arc<Session>,
    ) -> Result<HttpResponse, HttpError> {
        match input.varying_indexes {
            Some(indexes) => {
                let s = Arc::clone(&session);

                let result =
                    VaryingQueryResult::get_varying(s, input.n_gram, indexes, input.amount).await;

                let result = match result {
                    Ok(result) => Ok(result),
                    Err(e) => Err(e.to_string()),
                };

                match result {
                    Ok(result) => Ok(HttpResponse::Ok().json(result)),
                    Err(e) => {
                        eprintln!("{}", e);
                        Ok(HttpResponse::BadRequest().json(e))
                    }
                }
            }
            None => {
                let s = Arc::clone(&session);
                let three_gram = VaryingQueryResult::get_one(s, input.n_gram).await;

                let three_gram = match three_gram {
                    Ok(three_gram) => three_gram,
                    Err(e) => {
                        eprintln!("{}", e);
                        return Ok(HttpResponse::BadRequest().json(e));
                    }
                };

                Ok(HttpResponse::Ok().json(three_gram))
            }
        }
    }
}

/// Trait for creating query parameters from a hashmap
///
/// # Methods
///
/// * `create` - create the query parameters from a hashmap
pub trait FromQueryParams {
    /// Create the query parameters from a hashmap
    ///
    /// # Arguments
    ///
    /// * `query` - the query parameters
    ///
    /// # Returns
    ///
    /// * `Result<Self, String>` - the query parameters
    fn create(query: HashMap<String, String>) -> Result<Self, String>
    where
        Self: Sized;
}

impl FromQueryParams for NgramQueryParams<ThreeGramInput> {
    fn create(query: HashMap<String, String>) -> Result<NgramQueryParams<ThreeGramInput>, String> {
        let varying_indexes = match query.get("vary") {
            Some(vary) => match parse_varying_indexes(vary, validate_indexes_3) {
                Ok(indexes) => Some(indexes),
                Err(e) => return Err(e),
            },
            None => None,
        };
        let amount = match query.get("amount") {
            Some(amount) => match parse_amount(amount) {
                Ok(amount) => amount,
                Err(e) => return Err(e),
            },
            None => DEFAULT_AMOUNT_OF_WORD_FREQ_PAIRS,
        };
        let three_gram = ThreeGramInput::from(&query);

        let three_gram = match three_gram {
            Ok(three_gram) => three_gram,
            Err(e) => return Err(e),
        };

        Ok(NgramQueryParams {
            n_gram: three_gram,
            varying_indexes,
            amount,
        })
    }
}

impl FromQueryParams for NgramQueryParams<TwoGramInput> {
    fn create(query: HashMap<String, String>) -> Result<NgramQueryParams<TwoGramInput>, String> {
        let varying_indexes = match query.get("vary") {
            Some(vary) => match parse_varying_indexes(vary, validate_indexes_2) {
                Ok(indexes) => Some(indexes),
                Err(e) => return Err(e),
            },
            None => None,
        };
        let amount = match query.get("amount") {
            Some(amount) => match parse_amount(amount) {
                Ok(amount) => amount,
                Err(e) => return Err(e),
            },
            None => DEFAULT_AMOUNT_OF_WORD_FREQ_PAIRS,
        };
        let two_gram = TwoGramInput::from(&query);

        let two_gram = match two_gram {
            Ok(two_gram) => two_gram,
            Err(e) => return Err(e),
        };

        Ok(NgramQueryParams {
            n_gram: two_gram,
            varying_indexes,
            amount,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_creating_three_gram_query_params() {
        let mut query = HashMap::new();
        query.insert("word1".to_string(), "hello".to_string());
        query.insert("word2".to_string(), "world".to_string());
        query.insert("word3".to_string(), "foo".to_string());
        query.insert("vary".to_string(), "1,2".to_string());
        query.insert("amount".to_string(), "10".to_string());

        let result = NgramQueryParams::<ThreeGramInput>::create(query);

        assert!(result.is_ok());
    }

    #[test]
    fn test_creating_three_gram_query_params_fail() {
        let mut query = HashMap::new();
        query.insert("word1".to_string(), "hello".to_string());
        query.insert("word2".to_string(), "world".to_string());
        query.insert("vary".to_string(), "1,2".to_string());
        query.insert("amount".to_string(), "10".to_string());

        let result = NgramQueryParams::<ThreeGramInput>::create(query);

        assert!(result.is_err());
    }

    #[test]
    fn test_creating_two_gram_query_params() {
        let mut query = HashMap::new();
        query.insert("word1".to_string(), "hello".to_string());
        query.insert("word2".to_string(), "world".to_string());
        query.insert("vary".to_string(), "1,2".to_string());
        query.insert("amount".to_string(), "10".to_string());

        let result = NgramQueryParams::<TwoGramInput>::create(query);

        assert!(result.is_ok());
    }

    #[test]
    fn test_creating_two_gram_query_params_fail() {
        let mut query = HashMap::new();
        query.insert("word1".to_string(), "hello".to_string());
        query.insert("vary".to_string(), "1,2".to_string());
        query.insert("amount".to_string(), "10".to_string());

        let result = NgramQueryParams::<TwoGramInput>::create(query);

        assert!(result.is_err());
    }

    #[test]
    fn test_creating_three_gram_query_params_without_amount() {
        let mut query = HashMap::new();
        query.insert("word1".to_string(), "hello".to_string());
        query.insert("word2".to_string(), "world".to_string());
        query.insert("word3".to_string(), "foo".to_string());
        query.insert("vary".to_string(), "1,2".to_string());

        let result = NgramQueryParams::<ThreeGramInput>::create(query);

        assert!(result.is_ok());
        assert!(result.unwrap().amount == DEFAULT_AMOUNT_OF_WORD_FREQ_PAIRS);
    }
}
