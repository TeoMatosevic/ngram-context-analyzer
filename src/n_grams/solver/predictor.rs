use crate::n_grams::solver::model::TimedSentenceResults;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents the prediction results.
///
/// # Fields
///
/// * `time_elapsed` - The time elapsed.
/// * `results` - The results.
#[derive(Deserialize, Serialize)]
pub struct PredictionResults {
    pub time_elapsed: String,
    pub results: Vec<PredictionResult>,
}

/// Represents the prediction result.
///
/// # Fields
///
/// * `sentence` - The sentence.
/// * `word_examined` - The word examined.
/// * `results` - The results.
#[derive(Deserialize, Serialize)]
pub struct PredictionResult {
    pub context: String,
    pub word_examined: String,
    pub results: HashMap<String, f64>,
}

/// Represents the Laplace smoothing result.
///
/// # Fields
///
/// * `results` - The results.
/// * `n_gram_counts` - The n-gram counts.
pub struct LaplaceSmoothingResult {
    pub results: HashMap<String, i32>,
    pub n_gram_counts: HashMap<i32, i64>,
}

impl LaplaceSmoothingResult {
    /// Gets the Laplace smoothing result.
    ///
    /// # Arguments
    ///
    /// * `results` - The results.
    /// * `n_gram_counts` - The n-gram counts.
    /// * `distinct_n_gram_counts` - The distinct n-gram counts.
    ///
    /// # Returns
    ///
    /// The Laplace smoothing result.
    fn get(
        results: HashMap<String, i32>,
        n_gram_counts: HashMap<i32, i64>,
        distinct_n_gram_counts: HashMap<i32, i64>,
    ) -> Self {
        let mut new_results = results.clone();
        for (_, v) in new_results.iter_mut() {
            *v += 1;
        }
        let mut new_n_gram_counts = n_gram_counts.clone();
        for (k, v) in new_n_gram_counts.iter_mut() {
            *v += distinct_n_gram_counts.get(k).unwrap();
        }

        LaplaceSmoothingResult {
            results: new_results,
            n_gram_counts: new_n_gram_counts,
        }
    }
}

/// Represents the predictor.
///
/// This trait is used to define the predictor.
///
/// # Methods
///
/// * `predict` - Predicts the results.
pub trait Predict {
    /// Predicts the results.
    ///
    /// # Arguments
    ///
    /// * `data` - The timed sentence results.
    /// * `confusion_set` - The confusion set.
    /// * `number_of_ngrams` - The number of n-grams.
    /// * `number_of_distinct_ngrams` - The number of distinct n-grams.
    fn predict(
        &self,
        data: TimedSentenceResults,
        confusion_set: Vec<Vec<String>>,
        number_of_ngrams: HashMap<i32, i64>,
        number_of_distinct_ngrams: HashMap<i32, i64>,
    ) -> PredictionResults;
}

fn fill_results(
    d: &mut HashMap<String, HashMap<String, i32>>,
    qr: &crate::n_grams::solver::model::QueryResult,
    cs: &Vec<String>,
) {
    for w in cs.iter() {
        if qr.input.contains(w) {
            if d.contains_key(w) {
                let h = d.get_mut(w).unwrap();
                let mut found = false;
                for (k2, v2) in h.iter_mut() {
                    if k2.to_lowercase().as_str() == qr.input.to_lowercase().as_str() {
                        *v2 += qr.frequency;
                        found = true;
                    }
                }
                if !found {
                    h.insert(qr.input.clone(), qr.frequency);
                }
            } else {
                let mut h: HashMap<String, i32> = HashMap::new();
                h.insert(qr.input.clone(), qr.frequency);
                d.insert(w.clone(), h);
            }
        }
    }
}

/// Represents the maximum predictor.
///
/// This struct is used to define the maximum predictor.
pub struct MaxPredictor {}

impl Predict for MaxPredictor {
    fn predict(
        &self,
        data: TimedSentenceResults,
        confusion_set: Vec<Vec<String>>,
        number_of_ngrams: HashMap<i32, i64>,
        number_of_distinct_ngrams: HashMap<i32, i64>,
    ) -> PredictionResults {
        let mut pr: Vec<PredictionResult> = Vec::new();
        for r in data.results.iter() {
            for cs in confusion_set.iter() {
                if cs.contains(&r.word) {
                    let mut d: HashMap<String, HashMap<String, i32>> = HashMap::new();
                    let mut unigram_frequencies = HashMap::new();

                    for qr in r.results.iter() {
                        if qr.length == 1 {
                            unigram_frequencies.insert(qr.input.clone(), qr.frequency);
                        } else {
                            fill_results(&mut d, qr, cs);
                        }
                    }

                    let mut results = HashMap::new();

                    for (k, v) in d.iter() {
                        let laplace = LaplaceSmoothingResult::get(
                            v.clone(),
                            number_of_ngrams.clone(),
                            number_of_distinct_ngrams.clone(),
                        );
                        let mut max = -1.0;
                        let uf = unigram_frequencies.get(k).unwrap();
                        for (k1, v1) in laplace.results.iter() {
                            let p: f64 = ((*uf as f64)
                                / (*laplace.n_gram_counts.get(&1).unwrap() as f64))
                                * ((*v1 as f64)
                                    / (*laplace
                                        .n_gram_counts
                                        .get(&(k1.split_whitespace().count() as i32))
                                        .unwrap() as f64));
                            if p > max {
                                max = p;
                            }
                        }
                        let log = max.log(10.0) * -1.0;
                        let log = (log * 10000.0).round() / 10000.0;
                        results.insert(k.clone(), log);
                    }

                    pr.push(PredictionResult {
                        context: r.sentence.clone(),
                        word_examined: r.word.clone(),
                        results,
                    });
                    break;
                }
            }
        }

        PredictionResults {
            results: pr,
            time_elapsed: data.time_taken,
        }
    }
}

