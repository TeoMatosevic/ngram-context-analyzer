/// This module contains the three grams of the application
///
/// # Modules
///
/// * `model` - Contains the model of the three grams.
/// * `routers` - Contains the routers of the three grams.
mod model;
mod routers;

/// Http request handlers for the three grams.
pub use routers::get_three_gram;
