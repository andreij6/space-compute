import sys


def should_run(days_of_fuel_estimate, threshold):
    return days_of_fuel_estimate >= threshold


def guard_message(days_of_fuel_estimate, threshold):
    return f"fuel guard: {days_of_fuel_estimate} days < threshold {threshold}"


if __name__ == "__main__":
    days = float(sys.argv[1])
    threshold = float(sys.argv[2])
    if should_run(days, threshold):
        print("ok")
        sys.exit(0)
    print(guard_message(days, threshold))
    sys.exit(1)
