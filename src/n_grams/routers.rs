use crate::{
    error_handler::HttpError,
    n_grams::{
        model::{FromQueryParams, NgramQueryParams, SUPPORTED_N_GRAMS},
        solver::model::{execute_queries, SolverWithConfusionSet},
        three_grams, two_grams,
    },
    parse_n, AppData,
};
use actix_web::{get, post, web, Error, HttpResponse};
use futures::StreamExt;
use std::{collections::HashMap, sync::Arc};

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
async fn get_n_gram(
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

/// Handles the text check.
///
/// # Arguments
///
/// * `payload` - The payload.
/// * `data` - The application data.
///
/// # Returns
///
/// * `HttpResponse` - The response.
///
/// # Errors
///
/// If the payload can not be read, a `HttpResponse` with the error message will be returned.
/// If the queries can not be executed, a `HttpResponse` with the error message will be returned.
#[post("/check")]
async fn check_text(
    mut payload: web::Payload,
    data: web::Data<AppData>,
) -> Result<HttpResponse, Error> {
    let mut body = web::BytesMut::new();

    while let Some(item) = payload.next().await {
        body.extend_from_slice(&item?);
    }

    let obj = match SolverWithConfusionSet::new(body) {
        Ok(obj) => obj,
        Err(err) => return Ok(HttpResponse::BadRequest().json(err)),
    };

    let queries = obj.find_queries();

    let session = Arc::clone(&data.scy_session);

    let result = execute_queries(queries, session).await;

    Ok(HttpResponse::Ok().json(result))
}

/// Initializes the routes for the n-grams.
///
/// # Arguments
///
/// * `cfg` - The configuration of the service.
pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(get_n_gram);
    cfg.service(check_text);
}
