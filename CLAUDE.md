# CLAUDE.md

Keep this file clear and short: it holds only what the code, `Cargo.toml`, `CONTRIBUTING.md` and Linear can't tell you.

When a change makes it wrong or incomplete, update it in the same change, without asking first.

## Important guideline

- Write clear, concise, secure, and optimized code, without redundancies.
- Write tests in the `tests/` folder (one file per module, e.g. `tests/error.rs`), not in inline `#[cfg(test)]` modules.
- Log errors and warnings by reporting an `AppError` (add a variant if none fits): only `AppError::report` calls `log::error!` and `log::warn!`.
- No `Co-authored-by` on GitHub.

## Workflow

- Branch name: the issue title in kebab-case. Never include the issue key or a user prefix.
- Commit messages: a plain summary. Never include the issue key.
- Never commit without the developer's approval.
- Never push: the developer pushes.
- Never create a pull request: the user opens PRs manually on GitHub.
- Checks, lint policy, code, and documentation conventions: `CONTRIBUTING.md`.

## Gotchas

- egui/eframe 0.36 differs from what you remember (many APIs changed after 0.31): `App::ui` is required (there is no `update`), `egui::Panel` replaces `SidePanel`/`TopBottomPanel`, `Area::new` takes an `Id`. Read the crate source in the cargo registry instead of relying on memory.
- Consolas is proprietary: load it from the system, never commit or embed it.
- `nucleo-matcher` is MPL-2.0, so palette search is in-house.
- eframe's `persistence` feature stays off (VestaDesk has its own storage). `egui_kittest` takes only `features = ["eframe"]`, because `snapshot` needs a GPU.
