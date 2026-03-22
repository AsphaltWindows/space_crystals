# Operator Insights

- Legacy tickets live in `qa_human_review/` and `qa_tasks/` at the project root (not inside `artifacts/`). These are from a pre-pipeline era and contain detailed technical context, QA results, and dependency information.
- When grouping tickets thematically, the Syndicate faction has two natural clusters: Agent-centric (commands, gathering, building) and Tunnel/Underground-centric (tunnel interface, HQ production, expansions, enter/exit). These overlap but are distinct enough to split.
- Many tickets have extensive "Technical Context" sections with file paths and line numbers — these are useful for the developer but the designer mainly needs the behavioral/UX descriptions and QA steps.
- `artifacts/qa_router/auto_capabilities.txt` controls routing: grep patterns that match QA instruction lines. All patterns commented out = everything goes to manual_qa. 37 items are stuck in manual_qa pending as of 2026-03-21.
- QA instruction patterns in manual_qa items fall into: stat/component checks, state machine transitions, constraint enforcement, ECS queries (all potentially automatable), and visual/interactive checks (require human). Most items mix both types.
- The architect agent does NOT participate in the forum. It's a framework/infrastructure role, not a scheduled pipeline agent. Don't direct forum work to the architect — redirect to developer or other pipeline agents instead.
