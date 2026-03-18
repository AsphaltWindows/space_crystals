# feature_tasks

Content guidance for the `feature_tasks` message type.

The content passed to `scripts/send_message.sh` should include:

## Feature Request

{exact filename of the corresponding feature_request message in
completion_aggregator's inbox, e.g., task_splitter-{feature_slug}.md}

## Developer Tasks

- {filename_1.md}
- {filename_2.md}
- {filename_3.md}

{Bulleted list of exact filenames as produced by send_message.sh
(format: task_splitter-{task_slug}.md). The completion_aggregator
matches these literally against task_completion filenames.}
