/// The `n_grams` module.
///
/// This module contains the n-grams of the application.
pub mod model;

/// The `router` module.
///
/// This module contains the routers of the application.
pub mod routers;
/// The `three_grams` module.
///
/// This module contains the implementation of three-grams.
mod three_grams;
/// The `two_grams` module.
///
/// This module contains the implementation of two-grams.
mod two_grams;
/// The `vary_n_gram` module.
///
/// This module contains the implementation of varying n-grams.
mod vary_n_gram;
/// The `word_freq_pair` module.
///
/// This module contains the implementation of word frequency pairs.
mod word_freq_pair;

/// Behavior needed for querying the database.
pub trait Queryable {
    /// Converts the `Queryable` to a `Vec<&str>`.
    ///
    /// # Returns
    ///
    /// A `Vec<&str>`.
    fn to_vec(&self) -> Vec<&str>;

    /// Gets the input for the given index.
    ///
    /// # Arguments
    ///
    /// * `index` - The index (i32 when the index is needed).
    ///
    /// # Returns
    ///
    /// A `Result` containing the `Vec<&String>` if the index is valid, otherwise a `String` with the error message.
    ///
    /// # Errors
    ///
    /// If the index is invalid, a `String` with the error message will be returned.
    fn get_query(&self, index: Option<i32>) -> Result<&str, String>;

    /// Gets the input for the given index.
    ///
    /// # Arguments
    ///
    /// * `index` - The index.
    ///
    /// # Returns
    ///
    /// A `Result` containing the `Vec<&String>` if the index is valid, otherwise a `String` with the error message.
    ///
    /// # Errors
    ///
    /// If the index is invalid, a `String` with the error message will be returned.
    fn get_input(&self, index: i32) -> Result<Vec<&String>, String>;

    /// Gets the word for the given index.
    ///
    /// # Arguments
    ///
    /// * `index` - The index.
    ///
    /// # Returns
    ///
    /// A `Result` containing the `&String` if the index is valid, otherwise a `String` with the error message.
    ///
    /// # Errors
    ///
    /// If the index is invalid, a `String` with the error message will be returned.
    fn get_word(&self, index: i32) -> Result<&String, String>;
}

/// Behavior needed for printing.
pub trait Printable {
    /// Prints the `Printable`.
    ///
    /// # Returns
    ///
    /// A `String`.
    fn print(&self) -> String;
}
