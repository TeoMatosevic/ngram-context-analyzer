use crate::{
    db::{
        GET_BY_FIRST_AND_SECOND_3, GET_BY_FIRST_AND_THIRD_3, GET_BY_SECOND_AND_THIRD_3, GET_FREQ_3,
    },
    n_grams::{Printable, Queryable},
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents the three gram that is given as input.
///
/// # Fields
///
/// * `word1` - The first word of the three-gram.
/// * `word2` - The second word of the three-gram.
/// * `word3` - The third word of the three-gram.
///
/// # Implements
///
/// * `Queryable` - Provides methods to query the database.
/// * `Printable` - Provides method for printing.
#[derive(Serialize, Deserialize, Clone)]
pub struct ThreeGramInput {
    pub word1: String,
    pub word2: String,
    pub word3: String,
}

impl ThreeGramInput {
    /// Creates a new `ThreeGramInput` from the given query.
    ///
    /// # Arguments
    ///
    /// * `query` - The query that contains the three-gram.
    ///
    /// # Returns
    ///
    /// A `Result` containing the `ThreeGramInput` if the query is valid, otherwise a `String` with the error message.
    pub fn from(query: &HashMap<String, String>) -> Result<ThreeGramInput, String> {
        let word1 = match query.get("word1") {
            Some(word1) => word1,
            None => return Err("word1 is required".to_string()),
        };

        let word2 = match query.get("word2") {
            Some(word2) => word2,
            None => return Err("word2 is required".to_string()),
        };

        let word3 = match query.get("word3") {
            Some(word3) => word3,
            None => return Err("word3 is required".to_string()),
        };

        Ok(ThreeGramInput {
            word1: word1.to_string(),
            word2: word2.to_string(),
            word3: word3.to_string(),
        })
    }
}

impl Queryable for ThreeGramInput {
    fn to_vec(&self) -> Vec<&str> {
        vec![&self.word1, &self.word2, &self.word3]
    }

    fn get_query(&self, index: Option<i32>) -> Result<&str, String> {
        match index {
            Some(index) => match index {
                1 => Ok(GET_BY_SECOND_AND_THIRD_3),
                2 => Ok(GET_BY_FIRST_AND_THIRD_3),
                3 => Ok(GET_BY_FIRST_AND_SECOND_3),
                _ => Err("Invalid index".to_string()),
            },
            None => Ok(GET_FREQ_3),
        }
    }

    fn get_input(&self, index: i32) -> Result<Vec<&String>, String> {
        match index {
            1 => Ok(vec![&self.word2, &self.word3]),
            2 => Ok(vec![&self.word1, &self.word3]),
            3 => Ok(vec![&self.word1, &self.word2]),
            _ => Err("Invalid index".to_string()),
        }
    }

    fn get_word(&self, index: i32) -> Result<&String, String> {
        match index {
            1 => Ok(&self.word1),
            2 => Ok(&self.word2),
            3 => Ok(&self.word3),
            _ => Err("Invalid index".to_string()),
        }
    }
}

impl Printable for ThreeGramInput {
    fn print(&self) -> String {
        format!("{} {} {}", self.word1, self.word2, self.word3)
    }
}

/// Validates the indexes.
///
/// # Arguments
///
/// * `indexes` - The indexes to validate.
///
/// # Returns
///
/// A `Result` containing `()` if the indexes are valid, otherwise a `String` with the error message.
pub fn validate(indexes: &Vec<i32>) -> Result<(), String> {
    let mut new = vec![];
    for index in indexes {
        if *index < 1 || *index > 3 {
            return Err("Invalid index".to_string());
        }
        if new.contains(index) {
            return Err("Invalid index".to_string());
        }
        new.push(*index);
    }
    if new.len() != indexes.len() {
        return Err("Invalid index".to_string());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_creating_three_gram_input() {
        let mut query = HashMap::new();

        query.insert("word1".to_string(), "hello".to_string());
        query.insert("word2".to_string(), "world".to_string());
        query.insert("word3".to_string(), "foo".to_string());

        let three_gram = ThreeGramInput::from(&query).unwrap();

        assert_eq!(three_gram.word1, "hello");
    }

    #[test]
    fn test_creating_three_gram_input_fail() {
        let mut query = HashMap::new();

        query.insert("word1".to_string(), "hello".to_string());
        query.insert("word2".to_string(), "world".to_string());

        let three_gram = ThreeGramInput::from(&query);

        assert_eq!(three_gram.is_err(), true);
    }

    #[test]
    fn test_three_gram_to_vec() {
        let three_gram = ThreeGramInput {
            word1: "hello".to_string(),
            word2: "world".to_string(),
            word3: "foo".to_string(),
        };

        let vec = three_gram.to_vec();

        assert_eq!(vec, vec!["hello", "world", "foo"]);
    }

    #[test]
    fn test_three_gram_get_query() {
        let three_gram = ThreeGramInput {
            word1: "hello".to_string(),
            word2: "world".to_string(),
            word3: "foo".to_string(),
        };

        let query = three_gram.get_query(Some(1)).unwrap();

        assert_eq!(query, GET_BY_SECOND_AND_THIRD_3);
    }

    #[test]
    fn test_three_gram_get_query_freq() {
        let three_gram = ThreeGramInput {
            word1: "hello".to_string(),
            word2: "world".to_string(),
            word3: "foo".to_string(),
        };

        let query = three_gram.get_query(None).unwrap();

        assert_eq!(query, GET_FREQ_3);
    }

    #[test]
    fn test_three_gram_get_input() {
        let three_gram = ThreeGramInput {
            word1: "hello".to_string(),
            word2: "world".to_string(),
            word3: "foo".to_string(),
        };

        let input = three_gram.get_input(1).unwrap();

        assert_eq!(input, vec![&"world", &"foo"]);
    }

    #[test]
    fn test_three_gram_get_word() {
        let three_gram = ThreeGramInput {
            word1: "hello".to_string(),
            word2: "world".to_string(),
            word3: "foo".to_string(),
        };

        let word = three_gram.get_word(1).unwrap();

        assert_eq!(word, &"hello");
    }

    #[test]
    fn test_three_gram_print() {
        let three_gram = ThreeGramInput {
            word1: "hello".to_string(),
            word2: "world".to_string(),
            word3: "foo".to_string(),
        };

        let print = three_gram.print();

        assert_eq!(print, "hello world foo");
    }

    #[test]
    fn test_validate() {
        let indexes = vec![1, 2, 3];

        let result = validate(&indexes);

        assert_eq!(result.is_ok(), true);
    }

    #[test]
    fn test_validate_fail_index_out_of_bounds() {
        let indexes = vec![1, 2, 4];

        let result = validate(&indexes);

        assert_eq!(result.is_err(), true);
    }

    #[test]
    fn test_validate_fail_index_duplicate() {
        let indexes = vec![1, 2, 2];

        let result = validate(&indexes);

        assert_eq!(result.is_err(), true);
    }
}
