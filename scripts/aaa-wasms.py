import re
import sys
import time

SOAK_NS = 48 * 3600 * 10**9


def wasms(text: str) -> list[dict]:
    out = []
    for version, body in re.findall(r"([\d_]+)\s*:\s*nat32;\s*record\s*\{([^}]*)\}", text):
        blob = re.search(r'module_sha256\s*=\s*blob\s*"((?:\\[0-9a-fA-F]{2})*)"', body)
        approved = re.search(r"approved\s*=\s*(true|false)", body)
        released = re.search(r"released_at\s*=\s*([\d_]+)", body)
        out.append({
            "version": int(version.replace("_", "")),
            "module_sha256": blob.group(1).replace("\\", "").lower() if blob else "",
            "approved": bool(approved) and approved.group(1) == "true",
            "released_at": int(released.group(1).replace("_", "")) if released else 0,
        })
    return out


def decide(text: str, target: str) -> str:
    entries = wasms(text)
    if any(e["module_sha256"] == target.lower() and e["approved"] for e in entries):
        return "ALREADY_APPROVED"
    return f"NEXT_VERSION={max((e['version'] for e in entries), default=0) + 1}"


def soaked(text: str, target: str, now_ns: int, soak_ns: int = SOAK_NS) -> bool:
    return any(
        e["module_sha256"] == target.lower() and e["approved"] and 0 < e["released_at"] <= now_ns - soak_ns
        for e in wasms(text)
    )


if __name__ == "__main__":
    cmd, target = sys.argv[1], sys.argv[2]
    text = sys.stdin.read()
    if cmd == "decide":
        print(decide(text, target))
    elif cmd == "soaked":
        soak_ns = int(sys.argv[3]) if len(sys.argv) > 3 else SOAK_NS
        sys.exit(0 if soaked(text, target, time.time_ns(), soak_ns) else 1)
    else:
        sys.exit(f"unknown command {cmd}")
