#!/usr/bin/env python3
"""Control the ESP32 observer or reduce a JSONL file captured on the bench."""
import argparse
import json
import math
import os
import re
import statistics
import sys

HZ = 80_000_000
WRAP = 1 << 32
MARKERS = 8
JITTER_TICKS = 256


def fields(line, prefix):
    words = line.split()
    if not words or words[0] != prefix:
        raise ValueError(f"expected {prefix}, received {line!r}")
    return dict(token.split("=", 1) for token in words[1:])


class Observer:
    def __init__(self, send, receive):
        self.send = send
        self.receive = receive

    def request(self, command):
        self.send(command)
        reply = []
        while True:
            line = self.receive()
            if line == "OK":
                return reply
            if line.startswith("ERR "):
                raise ValueError(f"{command}: {line}")
            reply.append(line)

    def arm(self, nonce, profile, limit):
        if self.request(f"obs_arm {nonce} {profile} {limit}"):
            raise ValueError("unexpected arm payload")

    def dump(self, nonce):
        status, *extra = self.request(f"obs_stop {nonce}")
        state = fields(status, "obs")
        counts = list(map(int, state.pop("pcnt").split(",")))
        state = {key: int(value) for key, value in state.items()}
        if state["nonce"] != nonce or state["active"]:
            raise ValueError(f"wrong stopped capture {state}")
        edges = []
        while len(edges) < state["count"]:
            offset = len(edges)
            count = min(64, state["count"] - offset)
            header, *lines = self.request(f"obs_read {nonce} {offset} {count}")
            if fields(header, "obs_data") != dict(nonce=str(nonce), offset=str(offset), count=str(count)):
                raise ValueError(f"bad page header {header}")
            if len(lines) != count:
                raise ValueError(f"short page {len(lines)} of {count}")
            for line in lines:
                match = re.fullmatch(r"edge (\d+) (\d+) ([01])", line)
                if not match:
                    raise ValueError(f"bad edge {line}")
                edges.append(list(map(int, match.groups())))
        capture = dict(state=state, pcnt=counts, edges=edges)
        if extra:
            line, = extra
            head = line.split(maxsplit=5)
            spi = {key: int(value) for key, value in fields(" ".join(head[:5]), "obs_spi").items()}
            spi["raw"] = [int(word, 16) for word in head[5].split()] if len(head) == 6 else []
            if spi["items"] != len(spi["raw"]):
                raise ValueError(f"bad RMT record {line}")
            capture["spi"] = spi
        return capture


def require(condition, message):
    if not condition:
        raise ValueError(message)


def envelope(values):
    return [min(values), max(values)]


def series(trial, pins, spi=False):
    for key in ("expired", "dropped", "saturated"):
        require(trial["state"][key] == 0, f"capture {key}={trial['state'][key]}")
    require({row[1] for row in trial["edges"]} == set(pins[:1] if spi else pins),
            "unexpected capture pin set")
    by_pin = []
    for i, pin in enumerate(pins[:1] if spi else pins):
        edges = [(tick, high) for tick, channel, high in trial["edges"] if channel == pin]
        ticks = [(tick - edges[0][0]) % WRAP for tick, _ in edges]
        require(all(a < b for a, b in zip(ticks, ticks[1:])) and ticks[-1] < WRAP // 2,
                f"GPIO{pin}: timestamp order or ambiguous wrap")
        require(all(a[1] != b[1] for a, b in zip(edges, edges[1:])),
                f"GPIO{pin}: edge polarity sequence")
        require(len(edges) == trial["pcnt"][i], f"GPIO{pin}: MCPWM count {len(edges)} != PCNT {trial['pcnt'][i]}")
        by_pin.append(edges)
    return by_pin


