/// This module contains the implementation of the n-grams solver.
///
/// # Modules
///
/// * `model` - Contains the model of the n-grams solver.
pub mod model;

/// Parses the text into sentences.
///
/// # Arguments
///
/// * `text` - The text.
///
/// # Returns
///
/// A `Vec<String>` containing the sentences.
pub fn parse_text_to_sentences(text: &str) -> Vec<String> {
    let mut text: Vec<String> = text.split(". ").map(|s| s.to_string()).collect();
    let last = text.pop().unwrap();

    if last.chars().last().unwrap() == '.' {
        let last = last.chars().take(last.len() - 1).collect();
        text.push(last);
    } else {
        text.push(last);
    }

    text
}