pub struct SumPredictor {}

impl Predict for SumPredictor {
    fn predict(
        &self,
        data: TimedSentenceResults,
        confusion_set: Vec<Vec<String>>,
        number_of_ngrams: HashMap<i32, i64>,
        number_of_distinct_ngrams: HashMap<i32, i64>,
    ) -> PredictionResults {
        let mut pr: Vec<PredictionResult> = Vec::new();
        for r in data.results.iter() {
            for cs in confusion_set.iter() {
                if cs.contains(&r.word) {
                    let mut d: HashMap<String, HashMap<String, i32>> = HashMap::new();
                    let mut unigram_frequencies = HashMap::new();

                    for qr in r.results.iter() {
                        if qr.length == 1 {
                            unigram_frequencies.insert(qr.input.clone(), qr.frequency);
                        } else {
                            fill_results(&mut d, qr, cs);
                        }
                    }

                    let mut results = HashMap::new();
                    for (k, v) in d.iter() {
                        let laplace = LaplaceSmoothingResult::get(
                            v.clone(),
                            number_of_ngrams.clone(),
                            number_of_distinct_ngrams.clone(),
                        );
                        let mut sum = 0.0;
                        let uf = unigram_frequencies.get(k).unwrap();
                        for (k1, v1) in laplace.results.iter() {
                            let p: f64 = ((*uf as f64)
                                / (*laplace.n_gram_counts.get(&1).unwrap() as f64))
                                * ((*v1 as f64)
                                    / (*laplace
                                        .n_gram_counts
                                        .get(&(k1.split_whitespace().count() as i32))
                                        .unwrap() as f64));
                            sum += p;
                        }
                        let log = sum.log(10.0) * -1.0;
                        let log = (log * 10000.0).round() / 10000.0;
                        results.insert(k.clone(), log);
                    }

                    pr.push(PredictionResult {
                        context: r.sentence.clone(),
                        word_examined: r.word.clone(),
                        results,
                    });
                    break;
                }
            }
        }

        PredictionResults {
            results: pr,
            time_elapsed: data.time_taken,
        }
    }
}

pub struct PowerSumPredictor {
    pub power: f64,
}

impl Predict for PowerSumPredictor {
    fn predict(
        &self,
        data: TimedSentenceResults,
        confusion_set: Vec<Vec<String>>,
        number_of_ngrams: HashMap<i32, i64>,
        number_of_distinct_ngrams: HashMap<i32, i64>,
    ) -> PredictionResults {
        let mut pr: Vec<PredictionResult> = Vec::new();
        for r in data.results.iter() {
            for cs in confusion_set.iter() {
                if cs.contains(&r.word) {
                    let mut d: HashMap<String, HashMap<String, i32>> = HashMap::new();
                    let mut unigram_frequencies = HashMap::new();

                    for qr in r.results.iter() {
                        if qr.length == 1 {
                            unigram_frequencies.insert(qr.input.clone(), qr.frequency);
                        } else {
                            fill_results(&mut d, qr, cs);
                        }
                    }

                    let mut results = HashMap::new();
                    for (k, v) in d.iter() {
                        let laplace = LaplaceSmoothingResult::get(
                            v.clone(),
                            number_of_ngrams.clone(),
                            number_of_distinct_ngrams.clone(),
                        );
                        let mut sum = 0.0;
                        let uf = unigram_frequencies.get(k).unwrap();
                        for (k1, v1) in laplace.results.iter() {
                            let length = k1.split_whitespace().count() as u32;
                            let v = *v1 as i32;
                            let p: f64 = ((*uf as f64)
                                / (*laplace.n_gram_counts.get(&1).unwrap() as f64))
                                * ((v as f64)
                                    / (*laplace
                                        .n_gram_counts
                                        .get(&(k1.split_whitespace().count() as i32))
                                        .unwrap() as f64))
                                    .powf(1 as f64 / (length as f64).powf(self.power) as f64);
                            sum += p;
                        }
                        let log = sum.log(10.0) * -1.0;
                        let log = (log * 10000.0).round() / 10000.0;
                        results.insert(k.clone(), log);
                    }

                    pr.push(PredictionResult {
                        context: r.sentence.clone(),
                        word_examined: r.word.clone(),
                        results,
                    });
                    break;
                }
            }
        }

        PredictionResults {
            results: pr,
            time_elapsed: data.time_taken,
        }
    }
}

/// Predicts the results.
///
/// # Arguments
///
/// * `predictor` - The predictor.
/// * `data` - The timed sentence results.
/// * `confusion_set` - The confusion set.
/// * `number_of_ngrams` - The number of n-grams.
///
/// # Returns
///
/// The prediction results.
pub fn predict<T>(
    predictor: T,
    data: TimedSentenceResults,
    confusion_set: Vec<Vec<String>>,
    number_of_ngrams: HashMap<i32, i64>,
    number_of_distinct_ngrams: HashMap<i32, i64>,
) -> PredictionResults
where
    T: Predict,
{
    predictor.predict(
        data,
        confusion_set,
        number_of_ngrams,
        number_of_distinct_ngrams,
    )
}
