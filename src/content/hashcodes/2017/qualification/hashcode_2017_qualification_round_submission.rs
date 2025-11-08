use std::collections::HashSet;

use crate::hashcodes::error::ProblemError;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProblemSubmission {
    pub cache_assignments: Vec<CacheAssignment>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CacheAssignment {
    pub cache_id: usize,
    pub videos: Vec<usize>,
}

pub fn parse_submission(raw: &str) -> Result<ProblemSubmission, ProblemError> {
    let mut lines = raw.lines();
    let header = lines.next().ok_or_else(|| {
        ProblemError::new(
            "missing-cache-count",
            "Submission must start with the number of cache servers that store videos",
        )
    })?;

    let declared_count = header.trim().parse::<usize>().map_err(|_err| {
        ProblemError::with_details(
            "invalid-cache-count",
            "First line must be a non-negative integer",
            serde_json::json!({ "line": 0, "value": header.trim() }),
        )
    })?;

    let mut cache_assignments = Vec::with_capacity(declared_count);

    for (line_index, line) in lines.enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let tokens: Vec<&str> = line.split_whitespace().collect();
        if tokens.is_empty() {
            continue;
        }
        let cache_id = tokens[0].parse::<usize>().map_err(|_err| {
            ProblemError::with_details(
                "invalid-cache-id",
                format!(
                    "Cache assignment line {line_no} does not start with a valid cache id",
                    line_no = line_index + 1
                ),
                serde_json::json!({ "line": line_index + 1, "content": line }),
            )
        })?;

        let mut videos = Vec::new();
        let mut seen_videos = HashSet::new();
        for (position, token) in tokens.iter().enumerate().skip(1) {
            let video_id = token.parse::<usize>().map_err(|_err| {
                ProblemError::with_details(
                    "invalid-video-id",
                    format!(
                        "Video entry {position} on line {line_no} is not a non-negative integer",
                        line_no = line_index + 1
                    ),
                    serde_json::json!({
                        "line": line_index + 1,
                        "position": position,
                        "value": token,
                    }),
                )
            })?;
            if !seen_videos.insert(video_id) {
                return Err(ProblemError::with_details(
                    "duplicate-video",
                    format!(
                        "Cache {cache_id} lists video {video_id} multiple times on line {line_no}",
                        line_no = line_index + 1
                    ),
                    serde_json::json!({
                        "line": line_index + 1,
                        "cache": cache_id,
                        "video": video_id,
                    }),
                ));
            }
            videos.push(video_id);
        }

        cache_assignments.push(CacheAssignment { cache_id, videos });
    }

    if cache_assignments.len() != declared_count {
        return Err(ProblemError::with_details(
            "cache-count-mismatch",
            format!(
                "Submission declares {declared_count} caches but provides {actual}",
                actual = cache_assignments.len()
            ),
            serde_json::json!({
                "declared": declared_count,
                "actual": cache_assignments.len(),
            }),
        ));
    }

    Ok(ProblemSubmission { cache_assignments })
}
