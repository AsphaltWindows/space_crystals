# task_completion

Content guidance for the `task_completion` message type.

Minimal marker message. The content passed to `scripts/send_message.sh` can be
a simple string like "Task complete." — the completion_aggregator only checks
whether the file exists, it does not read the content.

The message_name (slug) must match the corresponding developer_task slug so the
completion_aggregator can correlate them.
