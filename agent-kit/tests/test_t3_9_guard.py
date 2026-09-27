import pathlib
import sys

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parents[1] / "runner"))

import guard


def test_t3_9_guard_stops_below_threshold():
    assert guard.should_run(2.0, 3.0) is False


def test_t3_9_guard_proceeds_above_threshold():
    assert guard.should_run(5.0, 3.0) is True


def test_t3_9_guard_proceeds_at_threshold():
    assert guard.should_run(3.0, 3.0) is True


def test_t3_9_guard_message_names_days_and_threshold():
    msg = guard.guard_message(1.5, 3)
    assert "1.5" in msg
    assert "3" in msg
