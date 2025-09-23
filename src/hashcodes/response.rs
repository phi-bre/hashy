use serde::Serialize;

/// Successful score calculation payload returned by the API.
#[derive(Debug, Serialize)]
pub struct ScoreResponse {
    pub problem: &'static str,
    pub input_file: String,
    pub score: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

impl ScoreResponse {
    pub fn new(problem: &'static str, input_file: &str, score: i64) -> Self {
        Self {
            problem,
            input_file: input_file.to_string(),
            score,
            details: None,
        }
    }

    pub fn with_details(
        problem: &'static str,
        input_file: &str,
        score: i64,
        details: serde_json::Value,
    ) -> Self {
        Self {
            problem,
            input_file: input_file.to_string(),
            score,
            details: Some(details),
        }
    }
}
