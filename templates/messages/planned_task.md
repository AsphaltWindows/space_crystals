# planned_task

Content guidance for the `planned_task` message type.

The content passed to `scripts/send_message.sh` should include:

## Parent Feature

{filename of the parent feature_request, carried over from the developer_task}

## Task

{Original task description from the developer_task message.}

## Technical Context

{Specific files that need to change (with paths), existing patterns to follow,
relevant types/traits/components/resources, integration points, Bevy ECS
considerations — system ordering, queries, events, plugins.}

## Dependencies

{List of other planned_tasks or existing systems this task depends on.
Explain why each dependency exists. "None" if standalone.}