def signed_delta(new, old):
    return (new - old + WRAP // 2) % WRAP - WRAP // 2


def fit(ticks, brackets):
    # The integer RP1 hardware counter floors the end timestamp to the current microsecond.
    origin, baseline = ticks[0], brackets[0][0]
    x = [(t - origin) % WRAP for t in ticks]
    bounds = [[(v[0] - baseline) % WRAP, (v[1] - baseline) % WRAP + 1] for v in brackets]
    midpoint = [(lo + hi) / 2 for lo, hi in bounds]
    slope, offset = statistics.linear_regression(x, midpoint)
    residual = [offset + slope * t - y for t, y in zip(x, midpoint)]
    # Use all marker bracket pairs. Allow two nominal APB ticks for capture quantization.
    low, high = 0, math.inf
    for i, (lo_i, hi_i) in enumerate(bounds):
        for j in range(i + 1, len(bounds)):
            dt = x[j] - x[i]
            lo_j, hi_j = bounds[j]
            low = max(low, (lo_j - hi_i - 0.025) / dt)
            high = min(high, (hi_j - lo_i + 0.025) / dt)
    require(low <= high, "marker brackets admit no shared linear rate")
    slope_error_ppm = max(abs(slope / low - 1), abs(slope / high - 1)) * 1e6
    return dict(origin=origin, rp1_origin=baseline, offset_us=offset, us_per_tick=slope,
                rate_error_ppm=slope_error_ppm, esp_vs_rp1_ppm=(1e6 / slope / HZ - 1) * 1e6,
                esp_vs_rp1_ppm_bounds=[(1e6 / s / HZ - 1) * 1e6 for s in (high, low)],
                residual_us=envelope(residual),
                residual_rms_us=math.sqrt(statistics.mean(v * v for v in residual)),
                max_rp1_bracket_us=max(hi - lo for lo, hi in bounds))


def triples(data):
    return list(zip(data[::3], data[1::3], data[2::3]))


def marker_trial(trial, jitter=False, spi=False):
    primary, secondary = series(trial, [26, 33] if spi else [34, 4])
    expected = 2 * MARKERS
    require(len(primary) == expected + (JITTER_TICKS if jitter else 0) and len(secondary) == expected,
            f"marker counts {len(primary)}/{len(secondary)}")
    markers = primary[:MARKERS] + primary[-MARKERS:]
    brackets = triples(trial["rp1"][:3 * expected])
    require(len(brackets) == expected, "RP1 marker count")
    mask = (1 << 8) | (1 << 11) if spi else (1 << 12) | (1 << 13)
    polarity = [1 - i % 2 for i in range(expected)]
    require([high for _, high in markers] == polarity and [high for _, high in secondary] == polarity
            and [v[2] for v in brackets] == [mask * high for high in polarity], "marker polarity")
    fitted = fit([tick for tick, _ in markers], brackets)
    skew = [signed_delta(b, a) * fitted["us_per_tick"] for (a, _), (b, _) in zip(markers, secondary)]
    fitted["secondary_minus_primary_us"] = envelope(skew)
    return fitted, primary[MARKERS:-MARKERS]


def within(values, bounds, label):
    error = [v for v in values if not bounds[0] <= v <= bounds[1]]
    require(not error, f"{label}: {len(error)} out of bounds {bounds}; range {envelope(values)}")


def pwm(trial, calibrated, frequency):
    slope = calibrated["us_per_tick"]
    rate_error = calibrated["rate_error_ppm"] / 1e6
    period, high, clock, _ = trial["rp1"]
    require(clock / period == frequency and high * 2 == period, "wrong HAL PWM config")
    output = {}
    for pin, events in zip((34, 4), series(trial, [34, 4])):
        spans = []
        for first, falling, last in zip(events, events[1:], events[2:]):
            if first[1] == 1:
                spans.append(((last[0] - first[0]) % WRAP * slope,
                              (falling[0] - first[0]) % WRAP * slope))
        require(len(spans) >= 103, f"GPIO{pin}: too few complete periods")
        periods, highs = zip(*spans[2:102])
        target = 1e6 / frequency
        guard = [0.10 + rate_error * value for value in periods]
        lower = [p - e for p, e in zip(periods, guard)]
        upper = [p + e for p, e in zip(periods, guard)]
        duty = [100 * h / p for h, p in zip(highs, periods)]
        duty_lower = [100 * (h - e) / (p + e) for h, p, e in zip(highs, periods, guard)]
        duty_upper = [100 * (h + e) / (p - e) for h, p, e in zip(highs, periods, guard)]
        output[pin] = dict(period_us=envelope(periods), mean_period_us=statistics.mean(periods),
                           duty_percent=envelope(duty), mean_duty_percent=statistics.mean(duty),
                           accepted=all(target * 0.99 <= p <= target * 1.01 for p in lower + upper)
                           and all(49 <= d <= 51 for d in duty_lower + duty_upper),
                           complete_periods=len(spans), measured_periods=len(periods))
    return output


def jitter(trial):
    calibrated, edges = marker_trial(trial, jitter=True)
    data = triples(trial["rp1"][6 * MARKERS:])
    require(len(data) == JITTER_TICKS, "RP1 jitter sample count")
    origin = calibrated["rp1_origin"]
    first = (trial["deadline"] - origin) % WRAP
    slope = calibrated["us_per_tick"]
    x = [(t - calibrated["origin"]) % WRAP * slope + calibrated["offset_us"] for t, _ in edges]
    deadlines = [first + i * 1000 for i in range(JITTER_TICKS)]
    internal = [(before - origin) % WRAP - deadline for (before, _, _), deadline in zip(data, deadlines)]
    ext = [time - deadline for time, deadline in zip(x, deadlines)]
    diff = [e - i for e, i in zip(ext, internal)]
    gaps = [b - a for a, b in zip(x, x[1:])]
    require(all(high == (1 - i % 2) and levels == high << 13
                for i, ((_, high), (_, _, levels)) in enumerate(zip(edges, data))), "jitter polarity")
    checks = ((internal, [0, 50], "internal"), (ext, [-3, 53], "external"),
              (diff, [-3, 3], "external-minus-internal"), (gaps, [950, 1050], "interval"))
    violations = []
    for values, bounds, label in checks:
        try:
            within(values, bounds, label)
        except ValueError as error:
            violations.append(str(error))
    rank = math.ceil(0.99 * len(ext)) - 1
    summary = lambda values: dict(min=min(values), p99=sorted(values)[rank], max=max(values))
    return dict(internal_us=summary(internal), external_us=summary(ext),
                external_minus_internal_us=envelope(diff), external_gap_us=envelope(gaps),
                max_rp1_edge_bracket_us=max((b - a) % WRAP + 1 for a, b, _ in data),
                fit=calibrated, violations=violations, count=len(ext))


def spi_frame(trial, calibrated, skew):
    cs, = series(trial, [26, 33], spi=True)
    rmt = trial["spi"]
    require(trial["rp1"] == [1_000_000], "wrong HAL SPI config")
    require([high for _, high in cs] == [0, 1] and trial["pcnt"] == [2, 16, 0],
            "unexpected SPI PCNT or CS edges")
    require(rmt["latched"] and rmt["records"] == 1, "missing anchor or ambiguous RMT burst")
    widths = [(half & 0x7fff, half >> 15 & 1)
              for word in rmt["raw"] for half in (word & 0xffff, word >> 16) if half & 0x7fff]
    if widths and widths[0][1] == 0:
        widths.pop(0)
    if widths and widths[-1][1] == 0:
        widths.pop()
    require([level for _, level in widths] == [1 - i % 2 for i in range(15)],
            f"unexpected SPI RMT edge pattern {widths}")
    slope = calibrated["us_per_tick"]
    final = rmt["last"]
    initial = (final - sum(ticks for ticks, _ in widths)) % WRAP
    setup = signed_delta(initial, cs[0][0]) * slope
    hold = signed_delta(cs[1][0], final) * slope
    periods = [(a[0] + b[0]) * slope for a, b in zip(widths[::2], widths[1::2])]
    skew_guard = max(map(abs, skew["secondary_minus_primary_us"]))
    rate_error = calibrated["rate_error_ppm"] / 1e6
    guard = [0.10 + skew_guard + abs(t) * rate_error for t in (setup, hold)]
    # Bound a full tick of RMT quantization per half interval in the recovered first edge.
    guard[0] += len(widths) / (HZ / 1e6)
    accepted = (all(0.95 <= value - 0.025 - value * rate_error
                    and value + 0.025 + value * rate_error <= 1.05 for value in periods)
                and all(3.9 <= t - e and t + e <= 104 for t, e in zip((setup, hold), guard)))
    return dict(setup_us=setup, hold_us=hold, guard_us=guard, sclk_period_us=envelope(periods),
                first_sclk_tick=initial, last_sclk_tick=final, accepted=accepted)


def analyze(trials, *, calibration_and_pwm_only=False):
    grouped = {}
    for trial in trials:
        grouped.setdefault(trial["label"], []).append(trial)
    report = {}
    violations = []

    def run(label, calculate):
        try:
            group = grouped[label]
            require(not any("error" in trial for trial in group), "acquisition failed")
            report[label] = calculate(group)
        except (ValueError, KeyError, IndexError, ZeroDivisionError) as error:
            violations.append(f"{label}: {error}")

    run("calibration", lambda g: marker_trial(g[0])[0])
    if not calibration_and_pwm_only:
        run("spi_skew", lambda g: marker_trial(g[0], spi=True)[0])
    if "calibration" in report:
        cal = report["calibration"]
        # These are the frozen pre-run calibration acceptance bounds.
        for key, bounds in (("esp_vs_rp1_ppm_bounds", [-200, 200]), ("residual_us", [-1.5, 1.5]),
                            ("secondary_minus_primary_us", [-0.075, 0.075])):
            try:
                within(cal[key], bounds, key)
            except ValueError as error:
                violations.append(str(error))
        for frequency in (1000,) if calibration_and_pwm_only else (1000, 10000):
            label = f"pwm_{frequency}"
            run(label, lambda g: pwm(g[0], cal, frequency))
            if label in report and not all(row["accepted"] for row in report[label].values()):
                violations.append(f"{label}: range")
        if not calibration_and_pwm_only:
            run("spi", lambda g: [spi_frame(t, cal, report["spi_skew"]) for t in g])
            if "spi" in report and (len(report["spi"]) != 12 or not all(row["accepted"] for row in report["spi"])):
                violations.append("spi: count or range")
    if not calibration_and_pwm_only:
        run("jitter", lambda g: jitter(g[0]))
        if "jitter" in report:
            violations.extend(report["jitter"]["violations"])
    return dict(measurements=report, violations=violations)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--port", default=os.environ.get("CM5_ESP32_PORT"))
    commands = parser.add_subparsers(dest="command", required=True)
    commands.add_parser("caps")
    arm = commands.add_parser("arm")
    arm.add_argument("nonce", type=int)
    arm.add_argument("profile")
    arm.add_argument("limit", type=int)
    commands.add_parser("dump").add_argument("nonce", type=int)
    analysis = commands.add_parser("analyze")
    analysis.add_argument("capture", type=argparse.FileType("r"))
    analysis.add_argument("--calibration-and-pwm", action="store_true")
    args = parser.parse_args()
    if args.command == "analyze":
        report = analyze([json.loads(line) for line in args.capture],
                         calibration_and_pwm_only=args.calibration_and_pwm)
        print(json.dumps(report, indent=2, allow_nan=False))
        return bool(report["violations"])
    if not args.port:
        parser.error("set CM5_ESP32_PORT or pass --port for serial commands")

    import serial
    with serial.Serial(args.port, 115200, timeout=2, write_timeout=2, exclusive=True) as port:
        port.reset_input_buffer()

        def receive():
            wire = port.readline()
            if not wire.endswith(b"\n"):
                raise TimeoutError("observer serial response incomplete")
            return wire.decode("ascii").strip()

        client = Observer(lambda command: port.write((command + "\n").encode("ascii")), receive)
        if args.command == "caps":
            print(*client.request("obs_caps"), sep="\n")
        elif args.command == "arm":
            client.arm(args.nonce, args.profile, args.limit)
        else:
            print(json.dumps(client.dump(args.nonce)))
    return 0


if __name__ == "__main__":
    sys.exit(main())
