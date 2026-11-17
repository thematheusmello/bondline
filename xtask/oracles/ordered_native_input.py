"""Independent desired assertion over actual native probe evidence, including pre-patch RED."""
import json
import sys
from pathlib import Path

rows = [json.loads(line) for line in Path(sys.argv[1]).read_text().splitlines()]
focused = [row for row in rows if row["focused"]]
assert any(row["ctrl"] for row in focused), "OS Control-down did not reach the probe"
down = next(i for i, row in enumerate(focused) if row["ctrl"])
assert any(not row["ctrl"] for row in focused[down + 1:]), "OS Control-up not observed"
events = [event for row in rows for event in row["events"]]
transitions = [event for event in events if event.startswith("ModifiersChanged(")]
assert any("ctrl: true" in event for event in transitions), "ordered Control-down is missing"
press = next(i for i, event in enumerate(transitions) if "ctrl: true" in event)
assert any(event == "ModifiersChanged(Modifiers::NONE)" for event in transitions[press + 1:]), \
    "ordered Control-up is missing"
print(json.dumps({"rows": len(rows), "events": len(events),
                  "modifier_observations": len(transitions), "ordered_control": "pass"}))
