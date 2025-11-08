use std::collections::{HashMap, HashSet};

use crate::hashcodes::error::ProblemError;

use super::input::ProblemInput;
use super::submission::{CacheAssignment, ProblemSubmission};

/// Detailed score summary returned to API consumers.
#[derive(Clone, Debug)]
pub struct ScoreBreakdown {
    pub total_score: usize,
    pub total_saved_latency: usize,
    pub total_request_count: usize,
    pub used_caches: usize,
}

impl ScoreBreakdown {
    pub fn into_json(self) -> serde_json::Value {
        serde_json::json!({
            "total_score": self.total_score,
            "total_saved_latency": self.total_saved_latency,
            "total_request_count": self.total_request_count,
            "used_caches": self.used_caches,
        })
    }
}

/// Validates cache assignments against the input constraints and computes the overall score.
pub fn score(
    input: &ProblemInput,
    submission: &ProblemSubmission,
) -> Result<ScoreBreakdown, ProblemError> {
    validate_submission(input, submission)?;

    let cache_video_map = build_cache_video_map(submission);

    let mut total_saved_latency = 0usize;
    let mut total_request_count = 0usize;

    for request in &input.requests {
        total_request_count += request.request_count;
        let endpoint = &input.endpoints[request.endpoint_id];
        let mut best_latency = endpoint.latency_to_datacenter;
        for cache in &endpoint.cache_latencies {
            if let Some(videos) = cache_video_map.get(&cache.cache_id) {
                if videos.contains(&request.video_id) && cache.latency < best_latency {
                    best_latency = cache.latency;
                }
            }
        }
        if best_latency < endpoint.latency_to_datacenter {
            total_saved_latency +=
                (endpoint.latency_to_datacenter - best_latency) * request.request_count;
        }
    }

    let total_score = if total_request_count == 0 {
        0
    } else {
        (total_saved_latency * 1000) / total_request_count
    };

    Ok(ScoreBreakdown {
        total_score,
        total_saved_latency,
        total_request_count,
        used_caches: submission.cache_assignments.len(),
    })
}

fn validate_submission(
    input: &ProblemInput,
    submission: &ProblemSubmission,
) -> Result<(), ProblemError> {
    let mut seen_caches = HashSet::new();

    for (index, assignment) in submission.cache_assignments.iter().enumerate() {
        if assignment.cache_id >= input.cache_servers {
            return Err(ProblemError::with_details(
                "unknown-cache-server",
                format!(
                    "Submission references cache {cache} but only {available} caches exist",
                    cache = assignment.cache_id,
                    available = input.cache_servers
                ),
                serde_json::json!({
                    "cache": assignment.cache_id,
                    "assignment_index": index,
                    "available_caches": input.cache_servers,
                }),
            ));
        }
        if !seen_caches.insert(assignment.cache_id) {
            return Err(ProblemError::with_details(
                "duplicate-cache-assignment",
                format!(
                    "Cache {cache} appears multiple times in the submission",
                    cache = assignment.cache_id
                ),
                serde_json::json!({
                    "cache": assignment.cache_id,
                    "assignment_index": index,
                }),
            ));
        }

        validate_cache_capacity(input, assignment, index)?;
    }

    Ok(())
}

fn validate_cache_capacity(
    input: &ProblemInput,
    assignment: &CacheAssignment,
    assignment_index: usize,
) -> Result<(), ProblemError> {
    let mut seen_videos = HashSet::new();
    let mut used_capacity = 0usize;
    for (position, &video_id) in assignment.videos.iter().enumerate() {
        if video_id >= input.video_sizes.len() {
            return Err(ProblemError::with_details(
                "unknown-video",
                format!(
                    "Cache {cache} includes video {video} which is outside the available range",
                    cache = assignment.cache_id,
                    video = video_id
                ),
                serde_json::json!({
                    "cache": assignment.cache_id,
                    "video": video_id,
                    "assignment_index": assignment_index,
                    "position": position + 1,
                    "available_videos": input.video_sizes.len(),
                }),
            ));
        }
        if !seen_videos.insert(video_id) {
            return Err(ProblemError::with_details(
                "duplicate-video",
                format!(
                    "Cache {cache} lists video {video} multiple times",
                    cache = assignment.cache_id,
                    video = video_id
                ),
                serde_json::json!({
                    "cache": assignment.cache_id,
                    "video": video_id,
                    "assignment_index": assignment_index,
                    "position": position + 1,
                }),
            ));
        }
        used_capacity += input.video_sizes[video_id];
    }

    if used_capacity > input.cache_capacity {
        return Err(ProblemError::with_details(
            "cache-capacity-exceeded",
            format!(
                "Cache {cache} uses {used}MB but only {capacity}MB are available",
                cache = assignment.cache_id,
                used = used_capacity,
                capacity = input.cache_capacity
            ),
            serde_json::json!({
                "cache": assignment.cache_id,
                "used": used_capacity,
                "capacity": input.cache_capacity,
                "assignment_index": assignment_index,
            }),
        ));
    }

    Ok(())
}

fn build_cache_video_map(submission: &ProblemSubmission) -> HashMap<usize, HashSet<usize>> {
    let mut map = HashMap::new();
    for assignment in &submission.cache_assignments {
        map.insert(
            assignment.cache_id,
            assignment.videos.iter().copied().collect(),
        );
    }
    map
}
