use super::input::{load_input, parse_input, CacheLatency, Endpoint, ProblemInput, Request};
use super::scorer::{score, ScoreBreakdown};
use super::solver::solve;
use super::submission::{parse_submission, CacheAssignment, ProblemSubmission};

fn example_input() -> ProblemInput {
    let raw = load_input("me_at_the_zoo").expect("dataset");
    parse_input(raw).expect("parse input")
}

fn minimal_input() -> ProblemInput {
    ProblemInput {
        video_sizes: vec![80, 30],
        endpoints: vec![Endpoint {
            latency_to_datacenter: 100,
            cache_latencies: vec![CacheLatency {
                cache_id: 0,
                latency: 50,
            }],
        }],
        requests: vec![Request {
            video_id: 0,
            endpoint_id: 0,
            request_count: 100,
        }],
        cache_servers: 1,
        cache_capacity: 100,
    }
}

#[test]
fn solver_produces_valid_submission_for_example() {
    let input = example_input();
    let submission = solve(&input);
    let breakdown = score(&input, &submission).expect("solver output should score");
    assert!(breakdown.total_score > 0);
    assert!(breakdown.total_saved_latency >= breakdown.total_score);
}

#[test]
fn load_input_rejects_unknown_dataset() {
    let err = load_input("unknown").expect_err("should fail");
    assert_eq!(err.code, "unknown-input-file");
}

#[test]
fn parse_input_requires_header() {
    let err = parse_input("").expect_err("missing header");
    assert_eq!(err.code, "missing-header");
}

#[test]
fn parse_input_validates_header_numbers() {
    let err = parse_input("V E R C X\n").expect_err("invalid header numbers");
    assert_eq!(err.code, "invalid-number");
}

#[test]
fn parse_input_rejects_extra_header_values() {
    let err = parse_input("1 1 1 1 1 1\n0\n0 0\n0 0 1\n").expect_err("extra header");
    assert_eq!(err.code, "invalid-header");
}

#[test]
fn parse_input_requires_video_sizes_line() {
    let err = parse_input("1 0 0 0 0\n").expect_err("missing sizes");
    assert_eq!(err.code, "missing-video-sizes");
}

#[test]
fn parse_input_validates_video_count() {
    let err = parse_input("2 0 0 0 0\n10\n").expect_err("video count");
    assert_eq!(err.code, "invalid-video-count");
}

#[test]
fn parse_input_detects_invalid_endpoint_reference() {
    let err = parse_input("1 0 1 0 0\n10\n0 0 1\n").expect_err("invalid request");
    assert_eq!(err.code, "unknown-endpoint");
}

#[test]
fn parse_input_detects_invalid_cache_connection() {
    let err = parse_input("1 1 0 1 100\n10\n100 1\n0 10 20\n").expect_err("invalid cache");
    assert_eq!(err.code, "invalid-cache-connection");
}

#[test]
fn parse_input_detects_unknown_cache_id() {
    let err = parse_input("1 1 0 1 100\n10\n100 1\n1 10\n").expect_err("unknown cache");
    assert_eq!(err.code, "unknown-cache-server");
}

#[test]
fn parse_input_detects_duplicate_cache_reference() {
    let err = parse_input("1 1 0 2 100\n10\n100 2\n0 10\n0 20\n").expect_err("duplicate cache");
    assert_eq!(err.code, "duplicate-cache-connection");
}

#[test]
fn parse_input_detects_invalid_request_endpoint() {
    let err = parse_input("1 1 1 0 0\n10\n100 0\n0 1 5\n").expect_err("unknown endpoint");
    assert_eq!(err.code, "unknown-endpoint");
}

#[test]
fn parse_input_detects_invalid_request_video() {
    let err = parse_input("1 1 1 0 0\n10\n100 0\n1 1 5\n").expect_err("unknown video");
    assert_eq!(err.code, "unknown-video");
}

#[test]
fn parse_input_detects_zero_request_count() {
    let err = parse_input("1 1 1 0 0\n10\n100 0\n0 0 0\n").expect_err("zero request");
    assert_eq!(err.code, "invalid-request-count");
}

#[test]
fn submission_parser_detects_missing_count() {
    let err = parse_submission("").expect_err("missing");
    assert_eq!(err.code, "missing-cache-count");
}

#[test]
fn submission_parser_validates_count_as_number() {
    let err = parse_submission("two\n0\n").expect_err("invalid count");
    assert_eq!(err.code, "invalid-cache-count");
}

#[test]
fn submission_parser_detects_mismatched_count() {
    let err = parse_submission("2\n0\n").expect_err("count mismatch");
    assert_eq!(err.code, "cache-count-mismatch");
}

#[test]
fn submission_parser_detects_invalid_cache_id() {
    let err = parse_submission("1\nX\n").expect_err("invalid cache id");
    assert_eq!(err.code, "invalid-cache-id");
}

#[test]
fn submission_parser_detects_invalid_video_id() {
    let err = parse_submission("1\n0 X\n").expect_err("invalid video id");
    assert_eq!(err.code, "invalid-video-id");
}

#[test]
fn submission_parser_detects_duplicate_videos() {
    let err = parse_submission("1\n0 1 1\n").expect_err("duplicate video");
    assert_eq!(err.code, "duplicate-video");
}

#[test]
fn scoring_rejects_unknown_cache() {
    let input = minimal_input();
    let submission = ProblemSubmission {
        cache_assignments: vec![CacheAssignment {
            cache_id: 1,
            videos: vec![0],
        }],
    };
    let err = score(&input, &submission).expect_err("invalid cache");
    assert_eq!(err.code, "unknown-cache-server");
}

#[test]
fn scoring_rejects_duplicate_cache_assignment() {
    let input = minimal_input();
    let submission = ProblemSubmission {
        cache_assignments: vec![
            CacheAssignment {
                cache_id: 0,
                videos: vec![0],
            },
            CacheAssignment {
                cache_id: 0,
                videos: vec![1],
            },
        ],
    };
    let err = score(&input, &submission).expect_err("duplicate cache");
    assert_eq!(err.code, "duplicate-cache-assignment");
}

#[test]
fn scoring_rejects_unknown_video() {
    let input = minimal_input();
    let submission = ProblemSubmission {
        cache_assignments: vec![CacheAssignment {
            cache_id: 0,
            videos: vec![5],
        }],
    };
    let err = score(&input, &submission).expect_err("unknown video");
    assert_eq!(err.code, "unknown-video");
}

#[test]
fn scoring_rejects_capacity_overflow() {
    let input = minimal_input();
    let submission = ProblemSubmission {
        cache_assignments: vec![CacheAssignment {
            cache_id: 0,
            videos: vec![0, 1],
        }],
    };
    let err = score(&input, &submission).expect_err("capacity overflow");
    assert_eq!(err.code, "cache-capacity-exceeded");
}

#[test]
fn scoring_accepts_valid_submission() {
    let input = minimal_input();
    let submission = ProblemSubmission {
        cache_assignments: vec![CacheAssignment {
            cache_id: 0,
            videos: vec![0],
        }],
    };
    let breakdown: ScoreBreakdown = score(&input, &submission).expect("score");
    assert_eq!(breakdown.total_score, 50000);
    assert_eq!(breakdown.total_saved_latency, 5000);
    assert_eq!(breakdown.total_request_count, 100);
    assert_eq!(breakdown.used_caches, 1);
}
