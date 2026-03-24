"""Agent discovery from pipeline.yaml and hardcoded agents.

Reads the pipeline configuration and extracts agents with ``type: source``,
merges them with hardcoded agents (e.g. the operator agent), and provides
a unified list of available agents for Telegram command registration.
"""

from __future__ import annotations

import logging
from pathlib import Path
from typing import List, Optional, Union

import yaml


logger = logging.getLogger(__name__)

# Project root — inherited from the process working directory.
# The launcher script (run_bot.sh) is expected to cd to the project root
# before starting the bot, making Path.cwd() the correct value.
_PROJECT_ROOT = Path.cwd()
_DEFAULT_PIPELINE_PATH = _PROJECT_ROOT / "pipeline.yaml"

# Agents that are always available, independent of pipeline.yaml.
# The operator agent is a Claude agent defined at .claude/agents/operator.md
# and exists outside the pipeline framework.
_HARDCODED_AGENTS: List[str] = ["operator"]


def discover_source_agents(
    pipeline_path: Optional[Union[Path, str]] = None,
) -> List[str]:
    """Discover source agents from the pipeline configuration.

    Reads ``pipeline.yaml`` and returns the names of all agents whose
    ``type`` is ``"source"``.  Nodes with ``scheduled: false`` are still
    included — the ``scheduled`` flag does not affect discovery.

    Parameters
    ----------
    pipeline_path:
        Optional path to the pipeline YAML file.  Defaults to
        ``pipeline.yaml`` in the project root.

    Returns
    -------
    list[str]
        Agent names where ``type == "source"``.  May be empty if no
        source agents are defined.

    Raises
    ------
    FileNotFoundError
        If the pipeline file does not exist at the resolved path.
    ValueError
        If the file cannot be parsed as YAML or has an unexpected
        structure (e.g. missing ``nodes`` key, or ``nodes`` is not a list).
    """
    path = Path(pipeline_path) if pipeline_path is not None else _DEFAULT_PIPELINE_PATH

    if not path.exists():
        raise FileNotFoundError(
            f"Pipeline configuration file not found: {path}"
        )

    with open(path, "r", encoding="utf-8") as fh:
        try:
            data = yaml.safe_load(fh)
        except yaml.YAMLError as exc:
            raise ValueError(
                f"Failed to parse pipeline YAML at {path}: {exc}"
            ) from exc

    if data is None or not isinstance(data, dict):
        raise ValueError(
            f"Pipeline file at {path} is empty or not a YAML mapping."
        )

    if "nodes" not in data:
        raise ValueError(
            f"Pipeline file at {path} is missing the required 'nodes' key."
        )

    nodes = data["nodes"]
    if not isinstance(nodes, list):
        raise ValueError(
            f"'nodes' in {path} must be a list, got {type(nodes).__name__}."
        )

    return [
        node["name"]
        for node in nodes
        if isinstance(node, dict) and node.get("type") == "source"
    ]


def get_all_agents(
    pipeline_path: Optional[Union[Path, str]] = None,
    hardcoded_agents: Optional[List[str]] = None,
) -> List[str]:
    """Return the combined set of pipeline-discovered and hardcoded agents.

    Hardcoded agents take precedence: if a name appears in both the
    hardcoded list and the pipeline-discovered list, only the hardcoded
    entry is kept (no duplicates).

    Parameters
    ----------
    pipeline_path:
        Optional path to the pipeline YAML file.  Passed through to
        :func:`discover_source_agents`.
    hardcoded_agents:
        Optional override for the hardcoded agent list.  Defaults to
        the module-level ``_HARDCODED_AGENTS`` constant.

    Returns
    -------
    list[str]
        Deduplicated list of agent names — hardcoded agents first,
        followed by pipeline agents not already present.
    """
    if hardcoded_agents is None:
        hardcoded_agents = list(_HARDCODED_AGENTS)

    pipeline_agents = discover_source_agents(pipeline_path=pipeline_path)

    # Build combined list: hardcoded first, then pipeline agents not in hardcoded.
    seen: set[str] = set(hardcoded_agents)
    combined = list(hardcoded_agents)
    for agent in pipeline_agents:
        if agent not in seen:
            seen.add(agent)
            combined.append(agent)

    logger.info(
        "Discovered pipeline agents: [%s]. Hardcoded agents: [%s]. Combined: [%s]",
        ", ".join(pipeline_agents) if pipeline_agents else "",
        ", ".join(hardcoded_agents) if hardcoded_agents else "",
        ", ".join(combined) if combined else "",
    )

    return combined
