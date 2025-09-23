use crate::hashcodes::error::ProblemError;

use super::input::ProblemInput;
use super::submission::ProblemSubmission;
use super::{Ingredient, Slice};

/// Captures the totals reported back through the API.
#[derive(Clone, Debug)]
pub struct ScoreBreakdown {
    pub total_score: usize,
    pub slice_count: usize,
    pub covered_cells: usize,
}

impl ScoreBreakdown {
    pub fn into_json(self) -> serde_json::Value {
        serde_json::json!({
            "total_score": self.total_score,
            "slice_count": self.slice_count,
            "covered_cells": self.covered_cells,
        })
    }
}

/// Validates a submission exactly as described in the PDF:
/// - every slice must stay within the pizza grid
/// - the number of cells per slice cannot exceed `L` (called `max_cells` here)
/// - each slice has to contain at least `min_ingredient` of both tomatoes and mushrooms
/// - slices must not overlap
/// The problem's score equals the sum of the slice areas, which in turn matches
/// the number of covered cells when every rule is satisfied.
pub fn score(
    input: &ProblemInput,
    submission: &ProblemSubmission,
) -> Result<ScoreBreakdown, ProblemError> {
    let mut occupied = vec![vec![false; input.cols]; input.rows];
    let mut total_score = 0usize;
    let mut covered_cells = 0usize;

    for (index, slice) in submission.slices.iter().enumerate() {
        validate_slice_bounds(input, slice, index)?;
        validate_slice_area(input, slice, index)?;
        validate_slice_ingredients(input, slice, index)?;
        let cells = iter_slice_cells(slice);
        for (row, col) in cells {
            if occupied[row][col] {
                return Err(ProblemError::with_details(
                    "overlapping-slices",
                    format!("Slice {index} overlaps with another slice"),
                    serde_json::json!({ "slice": index, "row": row, "col": col }),
                ));
            }
            occupied[row][col] = true;
            covered_cells += 1;
        }
        total_score += slice.cell_count();
    }

    Ok(ScoreBreakdown {
        total_score,
        slice_count: submission.slices.len(),
        covered_cells,
    })
}

fn validate_slice_bounds(
    input: &ProblemInput,
    slice: &Slice,
    index: usize,
) -> Result<(), ProblemError> {
    if slice.start_row > slice.end_row || slice.start_col > slice.end_col {
        return Err(ProblemError::with_details(
            "invalid-slice-orientation",
            format!("Slice {index} has inverted coordinates"),
            serde_json::json!({
                "slice": index,
                "start_row": slice.start_row,
                "start_col": slice.start_col,
                "end_row": slice.end_row,
                "end_col": slice.end_col,
            }),
        ));
    }
    if slice.end_row >= input.rows || slice.end_col >= input.cols {
        return Err(ProblemError::with_details(
            "slice-out-of-bounds",
            format!("Slice {index} extends outside the pizza"),
            serde_json::json!({
                "slice": index,
                "max_row": input.rows - 1,
                "max_col": input.cols - 1,
                "end_row": slice.end_row,
                "end_col": slice.end_col,
            }),
        ));
    }
    Ok(())
}

fn validate_slice_area(
    input: &ProblemInput,
    slice: &Slice,
    index: usize,
) -> Result<(), ProblemError> {
    let cells = slice.cell_count();
    if cells > input.max_cells {
        return Err(ProblemError::with_details(
            "slice-too-large",
            format!(
                "Slice {index} uses {cells} cells which exceeds the maximum of {max}",
                max = input.max_cells
            ),
            serde_json::json!({
                "slice": index,
                "cell_count": cells,
                "max_cells": input.max_cells,
            }),
        ));
    }
    Ok(())
}

fn validate_slice_ingredients(
    input: &ProblemInput,
    slice: &Slice,
    index: usize,
) -> Result<(), ProblemError> {
    let mut tomato_count = 0usize;
    let mut mushroom_count = 0usize;
    for (row, col) in iter_slice_cells(slice) {
        match input.grid[row][col] {
            Ingredient::Tomato => tomato_count += 1,
            Ingredient::Mushroom => mushroom_count += 1,
        }
    }
    if tomato_count < input.min_ingredient || mushroom_count < input.min_ingredient {
        return Err(ProblemError::with_details(
            "slice-not-enough-ingredients",
            format!(
                "Slice {index} must contain at least {required} of each ingredient but has {tomatoes} tomatoes and {mushrooms} mushrooms",
                required = input.min_ingredient,
                tomatoes = tomato_count,
                mushrooms = mushroom_count
            ),
            serde_json::json!({
                "slice": index,
                "required_per_ingredient": input.min_ingredient,
                "tomatoes": tomato_count,
                "mushrooms": mushroom_count,
            }),
        ));
    }
    Ok(())
}

fn iter_slice_cells(slice: &Slice) -> impl Iterator<Item = (usize, usize)> + '_ {
    (slice.start_row..=slice.end_row)
        .flat_map(move |row| (slice.start_col..=slice.end_col).map(move |col| (row, col)))
}
