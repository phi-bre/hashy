use crate::hashcodes::error::ProblemError;

use super::Ingredient;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProblemInput {
    pub rows: usize,
    pub cols: usize,
    pub min_ingredient: usize,
    pub max_cells: usize,
    pub grid: Vec<Vec<Ingredient>>,
}

pub fn load_input(input_file: &str) -> Result<&'static str, ProblemError> {
    let normalized = input_file.trim().trim_end_matches(".in");
    match normalized {
        "a_example" | "example" => Ok(include_str!("hashcode_2017_practice_round.in/example.in")),
        "b_small" | "small" => Ok(include_str!("hashcode_2017_practice_round.in/small.in")),
        "c_medium" | "medium" => Ok(include_str!("hashcode_2017_practice_round.in/medium.in")),
        "d_big" | "big" => Ok(include_str!("hashcode_2017_practice_round.in/big.in")),
        other => Err(ProblemError::with_details(
            "unknown-input-file",
            format!("Unsupported 2017 practice dataset '{other}'"),
            serde_json::json!({
                "requested": input_file,
                "available": ["a_example", "b_small", "c_medium", "d_big"],
            }),
        )),
    }
}

pub fn parse_input(raw: &str) -> Result<ProblemInput, ProblemError> {
    let mut lines = raw.lines();
    let header = lines
        .next()
        .ok_or_else(|| ProblemError::new("missing-header", "Input file is empty"))?;
    let mut header_parts = header.split_whitespace();
    let rows = parse_usize(header_parts.next(), "rows")?;
    let cols = parse_usize(header_parts.next(), "columns")?;
    let min_ingredient = parse_usize(header_parts.next(), "min_ingredient")?;
    let max_cells = parse_usize(header_parts.next(), "max_cells")?;

    if header_parts.next().is_some() {
        return Err(ProblemError::new(
            "invalid-header",
            "Header contains extra unexpected values",
        ));
    }

    let mut grid = Vec::with_capacity(rows);
    for (row_index, line) in lines.enumerate() {
        if row_index >= rows {
            return Err(ProblemError::with_details(
                "too-many-rows",
                format!("Input declares {rows} rows but contains more data lines"),
                serde_json::json!({ "expected_rows": rows, "actual_rows": row_index + 1 }),
            ));
        }
        let chars: Result<Vec<Ingredient>, ProblemError> =
            line.chars().map(Ingredient::from_char).collect();
        let row = chars?;
        if row.len() != cols {
            return Err(ProblemError::with_details(
                "invalid-row-length",
                format!(
                    "Row {row_index} has {len} columns but expected {cols}",
                    len = row.len()
                ),
                serde_json::json!({
                    "row": row_index,
                    "expected_columns": cols,
                    "actual_columns": row.len()
                }),
            ));
        }
        grid.push(row);
    }

    if grid.len() != rows {
        return Err(ProblemError::with_details(
            "not-enough-rows",
            format!(
                "Input declares {rows} rows but only {actual} were provided",
                actual = grid.len()
            ),
            serde_json::json!({ "expected_rows": rows, "actual_rows": grid.len() }),
        ));
    }

    Ok(ProblemInput {
        rows,
        cols,
        min_ingredient,
        max_cells,
        grid,
    })
}

fn parse_usize(value: Option<&str>, field: &str) -> Result<usize, ProblemError> {
    let raw = value.ok_or_else(|| {
        ProblemError::with_details(
            "missing-value",
            format!("Header is missing the '{field}' entry"),
            serde_json::json!({ "field": field }),
        )
    })?;
    raw.parse::<usize>().map_err(|_err| {
        ProblemError::with_details(
            "invalid-number",
            format!("Could not parse '{field}' as a positive integer"),
            serde_json::json!({ "field": field, "value": raw }),
        )
    })
}
