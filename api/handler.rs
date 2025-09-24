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

    let (problem_key, input_file_raw) = match relevant_segments {
        ["hashcodes", year, round, input_file] => match ProblemKey::from_route(year, round) {
            Some(key) => (key, (*input_file).to_owned()),
            None => {
                return unknown_problem_response(year, round);
            }
        },
        _ => match route_from_query(parts.uri.query()) {
            RouteMatch::Matched { key, input_file } => (key, input_file),
            RouteMatch::UnknownProblem { year, round } => {
                return unknown_problem_response(&year, &round);
            }
            RouteMatch::NotMatched => {
                return unknown_endpoint_response();
            }
        },
    };

    let input_file = input_file_raw.trim();

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

fn unknown_problem_response(year: &str, round: &str) -> Result<Response<Body>, Error> {
    response(
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
    )
}

fn unknown_endpoint_response() -> Result<Response<Body>, Error> {
    response(
        StatusCode::NOT_FOUND,
        &json!({
            "status": "error",
            "error": {
                "code": "unknown-endpoint",
                "message": "Expected /api/hashcodes/{year}/{round}/{input_file}",
            }
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

enum RouteMatch {
    Matched { key: ProblemKey, input_file: String },
    UnknownProblem { year: String, round: String },
    NotMatched,
}

fn route_from_query(query: Option<&str>) -> RouteMatch {
    let query = match query {
        Some(value) if !value.is_empty() => value,
        _ => return RouteMatch::NotMatched,
    };

    let mut year: Option<String> = None;
    let mut round: Option<String> = None;
    let mut dataset: Option<String> = None;

    for pair in query.split('&') {
        if pair.is_empty() {
            continue;
        }

        let mut parts = pair.splitn(2, '=');
        let name = parts.next().unwrap_or("");
        let raw_value = parts.next().unwrap_or("");
        let decoded =
            decode_uri_component(raw_value).unwrap_or_else(|| raw_value.replace('+', " "));
        let value = decoded.trim().to_owned();

        match name {
            "year" if year.is_none() => {
                year = Some(value);
            }
            "round" if round.is_none() => {
                round = Some(value);
            }
            "dataset" | "input" if dataset.is_none() => {
                dataset = Some(value);
            }
            _ => {}
        }
    }

    let (year, round, dataset) = match (year, round, dataset) {
        (Some(year), Some(round), Some(dataset)) => (year, round, dataset),
        _ => return RouteMatch::NotMatched,
    };

    match ProblemKey::from_route(year.as_str(), round.as_str()) {
        Some(key) => RouteMatch::Matched {
            key,
            input_file: dataset,
        },
        None => RouteMatch::UnknownProblem { year, round },
    }
}

fn decode_uri_component(value: &str) -> Option<String> {
    let bytes = value.as_bytes();
    let mut result = Vec::with_capacity(bytes.len());

    let mut index = 0;
    while index < bytes.len() {
        match bytes[index] {
            b'%' => {
                if index + 2 >= bytes.len() {
                    return None;
                }
                let high = from_hex_digit(bytes[index + 1])?;
                let low = from_hex_digit(bytes[index + 2])?;
                result.push((high << 4) | low);
                index += 3;
            }
            b'+' => {
                result.push(b' ');
                index += 1;
            }
            byte => {
                result.push(byte);
                index += 1;
            }
        }
    }

    String::from_utf8(result).ok()
}

fn from_hex_digit(digit: u8) -> Option<u8> {
    match digit {
        b'0'..=b'9' => Some(digit - b'0'),
        b'a'..=b'f' => Some(digit - b'a' + 10),
        b'A'..=b'F' => Some(digit - b'A' + 10),
        _ => None,
    }
}
