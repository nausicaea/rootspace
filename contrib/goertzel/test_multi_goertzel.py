from collections import deque
from collections.abc import Iterable, Iterator, Generator, Sequence
from dataclasses import dataclass, InitVar, field
from math import cos, sin, tau, ceil
import numpy as np
from numpy import ndarray, array, concat, zeros
from numpy.random import default_rng
from numpy.random import Generator as NpGenerator
from pytest import fixture, FixtureRequest, approx, mark
from itertools import islice, repeat
from typing import NamedTuple, Any, Callable
import matplotlib.pyplot as plt
from matplotlib.colors import ListedColormap
from modulate import Spec, difference, ratio, modulate, normalize, center


@dataclass(slots=True, frozen=True)
class State:
    d1: float
    d2: float


ZERO: State = State(0.0, 0.0)


def _filter_pass(state: State, omega_real: float, sample: float) -> State:
    return State(
        sample + omega_real * state.d1 - state.d2,
        state.d1,
    )


@dataclass(slots=True, frozen=True)
class Snapshot:
    index: int
    sample_rate: int
    frequencies: list[int]
    omega: list[complex]
    state: list[State]

    def power(self) -> Generator[float, None, None]:
        """
        Return the signal power (signed value, linear scale) at a specific
        point in time for every frequency.
        """
        return (
            pow(s.d1, 2) + pow(s.d2, 2) - o.real * s.d1 * s.d2
            for o, s in zip(self.omega, self.state)
        )

    def total_power(self) -> float:
        """
        Return the sum of the absolutes of each frequency power.
        """
        return sum(abs(p) for p in self.power())


@dataclass(slots=True)
class NumericallyControlledOscillator:
    """
    Represents a circular iterator over the range 0:period_length. This iterator is infinite.
    """

    period_length: int
    counter: int = field(init=False, default=0)

    def reset(self) -> None:
        """
        Reset the current iteration count to zero.
        """
        self.counter = 0

    def is_at_start(self) -> bool:
        return self.counter == 0

    def is_at_half(self) -> bool:
        return self.counter == self.period_length // 2

    def is_at_end(self) -> bool:
        return self.counter == self.period_length - 1

    def __iter__(self) -> "NumericallyControlledOscillator":
        return self

    def __next__(self) -> int:
        c = self.counter
        self.counter = (self.counter + 1) % self.period_length
        return c


@dataclass
class Tracker:
    _attrs: dict[str, list[Any]] = field(default_factory=dict)

    def register(self, key: str) -> None:
        if key not in self._attrs:
            self._attrs[key] = list()

    def insert(self, key: str, value: Any) -> None:
        self._attrs[key].append(value)

    def get(self, key: str) -> ndarray:
        v = self._attrs[key]
        return array(v)

    def get_and[T](self, key: str, fun: Callable[[ndarray], T]) -> T | None:
        try:
            return fun(self.get(key))
        except KeyError:
            return None

    def get_and2[T](
        self, keys: tuple[str, str], fun: Callable[[ndarray, ndarray], T]
    ) -> T | None:
        try:
            key_a, key_b = keys
            return fun(self.get(key_a), self.get(key_b))
        except KeyError:
            return None


def classify(power_delta: float, threshold: float) -> bool:
    return power_delta > threshold


def classify_with_hysteresis(
    prev_bit: bool, power_delta: float, threshold: float, hysteresis: float
) -> bool:
    if not prev_bit and power_delta > (threshold + hysteresis):
        return True
    elif prev_bit and power_delta < (threshold - hysteresis):
        return False
    else:
        return prev_bit


def demodulate(
    spec: Spec,
    signal: Iterable[float],
    preamble: list[bool],
    tracker: Tracker | None = None,
) -> Generator[Snapshot, None, None]:
    def track(k: str, v: Any) -> None:
        if tracker is not None:
            tracker.insert(k, v)

    if tracker is not None:
        tracker.register("initial_sync_nco")
        tracker.register("initial_sync_complete")
        tracker.register("initial_sync_total_power")
        tracker.register("with_preamble_bit_candidate")
        tracker.register("with_preamble_buffer")
        tracker.register("with_preamble_preamble_matched")
        tracker.register("with_preamble_power")

    frequencies = [spec.mark_frequency, spec.space_frequency]

    # Goertzel algorithm variables
    phi = (tau * freq / spec.sample_rate for freq in frequencies)
    omega = [complex(2 * cos(p), sin(p)) for p in phi]
    state = [ZERO for _ in range(len(frequencies))]

    # Initial synchronization variables
    signal_synchronized = False
    total_power_threshold = 25
    nco = NumericallyControlledOscillator(spec.bit_width())

    # Preamble detection variables
    preamble_matched = False
    buffer = deque(repeat(False, len(preamble)), maxlen=len(preamble))
    delta_power_threshold = 0
    delta_power_hysteresis = 5
    bit_candidate = False

    # Iterate over the incoming waveform
    for idx, sample in enumerate(iter(signal)):
        # Apply the Goertzel filter pass
        state = [_filter_pass(s, o.real, sample) for o, s in zip(omega, state)]
        snapshot = Snapshot(idx, spec.sample_rate, frequencies, omega, state)

        # Debugging
        track("initial_sync_nco", nco.counter)
        track("initial_sync_complete", signal_synchronized)
        track("initial_sync_total_power", snapshot.total_power())
        track("with_preamble_bit_candidate", bit_candidate)
        track("with_preamble_buffer", list(buffer))
        track("with_preamble_preamble_matched", preamble_matched)
        track("with_preamble_power", list(snapshot.power()))

        if nco.is_at_end():
            state = [ZERO for _ in range(len(frequencies))]

        if not signal_synchronized:
            if snapshot.total_power() >= total_power_threshold:
                nco.reset()
                signal_synchronized = True

            next(nco)
            continue

        if not preamble_matched:
            if nco.is_at_half():
                mark_power, space_power = tuple(islice(snapshot.power(), 2))
                power_delta = mark_power - space_power
                bit_candidate = classify_with_hysteresis(
                    buffer[-1],
                    power_delta,
                    delta_power_threshold,
                    delta_power_hysteresis,
                )
                buffer.append(bit_candidate)

                if all(l == r for l, r in zip(buffer, preamble)):
                    preamble_matched = True

            next(nco)
            continue

        next(nco)
        yield snapshot


