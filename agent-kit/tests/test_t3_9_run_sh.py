import json
import os
import pathlib
import stat
import subprocess

RUNNER_DIR = pathlib.Path(__file__).resolve().parents[1] / "runner"


def make_stub(path, body):
    path.write_text(f"#!/usr/bin/env bash\n{body}\n")
    path.chmod(path.stat().st_mode | stat.S_IEXEC)


def run_with_fuel(tmp_path, days, threshold="3"):
    bin_dir = tmp_path / "bin"
    bin_dir.mkdir()
    claude_called_marker = tmp_path / "claude_called"

    make_stub(
        bin_dir / "icp",
        f'echo \'{{"days_of_fuel_estimate": {days}}}\'',
    )
    make_stub(
        bin_dir / "claude",
        f"touch {claude_called_marker}",
    )

    (tmp_path / ".space-compute.json").write_text(
        json.dumps({"aaa": "aaa-id", "identity": "sc-operator-test", "net": "-n ic"})
    )

    log_home = tmp_path / "home"
    log_home.mkdir()

    env = dict(os.environ)
    env["PATH"] = f"{bin_dir}:{env['PATH']}"
    env["HOME"] = str(log_home)
    env["SPACE_COMPUTE_DIR"] = str(tmp_path)
    env["SPACE_COMPUTE_FUEL_THRESHOLD"] = threshold

    result = subprocess.run(
        ["bash", str(RUNNER_DIR / "run.sh")],
        env=env,
        capture_output=True,
        text=True,
        timeout=30,
    )
    return result, claude_called_marker


def test_t3_9_run_sh_stops_at_fuel_guard(tmp_path):
    result, marker = run_with_fuel(tmp_path, days=1, threshold="3")
    assert result.returncode == 0
    assert not marker.exists()
    assert "fuel guard" in result.stdout


def test_t3_9_run_sh_proceeds_above_threshold(tmp_path):
    result, marker = run_with_fuel(tmp_path, days=10, threshold="3")
    assert result.returncode == 0
    assert marker.exists()
