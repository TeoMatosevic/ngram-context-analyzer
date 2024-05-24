use super::model::{FromQueryParams, NgramQueryParams, SUPPORTED_N_GRAMS};
use super::three_grams;
use super::two_grams;
use crate::parse_n;
use crate::{error_handler::HttpError, AppData};
use actix_web::{get, web, HttpResponse};
use std::collections::HashMap;
use std::sync::Arc;

/// Handles the n-gram query.
///
/// # Arguments
///
/// * `query` - The query parameters.
/// * `data` - The application data.
///
/// # Returns
///
/// * `HttpResponse` - The response.
#[get("/n-gram")]
pub async fn get_n_gram(
    query: web::Query<HashMap<String, String>>,
    data: web::Data<AppData>,
) -> Result<HttpResponse, HttpError> {
    let session = Arc::clone(&data.scy_session);

    let query = query.into_inner().clone();

    let n = match parse_n(&query) {
        Ok(n) => n,
        Err(err) => return Ok(HttpResponse::BadRequest().json(err)),
    };

    if !SUPPORTED_N_GRAMS.contains(&n) {
        return Ok(HttpResponse::BadRequest().json(format!("{}-grams are not supported", n)));
    }

    match n {
        2 => {
            let query_params =
                match NgramQueryParams::<two_grams::model::TwoGramInput>::create(query) {
                    Ok(query_params) => query_params,
                    Err(err) => return Ok(HttpResponse::BadRequest().json(err)),
                };

            let result = NgramQueryParams::execute(query_params, session).await;

            result
        }
        3 => {
            let query_params =
                match NgramQueryParams::<three_grams::model::ThreeGramInput>::create(query) {
                    Ok(query_params) => query_params,
                    Err(err) => return Ok(HttpResponse::BadRequest().json(err)),
                };

            let result = NgramQueryParams::execute(query_params, session).await;

            result
        }
        _ => {
            unreachable!("The n-gram is not supported");
        }
    }
}

/// Initializes the routes for the n-grams.
///
/// # Arguments
///
/// * `cfg` - The configuration of the service.
pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(get_n_gram);
}