def test_state_constructor() -> None:
    s = State(1, 2)
    assert s.d1 == 1 and s.d2 == 2


def plot(
    filename: str, signal: ndarray, output: Iterable[Snapshot], tracker: Tracker
) -> None:
    fig = plt.figure(figsize=(10, 12), layout="constrained")
    axs = fig.subplot_mosaic(
        [
            ["signal"],
            ["output"],
            ["nco"],
            ["buffer"],
            ["preamble_matched"],
            ["power"],
            ["total_power"],
            ["power_delta"],
            ["power_ratio"],
            ["bit_candidate"],
        ]
    )
    axs["signal"].set_title("Original Signal (normalized)")
    axs["signal"].plot(signal)
    axs["output"].set_title("Demodulation Output")
    axs["output"].plot(array([tuple(o.power()) for o in output]))
    axs["nco"].set_title("NCO Clock State")
    tracker.get_and("initial_sync_nco", lambda d: axs["nco"].plot(d))
    axs["buffer"].set_title("Preamble Buffer")
    tracker.get_and(
        "with_preamble_buffer",
        lambda d: axs["buffer"].imshow(
            d.T,
            cmap=ListedColormap(["red", "green"]),
            origin="lower",
            aspect="auto",
            interpolation="nearest",
        ),
    )
    axs["preamble_matched"].set_title("Preamble Matched")
    tracker.get_and(
        "with_preamble_preamble_matched", lambda d: axs["preamble_matched"].plot(d)
    )
    axs["power"].set_title("Post-Goertzel Signal Power")
    tracker.get_and("with_preamble_power", lambda d: axs["power"].plot(d))
    axs["total_power"].set_title("Total Power")
    tracker.get_and("initial_sync_total_power", lambda d: axs["total_power"].plot(d))
    axs["power_delta"].set_title("Power Delta")
    tracker.get_and2(
        ("with_preamble_power", "initial_sync_total_power"),
        lambda a, b: axs["power_delta"].plot(difference(a, b)),
    )
    axs["power_ratio"].set_title("Power Ratio")
    tracker.get_and2(
        ("with_preamble_power", "initial_sync_total_power"),
        lambda a, b: axs["power_ratio"].plot(ratio(a, b)),
    )
    axs["bit_candidate"].set_title("Bit Candidate")
    tracker.get_and(
        "with_preamble_bit_candidate", lambda d: axs["bit_candidate"].plot(d)
    )
    plt.savefig(filename)


# @mark.skip
@mark.parametrize("n", [128, 256])
def test_with_preamble_only_noise(f_spec: Spec, f_rng: NpGenerator, n: int) -> None:
    """
    The preamble shall not be detected in a noise-only signal.
    """
    tracker = Tracker()
    signal = f_rng.standard_normal(n)
    output = list(
        demodulate(
            f_spec, signal, [True, False, True, False, True, False], tracker=tracker
        )
    )
    filename = f"with-preamble-only-noise-{n}len.png"
    plot(filename, signal, output, tracker)

    assert (tracker.get("with_preamble_preamble_matched") == False).all()


@mark.parametrize("p", [0, 20, 40, 80, 100, 120])
def test_with_preamble_only_preamble_with_padding_no_noise(
    f_spec: Spec, f_rng: NpGenerator, f_preamble: ndarray, p: int
) -> None:
    """
    The preamble must be detected if it is present.
    """

    tracker = Tracker()
    modulated = np.array(list(modulate(f_spec, f_preamble)))
    s = concat([zeros((p,)), modulated, zeros((p,))])
    signal = normalize(center(s))
    output = list(
        demodulate(f_spec, signal, [bool(v) for v in f_preamble], tracker=tracker)
    )

    pmble = "".join("1" if v else "0" for v in f_preamble)
    filename = f"with-preamble-only-preamble-{pmble}preamble-{p}padding.png"

    plot(filename, signal, output, tracker)

    assert tracker.get("with_preamble_preamble_matched")[-1] == True
    assert (tracker.get("with_preamble_buffer")[-1, :] == f_preamble).all()


@mark.parametrize("w", [0.0, 0.1, 0.2, 0.3, 0.8])
def test_with_preamble_only_preamble_no_padding_with_noise(
    f_spec: Spec, f_rng: NpGenerator, f_preamble: ndarray, w: float
) -> None:
    """
    The preamble must be detected if it is present.
    """

    tracker = Tracker()
    modulated = np.array(list(modulate(f_spec, f_preamble)))
    s = modulated
    n = f_rng.standard_normal(len(modulated))
    signal = normalize(center(s + w * n))
    output = list(
        demodulate(f_spec, signal, [bool(v) for v in f_preamble], tracker=tracker)
    )

    pmble = "".join("1" if v else "0" for v in f_preamble)
    filename = f"with-preamble-only-preamble-{pmble}preamble-{w}noise.png"

    plot(filename, signal, output, tracker)

    assert tracker.get("with_preamble_preamble_matched")[-1] == True
    assert (tracker.get("with_preamble_buffer")[-1, :] == f_preamble).all()
