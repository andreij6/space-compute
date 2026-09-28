import json
import os
import pathlib
import stat
import subprocess

RUNNER_DIR = pathlib.Path(__file__).resolve().parents[1] / "runner"
AAA = "ygs6i-cd777-77775-aaa2q-cai"


def make_stub(path, body):
    path.write_text(f"#!/usr/bin/env bash\n{body}\n")
    path.chmod(path.stat().st_mode | stat.S_IEXEC)


def status_envelope(days, variant="Ok"):
    if variant == "Err":
        candid = "(variant { Err = variant { Unauthorized } })"
    else:
        candid = (
            "(\n  variant {\n    Ok = record {\n      wasm_version = \"1\";\n"
            "      cycles = 3_015_589_305_743 : nat;\n"
            f"      days_of_fuel_estimate = {days} : float64;\n    }}\n  }},\n)"
        )
    return json.dumps({"response_bytes": "4449", "response_text": None, "response_candid": candid})


def run_runner(tmp_path, days=10, threshold="3", variant="Ok", config=None, project=None):
    project = project or tmp_path
    project.mkdir(parents=True, exist_ok=True)
    bin_dir = tmp_path / "bin"
    bin_dir.mkdir()
    icp_args = tmp_path / "icp_args"
    claude_marker = tmp_path / "claude_called"

    (tmp_path / "status.json").write_text(status_envelope(days, variant))
    make_stub(
        bin_dir / "icp",
        f'printf "%s\\n" "$@" > "{icp_args}"\ncat "{tmp_path / "status.json"}"',
    )
    make_stub(
        bin_dir / "claude",
        f'{{ pwd; printf "%s\\n" "$@"; }} > "{claude_marker}"',
    )

    cfg = {"aaa": AAA, "identity": "sc-operator-test", "net": "-n ic"}
    cfg.update(config or {})
    (project / ".space-compute.json").write_text(json.dumps(cfg))

    log_home = tmp_path / "home"
    log_home.mkdir()

    env = dict(os.environ)
    env["PATH"] = f"{bin_dir}:{env['PATH']}"
    env["HOME"] = str(log_home)
    env["SPACE_COMPUTE_DIR"] = str(project)
    env["SPACE_COMPUTE_FUEL_THRESHOLD"] = threshold

    result = subprocess.run(
        ["bash", str(RUNNER_DIR / "run.sh")],
        env=env,
        capture_output=True,
        text=True,
        timeout=30,
        cwd=tmp_path,
    )
    return result, claude_marker, icp_args


def test_t3_9_run_sh_stops_at_fuel_guard(tmp_path):
    result, marker, _ = run_runner(tmp_path, days=1, threshold="3")
    assert result.returncode == 0
    assert not marker.exists()
    assert "fuel guard" in result.stdout


def test_t3_9_run_sh_proceeds_above_threshold(tmp_path):
    result, marker, _ = run_runner(tmp_path, days="10.5", threshold="3")
    assert result.returncode == 0, result.stdout + result.stderr
    assert marker.exists()


def test_t3_9_run_sh_fresh_aaa_with_infinite_fuel_proceeds(tmp_path):
    result, marker, _ = run_runner(tmp_path, days="inf")
    assert result.returncode == 0, result.stdout + result.stderr
    assert marker.exists()


def test_t3_9_run_sh_err_status_stops(tmp_path):
    result, marker, _ = run_runner(tmp_path, variant="Err")
    assert result.returncode == 1
    assert not marker.exists()


def test_t3_9_run_sh_passes_candid_json_query_identity_and_net(tmp_path):
    _, _, icp_args = run_runner(tmp_path, config={"net": "-e local"})
    args = icp_args.read_text().split("\n")
    assert args[:4] == ["canister", "call", AAA, "status"]
    for flag in ("--query", "--json", "--candid"):
        assert flag in args
    assert args[args.index("--identity") + 1] == "sc-operator-test"
    assert args[args.index("-e") + 1] == "local"
    assert args[args.index("--candid") + 1].endswith("reference/aaa.did")


def test_t3_9_run_sh_rejects_untrusted_config_values(tmp_path):
    bad = [
        {"net": "-n ic; touch pwned"},
        {"net": "--network mainnet"},
        {"aaa": "aaa-id; touch pwned"},
        {"identity": "../owner"},
        {"identity": "x $(touch pwned)"},
    ]
    for i, cfg in enumerate(bad):
        case = tmp_path / f"case{i}"
        case.mkdir()
        result, marker, icp_args = run_runner(case, config=cfg)
        assert result.returncode == 1, cfg
        assert not icp_args.exists(), cfg
        assert not marker.exists(), cfg
        assert not (case / "pwned").exists()


def test_t3_9_run_sh_config_path_is_not_interpolated_into_python(tmp_path):
    project = tmp_path / "it's here"
    result, marker, _ = run_runner(tmp_path, project=project)
    assert result.returncode == 0, result.stdout + result.stderr
    assert marker.exists()


def test_t3_9_run_sh_runs_claude_in_project_dir_with_restricted_tools(tmp_path):
    project = tmp_path / "proj"
    _, marker, _ = run_runner(tmp_path, project=project)
    lines = marker.read_text().split("\n")
    assert pathlib.Path(lines[0]).resolve() == project.resolve()
    args = lines[1:]
    assert "--allowedTools" in args
    assert "Bash(icp canister call:*)" in args
    assert not any(a in ("Bash", "Bash(*)", "WebFetch", "WebSearch") for a in args[args.index("--allowedTools") + 1 :])
    assert args[args.index("--tools") + 1 : args.index("--tools") + 3] == ["Bash", "Read"]
    assert int(args[args.index("--max-turns") + 1]) > 0
    assert float(args[args.index("--max-budget-usd") + 1]) > 0
