import argparse
import json
import re
import subprocess
from pathlib import Path

CANISTER = "platform"
TOKEN = re.compile(r'\s*("(?:[^"\\]|\\.)*"|:\s*[a-z]\w*|[(){};=,]|-?[\d_]+(?:\.[\d_]+)?(?:[eE][-+]?\d+)?|[A-Za-z_]\w*)')
ESCAPES = {"n": "\n", "t": "\t", "r": "\r", '"': '"', "'": "'", "\\": "\\"}


def unescape(body: str) -> str:
    def repl(m: re.Match) -> str:
        e = m.group(1)
        if e.startswith("u{"):
            return chr(int(e[2:-1], 16))
        return chr(int(e, 16)) if len(e) == 2 else ESCAPES.get(e, e)
    return re.sub(r"\\(u\{[0-9a-fA-F]+\}|[0-9a-fA-F]{2}|.)", repl, body)


def tokenize(text: str) -> list[str]:
    tokens, pos = [], 0
    while pos < len(text.rstrip()):
        m = TOKEN.match(text, pos)
        if not m:
            raise ValueError(f"unparseable candid at {pos}: {text[pos:pos + 40]!r}")
        if not m.group(1).startswith(":"):
            tokens.append(m.group(1))
        pos = m.end()
    return tokens


def parse_candid(text: str):
    tokens = tokenize(text)
    i = 0

    def value():
        nonlocal i
        t = tokens[i]
        i += 1
        if t.startswith('"'):
            return unescape(t[1:-1])
        if t in ("opt", "blob", "principal"):
            return value()
        if t == "null":
            return None
        if t in ("true", "false"):
            return t == "true"
        if t in ("record", "vec", "variant"):
            return block(t)
        n = t.replace("_", "")
        return float(n) if any(c in n for c in ".eE") else int(n)

    def block(kind):
        nonlocal i
        i += 1
        fields, items = {}, []
        while tokens[i] != "}":
            if tokens[i + 1] == "=":
                key = tokens[i]
                i += 2
                fields[key] = value()
            elif kind == "variant":
                fields[tokens[i]] = None
                i += 1
            else:
                items.append(value())
            if tokens[i] == ";":
                i += 1
        i += 1
        if kind == "variant":
            (key, val), = fields.items()
            return key if val is None else {key: val}
        return fields if fields else items if kind == "vec" or items else {}

    i = 1
    values = []
    while tokens[i] != ")":
        values.append(value())
        if tokens[i] == ",":
            i += 1
    return values[0] if len(values) == 1 else values


def call(method: str, args: str, network: str = "local", identity: str = "anonymous"):
    cmd = ["icp", "canister", "call", CANISTER, method, args, "-e", network, "--identity", identity, "--query", "--json"]
    out = subprocess.run(cmd, capture_output=True, text=True, check=True)
    return parse_candid(json.loads(out.stdout)["response_candid"])


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


def fetch_consensus(subject_ids, caller=call, network: str = "local", identity: str = "anonymous") -> list[dict]:
    rows = []
    for sid in sorted(subject_ids):
        cons = caller("get_subject_consensus", f"({sid} : nat32)", network, identity)
        if not cons:
            continue
        votes: dict[str, dict[str, int]] = {}
        for c in caller("list_subject_classifications", f"({sid} : nat32)", network, identity) or []:
            if c.get("image_mismatch"):
                continue
            for a in c["answers"]:
                q = votes.setdefault(a["question_id"], {})
                q[a["answer_id"]] = q.get(a["answer_id"], 0) + 1
        rows.append({**cons, "votes": votes} if votes else cons)
    return rows


def export(out: Path, caller=call, network: str = "local") -> list[dict]:
    rows = [{"discovery": d, "citation": fetch_citation(d["public_id"], caller, network)} for d in fetch_discoveries(caller, network)]
    rows.sort(key=lambda r: r["discovery"]["public_id"])
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(rows, indent=1, sort_keys=True))
    return rows


def main() -> None:
    p = argparse.ArgumentParser(description="Export resolved discoveries + citations from the platform's public queries")
    p.add_argument("--out", default="../../target/curation/v1/discoveries_export.json")
    p.add_argument("--consensus-out", default="../../target/curation/v1/consensus_export.json")
    p.add_argument("--manifest", default="../../data/curation/v1/manifest_v1.jsonl")
    p.add_argument("--network", default="local")
    p.add_argument("--identity", default="anonymous")
    a = p.parse_args()
    rows = export(Path(a.out), network=a.network)
    ids = [json.loads(l)["subject_id"] for l in Path(a.manifest).read_text().splitlines()]
    consensus = fetch_consensus(ids, network=a.network, identity=a.identity)
    Path(a.consensus_out).write_text(json.dumps(consensus, indent=1, sort_keys=True))
    print(json.dumps({"discoveries": len(rows), "consensus_subjects": len(consensus)}, indent=1))


if __name__ == "__main__":
    main()
