# Product Analyst Memory

## Project Structure
- 10 feature files in `/features/` covering all formal design content as of 2026-03-06
- 7 formal design files in `/design/` plus `to_be_converted.md`, `designer_notes.md`, `design_questions.md`
- `designer_notes.md` has 9 sessions of design history - essential for understanding intent and open questions

## Key Patterns
- Design update files are single-write. Feature update files are single-write.
- Feature files are organized by system, not by design file. One design file may map to multiple features.
- Only GDO faction objects are fully specified. Other 3 factions have only resource definitions formalized.

## Design Conventions
- Production costs belong to the producer (Barracks defines Peacekeeper cost, not Peacekeeper itself)
- Rally points use same resolution as right-click
- Commands only set BaseCommandState; TurretCommandState is managed by behaviors
- Gliders are special-cased in almost every behavior (can never stop)
