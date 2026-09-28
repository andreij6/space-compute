import argparse
import json
import subprocess
from pathlib import Path

CANISTER = "platform"


def call(method: str, args: str, network: str = "local", identity: str = "anonymous") -> dict:
    cmd = ["icp", "canister", "call", CANISTER, method, args, "-e", network, "--identity", identity, "--query", "--json"]
    out = subprocess.run(cmd, capture_output=True, text=True, check=True)
    return json.loads(out.stdout)


def fetch_discoveries(caller=call, network: str = "local", page_size: int = 100) -> list[dict]:
    items, cursor = [], None
    while True:
        cursor_arg = "null" if cursor is None else f"opt ({cursor} : nat64)"
        args = f"(record {{ status = opt variant {{ Confirmed }}; category = null }}, {cursor_arg}, {page_size} : nat32)"
        page = caller("list_discoveries", args, network)
        batch = page.get("items", [])
        items += batch
        cursor = page.get("next_cursor")
        if not batch or cursor is None:
            break
    return items


def fetch_citation(public_id: str, caller=call, network: str = "local") -> dict | None:
    result = caller("get_citation", f'("{public_id}")', network)
    return result.get("citation") if result else None


def export(out: Path, caller=call, network: str = "local") -> list[dict]:
    rows = [{"discovery": d, "citation": fetch_citation(d["public_id"], caller, network)} for d in fetch_discoveries(caller, network)]
    rows.sort(key=lambda r: r["discovery"]["public_id"])
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(rows, indent=1, sort_keys=True))
    return rows


def main() -> None:
    p = argparse.ArgumentParser(description="Export resolved discoveries + citations from the platform's public queries")
    p.add_argument("--out", default="../../target/curation/v1/discoveries_export.json")
    p.add_argument("--network", default="local")
    a = p.parse_args()
    rows = export(Path(a.out), network=a.network)
    print(json.dumps({"discoveries": len(rows)}, indent=1))


if __name__ == "__main__":
    main()
