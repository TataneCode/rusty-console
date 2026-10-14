# Architecture log - 26/04/2026

## Summary

This log tracks the structural refactor from the original feature-first layout to the current numbered layer-first architecture.

## Final structure

- `src/01_domain`
- `src/02_application`
- `src/03_infrastructure`
- `src/04_presentation`

## Structural changes completed

1. Added the numbered layer directories and made them the primary import surface.
2. Moved the shared Docker client to `src/03_infrastructure/docker/client.rs`.
3. Moved layered error definitions to:
   - `src/01_domain/error.rs`
   - `src/02_application/error.rs`
   - `src/03_infrastructure/error.rs`
4. Decoupled the `stack` feature from `container` internals by introducing stack-local models and DTOs.
5. Moved the business slice implementations into the numbered layers:
   - domain code under `01_domain/*`
   - application code under `02_application/*`
   - Docker/infrastructure code under `03_infrastructure/docker/*`
   - TUI code under `04_presentation/tui/*`
6. Deleted the old source trees:
   - `src/container/`
   - `src/image/`
   - `src/volume/`
   - `src/stack/`
   - `src/docker/`
   - `src/errors/`
   - `src/ui/`
7. Replaced the deleted trees with temporary facade files:
   - `src/container.rs`
   - `src/image.rs`
   - `src/volume.rs`
   - `src/stack.rs`
   - `src/docker.rs`
   - `src/errors.rs`
   - `src/ui.rs`
8. Removed the temporary facade files after all imports were updated to use the numbered layer-first modules directly.

## Checkpoint commits

1. `8d43eca` - scaffold numbered layer architecture
2. `c4a377d` - decouple stack boundary
3. `b611f29` - move infrastructure and errors
4. `1a78ce3` - migrate container slice
5. `907ebf2` - migrate remaining slices
6. `23fe09f` - finalize domain centric architecture
7. `996507f` - remove legacy source trees

## Current state

- The numbered layers are the real implementation location.
- The old directory trees are gone.
- All imports now point directly to the numbered layer-first modules.
