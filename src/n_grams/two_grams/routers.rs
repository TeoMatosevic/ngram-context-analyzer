use super::{super::vary_n_gram::VaryingQueryResult, model::TwoGramQueryParams};
use crate::{error_handler::HttpError, parse, AppData};
use actix_web::{get, web, HttpResponse};
use std::collections::HashMap;
use std::sync::Arc;

/// Handles the GET request to get a two-gram.
///
/// # Arguments
///
/// * `query` - The query that contains the two-gram and additional information.
/// * `data` - The application data.
///
/// # Returns
///
/// A `Result` containing the `HttpResponse` with the two-gram if the request is successful, otherwise a `HttpError`.
#[get("/two-gram")]
pub async fn get_two_gram(
    query: web::Query<HashMap<String, String>>,
    data: web::Data<AppData>,
) -> Result<HttpResponse, HttpError> {
    let session = Arc::clone(&data.scy_session);

    let query = query.into_inner().clone();

    let input = match parse::<TwoGramQueryParams>(&query) {
        Ok(input) => input,
        Err(err) => return Ok(HttpResponse::BadRequest().json(err)),
    };

    match input.varying_indexes {
        Some(indexes) => {
            let s = Arc::clone(&session);

            let result =
                VaryingQueryResult::get_varying(s, input.two_gram, indexes, input.amount).await;

            let result = match result {
                Ok(result) => result,
                Err(e) => {
                    eprintln!("{}", e);
                    return Ok(HttpResponse::BadRequest().json(e));
                }
            };

            return Ok(HttpResponse::Ok().json(result));
        }
        None => {
            let s = Arc::clone(&session);
            let two_gram = VaryingQueryResult::get_one(s, input.two_gram).await;

            let two_gram = match two_gram {
                Ok(two_gram) => two_gram,
                Err(e) => {
                    eprintln!("{}", e);
                    return Ok(HttpResponse::BadRequest().json(e));
                }
            };

            return Ok(HttpResponse::Ok().json(two_gram));
        }
    }
}
