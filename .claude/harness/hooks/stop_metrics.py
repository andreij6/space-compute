#!/usr/bin/env python3
"""Stop hook: aggregate token usage, tool errors and duration for this session → metrics/<session>.json."""
import json, os, sys
inp = json.load(sys.stdin)
H = os.path.join(os.environ.get("CLAUDE_PROJECT_DIR", "."), ".claude/harness")
path = inp.get("transcript_path")
if not path or not os.path.exists(path): sys.exit(0)
tok = dict(input=0, output=0, cache_read=0, cache_write=0); errors = turns = 0; ts = []
for line in open(path):
    try: e = json.loads(line)
    except ValueError: continue
    if e.get("timestamp"): ts.append(e["timestamp"])
    m = e.get("message") or {}
    u = m.get("usage") if isinstance(m, dict) else None
    if u:
        turns += 1
        tok["input"] += u.get("input_tokens", 0); tok["output"] += u.get("output_tokens", 0)
        tok["cache_read"] += u.get("cache_read_input_tokens", 0); tok["cache_write"] += u.get("cache_creation_input_tokens", 0)
    for c in (m.get("content") if isinstance(m.get("content"), list) else []):
        if isinstance(c, dict) and c.get("type") == "tool_result" and c.get("is_error"): errors += 1
cur = open(f"{H}/current").read().strip() if os.path.exists(f"{H}/current") else None
os.makedirs(f"{H}/metrics", exist_ok=True)
json.dump({"session": inp.get("session_id"), "task": cur, "tokens": tok, "api_calls": turns, "tool_errors": errors,
           "first": ts[0] if ts else None, "last": ts[-1] if ts else None},
          open(f"{H}/metrics/{inp.get('session_id','unknown')}.json", "w"), indent=1)
