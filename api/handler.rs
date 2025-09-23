use serde_json::json;
use vercel_runtime::{run, Body, Error, Request, Response, StatusCode};

use hashy::hashcodes::{error::ProblemError, ProblemKey};

#[tokio::main]
async fn main() -> Result<(), Error> {
    run(handler).await
}

pub async fn handler(req: Request) -> Result<Response<Body>, Error> {
    let (parts, body_stream) = req.into_parts();
    if parts.method.as_str() != "POST" {
        return response(
            StatusCode::METHOD_NOT_ALLOWED,
            &json!({
                "status": "error",
                "error": {
                    "code": "method-not-allowed",
                    "message": "Only POST is supported for scoring endpoints",
                }
            }),
        );
    }

    let path = parts.uri.path();
    let segments: Vec<&str> = path
        .trim_start_matches('/')
        .split('/')
        .filter(|segment| !segment.is_empty())
        .collect();

    let relevant_segments: &[&str] = match segments.as_slice() {
        ["api", rest @ ..] => rest,
        other => other,
    };

    let (problem_key, input_file) = match relevant_segments {
        ["hashcodes", year, round, input_file] => match ProblemKey::from_route(year, round) {
            Some(key) => (key, *input_file),
            None => {
                return response(
                    StatusCode::NOT_FOUND,
                    &json!({
                        "status": "error",
                        "error": {
                            "code": "unknown-problem",
                            "message": format!(
                                "No Hash Code implementation registered for year={year} round={round}"
                            ),
                        }
                    }),
                );
            }
        },
        _ => {
            return response(
                StatusCode::NOT_FOUND,
                &json!({
                    "status": "error",
                    "error": {
                        "code": "unknown-endpoint",
                        "message": "Expected /api/hashcodes/{year}/{round}/{input_file}",
                    }
                }),
            );
        }
    };

    let submission_text = match body_stream {
        Body::Empty => String::new(),
        Body::Text(text) => text,
        Body::Binary(bytes) => String::from_utf8(bytes).map_err(Error::from)?,
    };

    match problem_key.score_submission(input_file, submission_text.as_str()) {
        Ok(score) => response(
            StatusCode::OK,
            &json!({
                "status": "ok",
                "result": score,
            }),
        ),
        Err(problem_error) => problem_error_response(problem_error),
    }
}

fn problem_error_response(error: ProblemError) -> Result<Response<Body>, Error> {
    let status = match error.code {
        "unknown-input-file" => StatusCode::NOT_FOUND,
        _ => StatusCode::BAD_REQUEST,
    };
    response(
        status,
        &json!({
            "status": "error",
            "error": error,
        }),
    )
}

fn response(status: StatusCode, payload: &serde_json::Value) -> Result<Response<Body>, Error> {
    let body = Body::from(payload.to_string());
    Response::builder()
        .status(status)
        .header("Content-Type", "application/json")
        .body(body)
        .map_err(Error::from)
}
