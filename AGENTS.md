# Repository Guidance for Hash Code API Implementations

This project exposes Google Hash Code scoring logic through the Vercel Rust runtime. All contributions must follow the structure
below to keep every year/round implementation aligned.

## Global conventions

- The Rust API entry point lives in `/api/handler.rs`. Add any additional helper modules under `src/` – do **not** add extra binaries.
- The API path structure is `/api/hashcodes/{year}/{round}/{input_file}` and only accepts `POST` submissions whose body is the exact text of the participant submission.
- Every API response is JSON encoded. On success it must include at minimum the dataset identifier and the computed score. Validation failures must return detailed machine-readable errors (`code`, `message`, and optional `details`).
- Prefer small, problem-specific enums/structs over stringly-typed code paths when routing requests.
- Keep everything `#![forbid(unsafe_code)]` friendly – we never need `unsafe`.

## Hash Code problem modules (shared between the API and the Astro content)

- Register problem façade modules directly from `src/hashcodes/mod.rs` using `#[path = "../content/hashcodes/{year}/{round}/{pdf_basename}.rs"]`. Avoid introducing extra year wrapper modules so the API routing stays flat and easy to extend.
- Every PDF must be accompanied by Rust sources in the **same folder** following this naming template:
  - `<pdf_basename>.rs` – the façade module that re-exports the problem pieces and exposes `score_submission`.
  - `<pdf_basename>_input.rs` – exposes `load_input(input_file: &str)` and `parse_input(raw: &str)`.
  - `<pdf_basename>_submission.rs` – exposes `parse_submission(raw: &str)`.
  - `<pdf_basename>_solver.rs` – exposes a deterministic `solve(input: &ProblemInput) -> ProblemSubmission` used by tests/examples.
  - `<pdf_basename>_scorer.rs` – exposes `score(input: &ProblemInput, submission: &ProblemSubmission) -> Result<ScoreBreakdown, ProblemError>`.
  - `<pdf_basename>_tests.rs` – integration-style unit tests that assert the parser/solver/scorer work together using the real datasets shipped alongside the crate.
- The façade module should load the child files with `#[path = "<file>.rs"]` so the Rust code stays colocated with the PDF. Export the shared types (`ProblemInput`, `ProblemSubmission`, helper enums) from the façade for reuse.
- Re-use the existing input dataset files that ship with the Astro content (for example the `.in` directories already in `src/content/hashcodes/...`). Do **not** duplicate those files into new folders; instead, point `include_str!` at the originals and, when helpful, accept both the official Google filename (e.g. `a_example`) and the local alias (`example`).
- Re-use the shared error/response helpers from `src/hashcodes` instead of inventing new JSON structures.
- Provide comprehensive validation errors that mirror the official statement rules and attach actionable details (e.g. “slice 3 overlaps slice 1”).
- The solver and scorer implementations must include comments summarising the logic in terms of the PDF rules so future contributors can quickly compare code to statement requirements.

## Testing expectations

- Run `cargo fmt` and `cargo test` after every change that touches Rust code.
- Tests within `<pdf_basename>_tests.rs` must cover every validation error exposed by that problem’s parsers and scorer (at minimum all error conditions described in the PDF).
- Manual API checks must be performed with `curl` (or similar) hitting the local dev server started by `vercel dev` or `cargo run` depending on context, and the command/output captured in the PR description when applicable.

## API handler specifics (`/api/handler.rs`)

- The handler must route requests purely by URL segments and invoke the appropriate scorer. Keep routing logic data-driven through enums/lookup tables instead of `if` chains where possible.
- Only `POST` is allowed for scoring endpoints. Return HTTP `405` for other methods and `404` for unknown datasets/problems.
- Always set the `Content-Type: application/json` response header.

Following these conventions will keep new Hash Code scoring modules consistent and immediately consumable by the API.
