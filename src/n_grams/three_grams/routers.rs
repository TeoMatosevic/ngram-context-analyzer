use super::{super::vary_n_gram::VaryingQueryResult, model::ThreeGramQueryParams};
use crate::{error_handler::HttpError, parse, AppData};
use actix_web::{get, web, HttpResponse};
use std::collections::HashMap;
use std::sync::Arc;

/// Handles the GET request to get a three-gram.
///
/// # Arguments
///
/// * `query` - The query that contains the three-gram.
/// * `data` - The application data.
///
/// # Returns
///
/// A `Result` containing the `HttpResponse` with the three-gram if the request is successful, otherwise a `HttpError`.
///
/// # Errors
///
/// If the three-gram is not found, a `HttpError` is returned.
#[get("/three-gram")]
pub async fn get_three_gram(
    query: web::Query<HashMap<String, String>>,
    data: web::Data<AppData>,
) -> Result<HttpResponse, HttpError> {
    let session = Arc::clone(&data.scy_session);

    let query = query.into_inner().clone();

    let input = match parse::<ThreeGramQueryParams>(&query) {
        Ok(input) => input,
        Err(err) => return Ok(HttpResponse::BadRequest().json(err)),
    };

    match input.varying_indexes {
        Some(indexes) => {
            let s = Arc::clone(&session);

            let result =
                VaryingQueryResult::get_varying(s, input.three_gram, indexes, input.amount).await;

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
            let three_gram = VaryingQueryResult::get_one(s, input.three_gram).await;

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
