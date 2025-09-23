use super::input::ProblemInput;
use super::submission::ProblemSubmission;
use super::{Ingredient, Slice};

/// Produces a quick feasible solution by scanning the pizza left-to-right and
/// creating 2-cell slices as suggested in the PDF's introductory example. The
/// practice statement emphasises every slice must contain at least the minimum
/// number of each ingredient and no more than the maximum number of cells.
/// We greedily attempt horizontal pairs first (to align with the example),
/// then vertical pairs, marking any covered cells as unavailable for future
/// slices. This deterministic strategy favours readability for regression
/// tests over optimality.
pub fn solve(input: &ProblemInput) -> ProblemSubmission {
    let mut used = vec![vec![false; input.cols]; input.rows];
    let mut slices = Vec::new();

    for r in 0..input.rows {
        for c in 0..input.cols {
            if used[r][c] {
                continue;
            }

            // Try to extend horizontally first, matching the worked example in
            // the PDF where adjacent cells form a valid slice when they respect
            // both the ingredient minimum and the maximum cell constraint.
            if c + 1 < input.cols && !used[r][c + 1] {
                let cells = [input.grid[r][c], input.grid[r][c + 1]];
                if is_valid_slice(&cells, input.min_ingredient, input.max_cells) {
                    used[r][c] = true;
                    used[r][c + 1] = true;
                    slices.push(Slice {
                        start_row: r,
                        start_col: c,
                        end_row: r,
                        end_col: c + 1,
                    });
                    continue;
                }
            }

            // If a horizontal pair fails the PDF constraints, try a vertical
            // pair before moving on. This keeps the solver simple while still
            // demonstrating how to respect the same validation rules enforced
            // by the scorer.
            if r + 1 < input.rows && !used[r + 1][c] {
                let cells = [input.grid[r][c], input.grid[r + 1][c]];
                if is_valid_slice(&cells, input.min_ingredient, input.max_cells) {
                    used[r][c] = true;
                    used[r + 1][c] = true;
                    slices.push(Slice {
                        start_row: r,
                        start_col: c,
                        end_row: r + 1,
                        end_col: c,
                    });
                }
            }
        }
    }

    ProblemSubmission { slices }
}

fn is_valid_slice(cells: &[Ingredient], min_ingredient: usize, max_cells: usize) -> bool {
    if cells.len() > max_cells {
        return false;
    }
    let tomato_count = cells.iter().filter(|&&i| i == Ingredient::Tomato).count();
    let mushroom_count = cells.iter().filter(|&&i| i == Ingredient::Mushroom).count();
    tomato_count >= min_ingredient && mushroom_count >= min_ingredient
}
