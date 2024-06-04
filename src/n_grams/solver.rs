/// This module contains the implementation of the n-grams solver.
///
/// # Modules
///
/// * `model` - Contains the model of the n-grams solver.
pub mod model;

/// This module contains the implementation of the n-grams predictor.
///
/// # Modules
///
/// * `predictor` - Contains the predictor of the n-grams solver.
pub mod predictor;

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
    let mut result: Vec<String> = Vec::new();

    for sentence in text.iter_mut() {
        let res: Vec<String> = sentence.split(", ").map(|s| s.to_string()).collect();
        for s in res {
            result.push(s);
        }
    }

    for sentence in result.iter_mut() {
        if sentence.ends_with('.') {
            sentence.pop();
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_text_to_sentences() {
        let text = "Danas sam poslao dva zahtijeva prodekanu za nastavu. On od mene zahtjeva da dolazim na nastavu. Ona zahtijeva. Krleža sve oduševio svojim dijelom. Svidjela mu se plaća koju je dobio. Uz velike napore, uspio je dobiti posao.";
        let result = parse_text_to_sentences(text);
        let expected = vec![
            "Danas sam poslao dva zahtijeva prodekanu za nastavu",
            "On od mene zahtjeva da dolazim na nastavu",
            "Ona zahtijeva",
            "Krleža sve oduševio svojim dijelom",
            "Svidjela mu se plaća koju je dobio",
            "Uz velike napore",
            "uspio je dobiti posao",
        ];
        assert_eq!(result, expected);
    }
}
