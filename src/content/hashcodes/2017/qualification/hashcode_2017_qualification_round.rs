//! Google Hash Code 2017 Qualification Round ("Streaming Videos") implementation.

#[path = "hashcode_2017_qualification_round_input.rs"]
pub mod input;
#[path = "hashcode_2017_qualification_round_scorer.rs"]
pub mod scorer;
#[path = "hashcode_2017_qualification_round_solver.rs"]
pub mod solver;
#[path = "hashcode_2017_qualification_round_submission.rs"]
pub mod submission;

#[cfg(test)]
#[path = "hashcode_2017_qualification_round_tests.rs"]
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
    let parsed_input: ProblemInput = parse_input(raw_input)?;
    let submission: ProblemSubmission = parse_submission(submission_text)?;
    let breakdown = score(&parsed_input, &submission)?;
    Ok(ScoreResponse::with_details(
        "hashcode_2017_qualification_round",
        input_file,
        breakdown.total_score as i64,
        breakdown.into_json(),
    ))
}

pub type ProblemInputData = ProblemInput;
pub type ProblemSubmissionData = ProblemSubmission;
