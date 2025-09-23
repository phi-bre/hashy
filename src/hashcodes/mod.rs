//! Core helpers and routing glue for Google Hash Code scoring modules.

pub mod error;
pub mod response;

#[path = "../content/hashcodes/2017/practice/hashcode_2017_practice_round.rs"]
mod hashcode_2017_practice_round;

use error::ProblemError;
use response::ScoreResponse;

/// Identifier for the supported Hash Code problem modules.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProblemKey {
    HashCode2017Practice,
}

impl ProblemKey {
    pub fn from_route(year: &str, round: &str) -> Option<Self> {
        match (year, round) {
            ("2017", "practice") | ("2017", "practice_round") | ("2017", "practice_problem") => {
                Some(ProblemKey::HashCode2017Practice)
            }
            _ => None,
        }
    }

    pub fn score_submission(
        self,
        input_file: &str,
        submission_text: &str,
    ) -> Result<ScoreResponse, ProblemError> {
        match self {
            ProblemKey::HashCode2017Practice => {
                hashcode_2017_practice_round::score_submission(input_file, submission_text)
            }
        }
    }
}
