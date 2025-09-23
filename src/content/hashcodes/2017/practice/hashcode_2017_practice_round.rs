//! Google Hash Code 2017 Practice Round ("Pizza") implementation.

#[path = "hashcode_2017_practice_round_input.rs"]
pub mod input;
#[path = "hashcode_2017_practice_round_scorer.rs"]
pub mod scorer;
#[path = "hashcode_2017_practice_round_solver.rs"]
pub mod solver;
#[path = "hashcode_2017_practice_round_submission.rs"]
pub mod submission;

#[cfg(test)]
#[path = "hashcode_2017_practice_round_tests.rs"]
mod tests;

use crate::hashcodes::error::ProblemError;
use crate::hashcodes::response::ScoreResponse;
use input::{load_input, parse_input, ProblemInput};
use scorer::score;
use submission::{parse_submission, ProblemSubmission};

pub fn score_submission(
    input_file: &str,
    submission_text: &str,
) -> Result<ScoreResponse, ProblemError> {
    let raw_input = load_input(input_file)?;
    let parsed_input = parse_input(raw_input)?;
    let submission = parse_submission(submission_text)?;
    let breakdown = score(&parsed_input, &submission)?;
    Ok(ScoreResponse::with_details(
        "hashcode_2017_practice_round",
        input_file,
        breakdown.total_score as i64,
        breakdown.into_json(),
    ))
}

/// Shared ingredient representation from the problem statement.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Ingredient {
    Tomato,
    Mushroom,
}

impl Ingredient {
    pub fn from_char(c: char) -> Result<Self, ProblemError> {
        match c {
            'T' => Ok(Ingredient::Tomato),
            'M' => Ok(Ingredient::Mushroom),
            other => Err(ProblemError::with_details(
                "invalid-ingredient",
                format!("Unexpected ingredient character '{other}'"),
                serde_json::json!({ "character": other }),
            )),
        }
    }
}

/// Inclusive rectangle describing a slice.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Slice {
    pub start_row: usize,
    pub start_col: usize,
    pub end_row: usize,
    pub end_col: usize,
}

impl Slice {
    pub fn cell_count(&self) -> usize {
        (self.end_row - self.start_row + 1) * (self.end_col - self.start_col + 1)
    }
}

pub type ProblemInputData = ProblemInput;
pub type ProblemSubmissionData = ProblemSubmission;
