## What this PR does

<!-- One sentence. Link the issue: "Closes #N." -->

## Changes

<!-- Bullet list of files/functions changed and why. -->

## Engine / shell boundary check

<!-- Every PR must answer this. -->
- [ ] No JSON parsing, formatting, or diagnostics logic was added to the macOS shell
- [ ] All new logic that must survive a shell swap lives in `engine/`

## Definition of done checklist

<!-- Copy from the issue, or fill in if this is a standalone fix. -->
- [ ] `cargo test` passes
- [ ] `cargo clippy -- -D warnings` is clean
- [ ] `cargo fmt --check` passes

## Verification

<!-- How did you confirm this works beyond tests? -->

## Notes

<!-- Anything the reviewer should know: risks, follow-ups, ADR implications. -->
