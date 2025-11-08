use std::collections::HashMap;

use super::input::ProblemInput;
use super::submission::{CacheAssignment, ProblemSubmission};

/// Produces a deterministic heuristic solution by filling each cache with the most
/// beneficial videos based on total latency savings across connected endpoints.
pub fn solve(input: &ProblemInput) -> ProblemSubmission {
    if input.cache_servers == 0 {
        return ProblemSubmission {
            cache_assignments: Vec::new(),
        };
    }

    let mut endpoint_cache_latency: Vec<HashMap<usize, usize>> =
        Vec::with_capacity(input.endpoints.len());
    for endpoint in &input.endpoints {
        let mut map = HashMap::new();
        for cache in &endpoint.cache_latencies {
            map.insert(cache.cache_id, cache.latency);
        }
        endpoint_cache_latency.push(map);
    }

    let mut cache_assignments = Vec::new();

    for cache_id in 0..input.cache_servers {
        let mut video_candidates = Vec::with_capacity(input.video_sizes.len());
        for (video_id, &size) in input.video_sizes.iter().enumerate() {
            if size > input.cache_capacity {
                continue;
            }
            let mut total_gain = 0usize;
            for request in &input.requests {
                if request.video_id != video_id {
                    continue;
                }
                if let Some(&cache_latency) =
                    endpoint_cache_latency[request.endpoint_id].get(&cache_id)
                {
                    let endpoint = &input.endpoints[request.endpoint_id];
                    if cache_latency < endpoint.latency_to_datacenter {
                        total_gain += (endpoint.latency_to_datacenter - cache_latency)
                            * request.request_count;
                    }
                }
            }
            if total_gain > 0 {
                video_candidates.push((video_id, total_gain));
            }
        }

        if video_candidates.is_empty() {
            continue;
        }

        video_candidates.sort_by(|&(video_a, gain_a), &(video_b, gain_b)| {
            let size_a = input.video_sizes[video_a];
            let size_b = input.video_sizes[video_b];
            let lhs = gain_a * size_b;
            let rhs = gain_b * size_a;
            lhs.cmp(&rhs)
                .reverse()
                .then_with(|| gain_a.cmp(&gain_b).reverse())
                .then_with(|| size_a.cmp(&size_b))
                .then_with(|| video_a.cmp(&video_b))
        });

        let mut remaining_capacity = input.cache_capacity;
        let mut selected_videos = Vec::new();
        for (video_id, _) in video_candidates {
            let size = input.video_sizes[video_id];
            if size <= remaining_capacity {
                selected_videos.push(video_id);
                remaining_capacity -= size;
            }
        }

        if !selected_videos.is_empty() {
            cache_assignments.push(CacheAssignment {
                cache_id,
                videos: selected_videos,
            });
        }
    }

    ProblemSubmission { cache_assignments }
}
