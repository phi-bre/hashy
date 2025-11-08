//! Core helpers and routing glue for Google Hash Code scoring modules.

pub mod error;
pub mod response;

#[path = "../content/hashcodes/2017/practice/hashcode_2017_practice_round.rs"]
mod hashcode_2017_practice_round;
#[path = "../content/hashcodes/2017/qualification/hashcode_2017_qualification_round.rs"]
mod hashcode_2017_qualification_round;

use error::ProblemError;
use response::ScoreResponse;

/// Identifier for the supported Hash Code problem modules.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProblemKey {
    HashCode2017Practice,
    HashCode2017Qualification,
}

impl ProblemKey {
    pub fn from_route(year: &str, round: &str) -> Option<Self> {
        match (year, round) {
            ("2017", "practice") | ("2017", "practice_round") | ("2017", "practice_problem") => {
                Some(ProblemKey::HashCode2017Practice)
            }
            ("2017", "qualification")
            | ("2017", "qualification_round")
            | ("2017", "streaming_videos") => Some(ProblemKey::HashCode2017Qualification),
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
            ProblemKey::HashCode2017Qualification => {
                hashcode_2017_qualification_round::score_submission(input_file, submission_text)
            }
        }
    }
}
