/// This module contains the logic for generating two-grams from a given text.
///
/// # Modules
///
/// * `model` - Contains the model of the two-grams.
/// * `routers` - Contains the routers of the two-grams.
pub mod model;
mod routers;

/// Http request handlers for the two-grams.
pub use routers::get_two_gram;
