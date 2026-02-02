# Space Crystals RTS

A real-time strategy game built with Bevy 0.14.

# Project Overview

Space Crystals is an RTS game currently in early development. The project uses the Bevy game engine and is written in Rust.

# Agents

New agent types will have their .md files in the ./agents directory.

They will store their history in the ./agent_logs directory.

This is the main agent which will be responsible only for maintaining the other agents' .md files with instructions.

## Available Agent Types

### Technical Project Manager
**File**: `agents/technical_project_manager.md`

Breaks down features and objectives into actionable tasks. Helps with:
- Task generation and decomposition
- Dependency mapping
- Scope definition
- Risk identification
- Work organization

Use this agent when you need to plan implementation of new features or organize development work.

### Developer
**File**: `agents/developer.md`

Implements tasks from the project_tasks directory. Handles:
- Task selection based on prerequisites
- Code implementation following Bevy ECS patterns
- Testing and validation
- Task completion tracking
- Blocker resolution

Use this agent to execute tasks created by the Technical Project Manager. The agent will automatically select appropriate tasks and implement them according to specifications.

### Product Analyst
**File**: `agents/product_analyst.md`

Monitors project health and provides insights into project state. Handles:
- Project status analysis and reporting
- Blocker identification and tracking
- Answering questions about current project state
- Storing findings in ./analyst directory for future reference
- Trend analysis and progress tracking

Use this agent when you need to understand project health, identify impediments, or get status updates. The agent maintains persistent knowledge in the ./analyst directory.

# Technical Stack

- **Language**: Rust (Edition 2021)
- **Game Engine**: Bevy 0.14
- **Build System**: Cargo

