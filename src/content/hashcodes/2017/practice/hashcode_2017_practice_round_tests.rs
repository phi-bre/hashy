use super::input::{load_input, parse_input};
use super::scorer::{score, ScoreBreakdown};
use super::solver::solve;
use super::submission::{parse_submission, ProblemSubmission};
use super::Slice;

fn example_input() -> super::input::ProblemInput {
    let raw_input = load_input("a_example").expect("dataset");
    parse_input(raw_input).expect("parse input")
}

fn parse_submission_str(raw: &str) -> ProblemSubmission {
    parse_submission(raw).expect("valid submission")
}

#[test]
fn solver_produces_valid_submission_for_example() {
    let parsed_input = example_input();
    let submission = solve(&parsed_input);
    let breakdown = score(&parsed_input, &submission).expect("scoring should succeed");
    assert!(breakdown.total_score > 0);
    assert_eq!(breakdown.covered_cells, breakdown.total_score);
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
fn parse_input_detects_missing_values() {
    let err = parse_input("3 5 1\nTT\n").expect_err("missing value");
    assert_eq!(err.code, "missing-value");
}

#[test]
fn parse_input_validates_numbers() {
    let err = parse_input("3 X 1 6\nTTT\nTTT\nTTT\n").expect_err("invalid number");
    assert_eq!(err.code, "invalid-number");
}

#[test]
fn parse_input_rejects_extra_header_values() {
    let err = parse_input("3 5 1 6 10\nTTTTT\nTTTTT\nTTTTT\n").expect_err("extra header");
    assert_eq!(err.code, "invalid-header");
}

#[test]
fn parse_input_detects_too_many_rows() {
    let err = parse_input("1 1 1 1\nT\nT\n").expect_err("too many rows");
    assert_eq!(err.code, "too-many-rows");
}

#[test]
fn parse_input_detects_not_enough_rows() {
    let err = parse_input("2 2 1 2\nTM\n").expect_err("not enough rows");
    assert_eq!(err.code, "not-enough-rows");
}

#[test]
fn parse_input_detects_invalid_row_length() {
    let err = parse_input("1 2 1 2\nT\n").expect_err("row length");
    assert_eq!(err.code, "invalid-row-length");
}

#[test]
fn parse_input_detects_invalid_ingredient() {
    let err = parse_input("1 1 1 1\nX\n").expect_err("invalid ingredient");
    assert_eq!(err.code, "invalid-ingredient");
}

#[test]
fn submission_parser_detects_missing_count() {
    let err = parse_submission("").expect_err("should fail");
    assert_eq!(err.code, "missing-slice-count");
}

#[test]
fn submission_parser_validates_count_as_number() {
    let err = parse_submission("two\n0 0 0 0\n").expect_err("invalid count");
    assert_eq!(err.code, "invalid-slice-count");
}

#[test]
fn submission_parser_validates_slice_format() {
    let err = parse_submission("1\n0 0 0\n").expect_err("invalid slice");
    assert_eq!(err.code, "invalid-slice");
}

#[test]
fn submission_parser_validates_slice_coordinates() {
    let err = parse_submission("1\n0 0 0 x\n").expect_err("invalid coordinate");
    assert_eq!(err.code, "invalid-slice-coordinate");
}

#[test]
fn submission_parser_detects_mismatched_count() {
    let err = parse_submission("2\n0 0 0 0\n").expect_err("should fail");
    assert_eq!(err.code, "slice-count-mismatch");
}

#[test]
fn scoring_rejects_out_of_bounds_slice() {
    let parsed_input = example_input();
    let submission = parse_submission_str("1\n0 0 3 3\n");
    let err = score(&parsed_input, &submission).expect_err("should fail");
    assert_eq!(err.code, "slice-out-of-bounds");
}

#[test]
fn scoring_rejects_inverted_slice() {
    let parsed_input = example_input();
    let submission = ProblemSubmission {
        slices: vec![Slice {
            start_row: 2,
            start_col: 2,
            end_row: 1,
            end_col: 1,
        }],
    };
    let err = score(&parsed_input, &submission).expect_err("should fail");
    assert_eq!(err.code, "invalid-slice-orientation");
}

#[test]
fn scoring_rejects_slice_too_large() {
    let parsed_input = example_input();
    let submission = parse_submission_str("1\n0 0 2 4\n");
    let err = score(&parsed_input, &submission).expect_err("should fail");
    assert_eq!(err.code, "slice-too-large");
}

#[test]
fn scoring_rejects_not_enough_ingredients() {
    let parsed_input = example_input();
    // Two tomato-only cells from the top row.
    let submission = parse_submission_str("1\n0 0 0 1\n");
    let err = score(&parsed_input, &submission).expect_err("should fail");
    assert_eq!(err.code, "slice-not-enough-ingredients");
}

#[test]
fn scoring_rejects_overlapping_slices() {
    let parsed_input = example_input();
    let overlapping = "2\n0 0 1 1\n0 1 1 2\n";
    let submission = parse_submission(overlapping).expect("parse submission");
    let err = score(&parsed_input, &submission).expect_err("should fail");
    assert_eq!(err.code, "overlapping-slices");
}

#[test]
fn scoring_accepts_valid_slice_set() {
    let parsed_input = example_input();
    let submission = parse_submission_str("1\n1 1 2 3\n");
    let breakdown: ScoreBreakdown = score(&parsed_input, &submission).expect("should score");
    assert_eq!(breakdown.total_score, 6);
    assert_eq!(breakdown.covered_cells, 6);
    assert_eq!(breakdown.slice_count, 1);
}
