use crate::hashcodes::error::ProblemError;

use super::Slice;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProblemSubmission {
    pub slices: Vec<Slice>,
}

pub fn parse_submission(raw: &str) -> Result<ProblemSubmission, ProblemError> {
    let mut lines = raw.lines();
    let header = lines.next().ok_or_else(|| {
        ProblemError::new(
            "missing-slice-count",
            "Submission must start with the number of slices",
        )
    })?;

    let declared_count = header.trim().parse::<usize>().map_err(|_err| {
        ProblemError::with_details(
            "invalid-slice-count",
            "First line must be a non-negative integer",
            serde_json::json!({ "line": 0, "value": header.trim() }),
        )
    })?;

    let mut slices = Vec::with_capacity(declared_count);

    for (index, line) in lines.enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let coords: Vec<&str> = line.split_whitespace().collect();
        if coords.len() != 4 {
            return Err(ProblemError::with_details(
                "invalid-slice",
                format!(
                    "Slice line {line_no} does not contain four integers",
                    line_no = index + 1
                ),
                serde_json::json!({ "line": index + 1, "content": line }),
            ));
        }
        let numbers: Result<Vec<usize>, ProblemError> = coords
            .iter()
            .enumerate()
            .map(|(pos, value)| {
                value.parse::<usize>().map_err(|_err| {
                    ProblemError::with_details(
                        "invalid-slice-coordinate",
                        format!(
                            "Coordinate {coord} on line {line_no} is not a non-negative integer",
                            coord = pos,
                            line_no = index + 1
                        ),
                        serde_json::json!({
                            "line": index + 1,
                            "position": pos,
                            "value": value
                        }),
                    )
                })
            })
            .collect();
        let values = numbers?;
        let slice = Slice {
            start_row: values[0],
            start_col: values[1],
            end_row: values[2],
            end_col: values[3],
        };
        slices.push(slice);
    }

    if slices.len() != declared_count {
        return Err(ProblemError::with_details(
            "slice-count-mismatch",
            format!(
                "Submission declares {declared_count} slices but provided {actual}",
                actual = slices.len()
            ),
            serde_json::json!({
                "declared": declared_count,
                "actual": slices.len(),
            }),
        ));
    }

    Ok(ProblemSubmission { slices })
}
