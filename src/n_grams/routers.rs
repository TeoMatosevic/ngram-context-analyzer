use super::three_grams;
use super::two_grams;
use actix_web::web;

/// Initializes the routes for the n-grams.
///
/// # Arguments
///
/// * `cfg` - The configuration of the service.
pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(three_grams::get_three_gram);
    cfg.service(two_grams::get_two_gram);
}
