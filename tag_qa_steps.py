#!/usr/bin/env python3
"""Tag QA steps with [auto], [human], or [semi] prefixes."""
import re
import sys
import os

# Keywords/patterns that strongly indicate [human] (UI/visual)
HUMAN_PATTERNS = [
    r'verify.*(?:button|panel|hud|menu|ui|label|text|title).*(?:show|display|appear|read|visible|styled|greyed|grayed|active|highlighted|dim)',
    r'verify.*(?:ghost|preview|placement ghost|green|red|tint|color|glow)',
    r'verify.*(?:smooth|animation|visual|render|glitch|flicker|artifact)',
    r'verify.*(?:camera|viewport|screen)',
    r'(?:command panel|hud).*(?:show|display|appear|update)',
    r'(?:button|hotkey label).*(?:appear|show|display|read)',
    r'verify.*(?:overlay|f3|diagnostic)',
    r'(?:drag|box.?select).*(?:appear|visual|rectangle)',
    r'verify.*(?:indicator|marker).*(?:appear|visible|display|shown)',
    r'(?:selection (?:indicator|box|circle)).*(?:appear|visible)',
    r'verify.*(?:portrait|thumbnail)',
    r'verify.*(?:bar|meter|gauge).*(?:display|show|update|fill)',
    r'verify.*(?:tooltip)',
    r'press.*verify.*(?:interface|panel|menu).*(?:enter|show|display|switch|transition)',
    r'verify.*(?:grid.*slot|command.*grid)',
    r'verify.*(?:console|log|output|print)',
    r'verify.*(?:resource.*bar|bar.*resource)',
    r'verify.*(?:no.*visual|cleanly|disappear)',
]

# Patterns that indicate [semi] (setup automatable, visual verification)
SEMI_PATTERNS = [
    r'(?:spawn|place|build).*verify.*(?:appear|visible|see|mesh|model)',
    r'verify.*(?:indicator|marker).*(?:color|position)',
    r'(?:move|attack).*verify.*(?:indicator|marker)',
    r'verify.*(?:looks|appears).*(?:correct|proper)',
]

# Patterns that strongly indicate [auto] (state-based verification)
AUTO_PATTERNS = [
    r'verify.*(?:hp|health|damage|hit point)',
    r'verify.*(?:die|dead|destroy|killed|removed)',
    r'verify.*(?:arrives?|reach|position|moves? to|walks? to|returns? to)',
    r'verify.*(?:behavior|state|phase).*(?:change|transition|switch|enter|become)',
    r'verify.*(?:command.*(?:issue|queue|receive|reject|accept|execute))',
    r'verify.*(?:resource|crystal|supply|cap).*(?:amount|count|change|increase|decrease|deduct|reduce)',
    r'verify.*(?:fog|visibility|visible|revealed|hidden|shroud)',
    r'verify.*(?:selection.*(?:count|contain|include|group|size|cycle))',
    r'verify.*(?:construction|progress|complete|build)',
    r'verify.*(?:unit.*cap|population)',
    r'verify.*(?:spawn|produce|train|create).*(?:unit|entity)',
    r'verify.*(?:armor|silhouette|attack.*type|range)',
    r'verify.*(?:elevation|modifier|bonus)',
    r'verify.*(?:path|pathfind|route|diagonal|waypoint)',
    r'verify.*(?:collision|occupy|block|overlap)',
    r'verify.*(?:enter|exit|eject|tunnel)',
    r'verify.*(?:idle|stop|halt|leash)',
    r'verify.*(?:patrol|reverse|attack.?move|hold.*position)',
    r'verify.*(?:target|engage|fire|shoot|attack)',
    r'verify.*(?:power|grid|ratio)',
    r'verify.*(?:flip|rotate|orient)',
    r'verify.*(?:size|validate|invalid)',
    r'(?:cargo test|cargo build|compile)',
    r'verify.*(?:file|script|directory|exist)',
    r'verify.*(?:tag|prefix|\[auto\]|\[human\]|\[semi\])',
    r'verify.*(?:count|total|number|percentage)',
    r'(?:select|deselect).*verify',
    r'verify.*(?:queue|dequeue|clear)',
]


def classify_step(step_text):
    """Classify a QA step as auto, human, or semi."""
    text = step_text.lower().strip()

    # Already tagged
    if re.match(r'^\d+\.\s*\[(auto|human|semi)\]', text):
        return None  # already tagged

    # Check human first (UI/visual takes priority)
    for pattern in HUMAN_PATTERNS:
        if re.search(pattern, text, re.IGNORECASE):
            # But check if it's actually testable programmatically
            return 'human'

    # Check semi
    for pattern in SEMI_PATTERNS:
        if re.search(pattern, text, re.IGNORECASE):
            return 'semi'

    # Check auto
    for pattern in AUTO_PATTERNS:
        if re.search(pattern, text, re.IGNORECASE):
            return 'auto'

    # Default heuristics based on keywords
    human_words = ['visually', 'visual', 'ui', 'hud', 'button', 'panel', 'ghost',
                   'preview', 'smooth', 'animation', 'render', 'glitch', 'camera',
                   'overlay', 'diagnostic', 'display', 'portrait', 'thumbnail',
                   'label', 'text reads', 'shows the', 'appears in', 'shows up',
                   'console', 'log output', 'f3', 'tooltip', 'grid slot',
                   'greyed', 'grayed', 'dimmed', 'highlighted', 'styled',
                   'bar shows', 'bar displays', 'bar updates']

    auto_words = ['verify hp', 'verify health', 'dies', 'dead', 'killed',
                  'arrives', 'reaches', 'position', 'damage', 'destroy',
                  'resource', 'crystal', 'cap', 'behavior', 'state',
                  'command', 'fog', 'visible', 'selection',
                  'spawn', 'enter', 'exit', 'eject', 'tunnel',
                  'compile', 'build', 'test', 'cargo',
                  'collision', 'pathfind', 'path', 'target',
                  'engage', 'fire', 'attack', 'patrol', 'reverse',
                  'hold position', 'construction', 'progress',
                  'count', 'total', 'amount']

    for w in human_words:
        if w in text:
            return 'human'

    for w in auto_words:
        if w in text:
            return 'auto'

    # Conservative default
    return 'auto'


def process_file(filepath):
    """Process a single QA task file."""
    with open(filepath, 'r') as f:
        content = f.read()

    lines = content.split('\n')
    in_qa_section = False
    modified = False
    result_lines = []

    for line in lines:
        if line.strip() == '## QA Steps':
            in_qa_section = True
            result_lines.append(line)
            continue

        if in_qa_section and line.startswith('## '):
            in_qa_section = False

        if in_qa_section and re.match(r'^(\d+)\.\s+', line):
            # Check if already tagged
            if re.match(r'^(\d+)\.\s+\[(auto|human|semi)\]', line):
                result_lines.append(line)
                continue

            tag = classify_step(line)
            if tag:
                # Insert tag after "N. "
                new_line = re.sub(r'^(\d+\.\s+)', rf'\1[{tag}] ', line)
                result_lines.append(new_line)
                modified = True
                continue

        result_lines.append(line)

    if modified:
        with open(filepath, 'w') as f:
            f.write('\n'.join(result_lines))
        return True
    return False


def main():
    qa_dir = '/home/iv/dev/space_crystals/qa_tasks'
    files = sorted(f for f in os.listdir(qa_dir) if f.endswith('.md'))

    for f in files:
        filepath = os.path.join(qa_dir, f)
        if process_file(filepath):
            print(f'Tagged: {f}')
        else:
            print(f'Skipped (already tagged or no steps): {f}')


if __name__ == '__main__':
    main()
