# Architecture Request Template

## Metadata
- **From**: {producing-agent}
- **To**: task_splitter

## Content

## Architecture Document

{path to the architecture doc, e.g., artifacts/software_architect/architecture/combat-system-refactor.md}

## Summary

{Brief summary of what the refactoring achieves and why it's needed. Reference recurring issues or forum discussions as evidence.}

## Scope

{What parts of the codebase are affected — specific modules, systems, components}

## QA Instructions

{How to verify the refactoring was successful:
- Compilation checks
- Behavioral equivalence requirements
- Any specific scenarios to validate}

## Testing Requirements

{Specific automated tests that must be written as part of this refactor:
- Unit tests for new or changed modules (what to test, expected behaviors)
- Integration tests if system interactions changed
- Regression tests to ensure existing behavior is preserved
- Property-based tests where applicable
- Test fixtures or helpers that should be created
Be concrete — name the modules/systems under test and the scenarios to cover.}
