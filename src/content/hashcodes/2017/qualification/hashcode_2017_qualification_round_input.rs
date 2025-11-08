use std::collections::HashSet;

use crate::hashcodes::error::ProblemError;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProblemInput {
    pub video_sizes: Vec<usize>,
    pub endpoints: Vec<Endpoint>,
    pub requests: Vec<Request>,
    pub cache_servers: usize,
    pub cache_capacity: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Endpoint {
    pub latency_to_datacenter: usize,
    pub cache_latencies: Vec<CacheLatency>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CacheLatency {
    pub cache_id: usize,
    pub latency: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Request {
    pub video_id: usize,
    pub endpoint_id: usize,
    pub request_count: usize,
}

pub fn load_input(input_file: &str) -> Result<&'static str, ProblemError> {
    let normalized = normalize_dataset_name(input_file);
    match normalized.as_str() {
        "me_at_the_zoo" | "example" => Ok(include_str!(
            "hashcode_2017_qualification_round.in/me_at_the_zoo.in"
        )),
        "trending_today" | "trending" => Ok(include_str!(
            "hashcode_2017_qualification_round.in/trending_today.in"
        )),
        "videos_worth_spreading" | "videos" => Ok(include_str!(
            "hashcode_2017_qualification_round.in/videos_worth_spreading.in"
        )),
        "kittens" | "kittens_worth_spreading" => Ok(include_str!(
            "hashcode_2017_qualification_round.in/kittens.in.txt"
        )),
        other => Err(ProblemError::with_details(
            "unknown-input-file",
            format!("Unsupported 2017 qualification dataset '{other}'"),
            serde_json::json!({
                "requested": input_file,
                "available": [
                    "me_at_the_zoo",
                    "trending_today",
                    "videos_worth_spreading",
                    "kittens",
                ],
            }),
        )),
    }
}

fn normalize_dataset_name(input_file: &str) -> String {
    let mut name = input_file.trim().to_string();
    loop {
        if let Some(stripped) = name.strip_suffix(".in") {
            name = stripped.to_string();
            continue;
        }
        if let Some(stripped) = name.strip_suffix(".txt") {
            name = stripped.to_string();
            continue;
        }
        break;
    }
    name
}

pub fn parse_input(raw: &str) -> Result<ProblemInput, ProblemError> {
    let mut lines = raw.lines();
    let header = lines.next().ok_or_else(|| {
        ProblemError::new(
            "missing-header",
            "Input file must start with the problem header",
        )
    })?;

    let mut header_parts = header.split_whitespace();
    let video_count = parse_usize(header_parts.next(), "video_count", 0)?;
    let endpoint_count = parse_usize(header_parts.next(), "endpoint_count", 0)?;
    let request_count = parse_usize(header_parts.next(), "request_count", 0)?;
    let cache_server_count = parse_usize(header_parts.next(), "cache_server_count", 0)?;
    let cache_capacity = parse_usize(header_parts.next(), "cache_capacity", 0)?;

    if header_parts.next().is_some() {
        return Err(ProblemError::new(
            "invalid-header",
            "Header line contains extra unexpected values",
        ));
    }

    let video_sizes_line = lines.next().ok_or_else(|| {
        ProblemError::new(
            "missing-video-sizes",
            "Second line must contain the video sizes",
        )
    })?;
    let video_sizes: Vec<usize> = video_sizes_line
        .split_whitespace()
        .enumerate()
        .map(|(index, value)| {
            value.parse::<usize>().map_err(|_err| {
                ProblemError::with_details(
                    "invalid-video-size",
                    format!("Video size entry {index} is not a non-negative integer"),
                    serde_json::json!({ "index": index, "value": value }),
                )
            })
        })
        .collect::<Result<_, _>>()?;

    if video_sizes.len() != video_count {
        return Err(ProblemError::with_details(
            "invalid-video-count",
            format!(
                "Header declares {video_count} videos but {actual} sizes were provided",
                actual = video_sizes.len()
            ),
            serde_json::json!({
                "declared": video_count,
                "actual": video_sizes.len(),
            }),
        ));
    }

    let mut endpoints = Vec::with_capacity(endpoint_count);
    for endpoint_index in 0..endpoint_count {
        let endpoint_line = lines.next().ok_or_else(|| {
            ProblemError::with_details(
                "missing-endpoint",
                format!("Endpoint {endpoint_index} header is missing"),
                serde_json::json!({ "endpoint": endpoint_index }),
            )
        })?;
        let mut endpoint_parts = endpoint_line.split_whitespace();
        let latency_to_datacenter = parse_usize(
            endpoint_parts.next(),
            "latency_to_datacenter",
            endpoint_index,
        )?;
        let cache_connection_count = parse_usize(
            endpoint_parts.next(),
            "cache_connection_count",
            endpoint_index,
        )?;

        if endpoint_parts.next().is_some() {
            return Err(ProblemError::with_details(
                "invalid-endpoint-header",
                format!("Endpoint {endpoint_index} header has extra values"),
                serde_json::json!({
                    "endpoint": endpoint_index,
                    "line": endpoint_line,
                }),
            ));
        }

        let mut cache_latencies = Vec::with_capacity(cache_connection_count);
        let mut seen_caches = HashSet::with_capacity(cache_connection_count);
        for connection_index in 0..cache_connection_count {
            let cache_line = lines.next().ok_or_else(|| {
                ProblemError::with_details(
                    "missing-cache-connection",
                    format!(
                        "Endpoint {endpoint_index} declares {cache_connection_count} cache connections but is missing line {connection_index}",
                    ),
                    serde_json::json!({
                        "endpoint": endpoint_index,
                        "expected_connections": cache_connection_count,
                        "missing_index": connection_index,
                    }),
                )
            })?;
            let mut cache_parts = cache_line.split_whitespace();
            let cache_id = parse_usize(cache_parts.next(), "cache_id", endpoint_index)?;
            if cache_id >= cache_server_count {
                return Err(ProblemError::with_details(
                    "unknown-cache-server",
                    format!(
                        "Endpoint {endpoint_index} references cache {cache_id} but only {cache_server_count} caches exist",
                    ),
                    serde_json::json!({
                        "endpoint": endpoint_index,
                        "cache": cache_id,
                        "available_caches": cache_server_count,
                    }),
                ));
            }
            if !seen_caches.insert(cache_id) {
                return Err(ProblemError::with_details(
                    "duplicate-cache-connection",
                    format!("Endpoint {endpoint_index} declares cache {cache_id} multiple times"),
                    serde_json::json!({
                        "endpoint": endpoint_index,
                        "cache": cache_id,
                    }),
                ));
            }
            let latency = parse_usize(cache_parts.next(), "cache_latency", endpoint_index)?;
            if cache_parts.next().is_some() {
                return Err(ProblemError::with_details(
                    "invalid-cache-connection",
                    format!("Endpoint {endpoint_index} cache connection line has extra values"),
                    serde_json::json!({
                        "endpoint": endpoint_index,
                        "line": cache_line,
                    }),
                ));
            }
            cache_latencies.push(CacheLatency { cache_id, latency });
        }

        endpoints.push(Endpoint {
            latency_to_datacenter,
            cache_latencies,
        });
    }

    let mut requests = Vec::with_capacity(request_count);
    for request_index in 0..request_count {
        let request_line = lines.next().ok_or_else(|| {
            ProblemError::with_details(
                "missing-request",
                format!("Request {request_index} is missing"),
                serde_json::json!({ "request": request_index }),
            )
        })?;
        let mut request_parts = request_line.split_whitespace();
        let video_id = parse_usize(request_parts.next(), "video_id", request_index)?;
        if video_id >= video_count {
            return Err(ProblemError::with_details(
                "unknown-video",
                format!(
                    "Request {request_index} references video {video_id} but only {video_count} exist"
                ),
                serde_json::json!({
                    "request": request_index,
                    "video": video_id,
                    "available_videos": video_count,
                }),
            ));
        }
        let endpoint_id = parse_usize(request_parts.next(), "endpoint_id", request_index)?;
        if endpoint_id >= endpoint_count {
            return Err(ProblemError::with_details(
                "unknown-endpoint",
                format!(
                    "Request {request_index} references endpoint {endpoint_id} but only {endpoint_count} exist"
                ),
                serde_json::json!({
                    "request": request_index,
                    "endpoint": endpoint_id,
                    "available_endpoints": endpoint_count,
                }),
            ));
        }
        let request_count_value =
            parse_usize(request_parts.next(), "request_count", request_index)?;
        if request_count_value == 0 {
            return Err(ProblemError::with_details(
                "invalid-request-count",
                format!("Request {request_index} must request at least one video"),
                serde_json::json!({
                    "request": request_index,
                    "value": request_count_value,
                }),
            ));
        }
        if request_parts.next().is_some() {
            return Err(ProblemError::with_details(
                "invalid-request",
                format!("Request line {request_index} has extra values"),
                serde_json::json!({
                    "request": request_index,
                    "line": request_line,
                }),
            ));
        }
        requests.push(Request {
            video_id,
            endpoint_id,
            request_count: request_count_value,
        });
    }

    Ok(ProblemInput {
        video_sizes,
        endpoints,
        requests,
        cache_servers: cache_server_count,
        cache_capacity,
    })
}

fn parse_usize(
    value: Option<&str>,
    field: &str,
    context_index: usize,
) -> Result<usize, ProblemError> {
    let raw = value.ok_or_else(|| {
        ProblemError::with_details(
            "missing-value",
            format!("Missing value for '{field}'"),
            serde_json::json!({ "field": field, "context": context_index }),
        )
    })?;
    raw.parse::<usize>().map_err(|_err| {
        ProblemError::with_details(
            "invalid-number",
            format!("Could not parse '{field}' as a non-negative integer"),
            serde_json::json!({
                "field": field,
                "value": raw,
                "context": context_index,
            }),
        )
    })
}
