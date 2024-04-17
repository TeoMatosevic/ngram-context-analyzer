use crate::{
    error_handler::HttpError,
    parse_varying_indexes,
    three_grams::model::{ThreeGramInput, VaryingQueryResult},
};
use actix_web::{get, web, HttpResponse};
use std::collections::HashMap;

/// Represents the application data.
///
/// # Fields
///
/// * `scy_session` - The ScyllaDB session.
///
/// This struct is used to store the application data.
pub struct AppData {
    pub scy_session: scylla::Session,
}

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
///
#[get("/three-gram")]
async fn get_three_gram(
    query: web::Query<HashMap<String, String>>,
    data: web::Data<AppData>,
) -> Result<HttpResponse, HttpError> {
    let session = &data.scy_session;

    let input = match ThreeGramInput::from(&query) {
        Ok(input) => input,
        Err(_) => return Ok(HttpResponse::BadRequest().json("Invalid input")),
    };

    let vary = query.get("vary");

    match vary {
        Some(vary) => {
            let indexes = match parse_varying_indexes(vary) {
                Ok(indexes) => indexes,
                Err(err) => return Ok(HttpResponse::BadRequest().json(err)),
            };

            let result = VaryingQueryResult::get_varying(&session, &input, indexes).await;

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
            let three_gram = VaryingQueryResult::get_one(session, input).await;

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

/// Initializes the routes for the three-gram module.
///
/// # Arguments
///
/// * `cfg` - The application configuration.
pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(get_three_gram);
}
