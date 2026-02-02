# Task 001: Establish Working Build System

## Description
Ensure the Space Crystals RTS project can build successfully and is ready for development. Currently, the project fails to compile due to a linker error when building with Bevy's dynamic linking feature (default configuration). The error occurs during compilation of `bevy_dylib` with a bus error signal.

The project needs to either:
1. Fix the dynamic linking compilation issue, or
2. Disable dynamic linking and use static compilation instead, or
3. Clean the build cache and retry if it's a temporary/corruption issue

## Why Needed
A working build system is the foundation for all development work. Without the ability to compile the project, no features can be implemented, tested, or validated. This must be resolved before any other tasks can begin.

## Acceptance Criteria
- [ ] Project compiles successfully with `cargo build`
- [ ] Project runs without crashing with `cargo run`
- [ ] Build configuration is documented (whether using dynamic or static linking)
- [ ] Build time is reasonable for development iteration
- [ ] Any workarounds or configuration changes are documented in the codebase

## Relevant Files/Components
- `Cargo.toml` - Build configuration and feature flags
- `src/main.rs` - Main entry point (currently has basic Bevy setup with camera)
- `target/` directory - May need cleaning if build cache is corrupted

## Technical Considerations
**Current Issue**: Linker bus error when compiling `bevy_dylib`
```
error: linking with `cc` failed: exit status: 1
collect2: fatal error: ld terminated with signal 7 [Bus error]
```

**Possible Solutions**:
1. **Clean build cache**: Try `cargo clean && cargo build` to eliminate corruption
2. **Disable dynamic linking**: Remove or disable the `dynamic_linking` feature in Cargo.toml
3. **Increase system resources**: Bus errors can indicate memory issues during linking
4. **Update dependencies**: Try `cargo update` in case there's a known issue fixed in newer versions

**Trade-offs**:
- Dynamic linking: Faster incremental compile times, but larger binary and potential stability issues
- Static linking: Slower compile times, but more stable and smaller final binary

## Prerequisites
None - This is the foundational task that enables all other development work.

## Complexity
Simple

## Notes
The current `Cargo.toml` has dynamic linking enabled by default via the features section. The main.rs file has basic Bevy setup with a 3D camera and movement controls, so once building works, the application should display a window and respond to input.
