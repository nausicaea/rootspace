from collections import deque
from collections.abc import Iterable, Iterator, Generator, Sequence
from dataclasses import dataclass, InitVar, field
from math import cos, sin, tau, ceil
import numpy as np
from numpy import ndarray, array, concat, zeros
from numpy.random import default_rng
from numpy.random import Generator as NpGenerator
from pytest import fixture, FixtureRequest, approx, mark
from itertools import islice
from typing import NamedTuple


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
        return (pow(s.d1, 2) + pow(s.d2, 2) - o.real * s.d1 * s.d2 for o, s in zip(self.omega, self.state))

    def total_power(self) -> float:
        """
        Return the sum of the absolutes of each frequency power.
        """
        return sum(abs(p) for p in self.power())


@dataclass
class MultiGoertzel:
    signal: InitVar[Iterable[float]]
    sample_rate: int
    frequencies: list[int]
    omega: list[complex] = field(init=False)
    state: list[State] = field(init=False)
    signal_iter: Iterator[tuple[int, float]] = field(init=False)

    def reset(self) -> None:
        self.state = [ZERO for _ in range(len(self.state))]

    def __post_init__(self, sig: Iterable[float]) -> None:
        phi = (tau * freq / self.sample_rate for freq in self.frequencies)
        self.omega = [complex(2 * cos(p), sin(p)) for p in phi]
        self.state = [ZERO for _ in range(len(self.frequencies))]
        self.signal_iter = enumerate(iter(sig))

    def __iter__(self) -> 'MultiGoertzel':
        return self

    def __next__(self) -> Snapshot:
        idx, sample = next(self.signal_iter)
        self.state = [_filter_pass(s, o.real, sample) for o, s in zip(self.omega, self.state)]
        return Snapshot(idx, self.sample_rate, self.frequencies, self.omega, self.state)


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

    def __iter__(self) -> 'NumericallyControlledOscillator':
        return self

    def __next__(self) -> int:
        c = self.counter
        self.counter = (self.counter + 1) % self.period_length
        return c


@dataclass
class Bracketed:
    signal: MultiGoertzel
    bit_width: InitVar[int]
    preamble: list[bool] = field(default_factory=lambda: [True, False, True, False])
    power_threshold: float = field(default=50)
    buffer: deque[bool] = field(init=False)
    nco: NumericallyControlledOscillator = field(init=False)
    preamble_matched: bool = field(init=False, default=False)

    def reset(self) -> None:
        self.signal.reset()

    def _is_after_preamble(self, sample: Snapshot) -> bool:
        # Early return if the preamble has been found
        if self.preamble_matched:
            return True

        # Proceed only if the signal power is large enough
        total_power = sample.total_power()
        if total_power > self.power_threshold:
            # At the end of the clock period, classify the signal as a bit, and
            # search for the preamble
            if self.nco.is_at_end():
                mark_power, space_power = tuple(islice(sample.power(), 2))
                power_delta = mark_power - space_power
                bit_candidate = power_delta > 0
                self.buffer.append(bit_candidate)

                # If the preamble was found in its entirety
                if all(l == r for l, r in zip(self.buffer, self.preamble)):
                    print(f'[{sample.index}] [{total_power=}, {power_delta=}, {bit_candidate=}] Preamble matched! Data starts next.')
                    self.preamble_matched = True

                # In any case, reset the filter state
                print(f'[{sample.index}] [{total_power=}, {power_delta=}, {bit_candidate=}, buffer={self.buffer}] Resetting the filter state')
                self.signal.reset()

            # Increment the counter only if the signal power is large enough
            next(self.nco)
            return self.preamble_matched

        return False

    def __post_init__(self, w: int) -> None:
        self.buffer = deque([], maxlen=len(self.preamble))
        self.nco = NumericallyControlledOscillator(w)

    def __iter__(self) -> 'Bracketed':
        return self

    def __next__(self) -> Snapshot:
        snapshot = next(self.signal)

        while not self._is_after_preamble(snapshot):
            snapshot = next(self.signal)

        return snapshot


def samples_per_bit(sr: int, f: int, n: int) -> int:
    return (n * sr) // f


@dataclass
class Spec:
    sample_rate: int
    mark_frequency: int
    mark_num_periods: int
    mark_power_threshold_db: float
    space_frequency: int
    space_num_periods: int
    space_power_threshold_db: float
    low: int
    high: int

    @classmethod
    def with_kcs(cls) -> 'Spec':
        return Spec(9600, 2400, 8, 44, 1200, 4, -44, -127, 127)

    def bit_width(self) -> int:
        return samples_per_bit(self.sample_rate, self.mark_frequency, self.mark_num_periods)


def to_nrz(bit: bool) -> float:
    return float(bit) * 2 - 1


def modulate_sample(a: float, cw: float, dw: float, t: float, dt: float) -> float:
    return a * cos(cw * t + dw * dt)


def interpolate(steps, data: Sequence[float]) -> Generator[float, None, None]:
    n = steps * len(data)
    yield 0
    for i in range(1, n):
        index = int(ceil((i + 1) / steps)) - 1
        index_prev = int(ceil(i / steps)) - 1
        yield (data[index_prev] + data[index]) / 2


def integrate(data: Iterable[float]) -> Generator[float, None, None]:
    m = 0.0
    for d in data:
        m += d
        yield m


def modulate(spec: Spec, data: Iterable[bool]) -> Generator[int, None, None]:
    # Amplitude settings
    amplitude = (spec.high - spec.low) / 2

    # Frequency settings
    carrier_freq = (spec.mark_frequency + spec.space_frequency) // 2
    delta_freq = abs(spec.mark_frequency - spec.space_frequency) // 2
    sample_rate = spec.sample_rate
    carrier_omega = tau * (carrier_freq / sample_rate)
    delta_omega = tau * (delta_freq / sample_rate)

    window_size = spec.bit_width()

    nrz_data = (to_nrz(bit) for bit in data)
    interpolated_data = interpolate(window_size, list(nrz_data))
    integrated_data = integrate(interpolated_data)
    for i, m in enumerate(integrated_data):
        y = modulate_sample(amplitude, carrier_omega, delta_omega, float(i), m)
        yield int(y)


def center(signal: ndarray) -> ndarray:
    dc_offset = sum(signal) / len(signal)
    return signal - dc_offset


def normalize(signal: ndarray) -> ndarray:
    max_amplitude = max(abs(signal))
    return signal / max_amplitude


@fixture
def f_spec() -> Spec:
    return Spec.with_kcs()


@fixture
def f_preamble() -> ndarray:
    return array([True, False, True, False, True, False])


@fixture
def f_rng() -> NpGenerator:
    return default_rng(0)


@fixture(params=[1, 6, 32, 128])
def f_bits(f_rng: NpGenerator, request: FixtureRequest) -> ndarray:
    return f_rng.choice([np.bool(True), np.bool(False)], size=request.param)


class FModulated(NamedTuple):
    bits: ndarray
    preamble: ndarray
    modulated: ndarray


@fixture
def f_modulated(f_spec: Spec, f_preamble: ndarray, f_bits: ndarray) -> FModulated:
    modulated = np.array(list(modulate(f_spec, np.concat([f_preamble, f_bits]))))
    return FModulated(f_bits, f_preamble, modulated)


@fixture(params=[0, 20, 40])
def f_padded(f_modulated: FModulated, request: FixtureRequest) -> FModulated:
    bits, preamble, modulated = f_modulated
    padding: int = request.param
    s = concat([zeros((padding,)), modulated, zeros((padding,))])
    return FModulated(bits, preamble, s)


@fixture(params=[0.0, 0.1, 0.2])
def f_noisy(f_rng: NpGenerator, f_padded: FModulated, request: FixtureRequest) -> FModulated:
    bits, preamble, s = f_padded
    noise_weight: float = request.param
    n = noise_weight * f_rng.standard_normal(len(s))
    return FModulated(bits, preamble, s + n)


@fixture
def f_cleaned(f_noisy: FModulated) -> FModulated:
    bits, preamble, s = f_noisy
    return FModulated(bits, preamble, normalize(center(s))) 


def test_state_constructor() -> None:
    s = State(1, 2)
    assert s.d1 == 1 and s.d2 == 2


def test_multi_goertzel_constructor() -> None:
    s = MultiGoertzel([], 9600, [2400, 1200])
    assert s.omega[0].real == approx(2 * cos(tau * 2400 / 9600))
    assert s.omega[0].imag == approx(sin(tau * 2400 / 9600))
    assert s.omega[1].real == approx(2 * cos(tau * 1200 / 9600))
    assert s.omega[1].imag == approx(sin(tau * 1200 / 9600))
    assert all(state.d1 == 0 and state.d2 == 0 for state in s.state)


def test_interpolate() -> None:
    assert list(interpolate(2, list(range(0, 4)))) == [0, 0, 0.5, 1, 1.5, 2, 2.5, 3]


def test_integrate() -> None:
    assert list(integrate([0, 3, 2])) == [0, 3, 5]


def test_modulate(f_spec: Spec) -> None:
    assert len(list(modulate(f_spec, [True, False]))) % f_spec.bit_width() == 0


@mark.parametrize('w,n', [
    (0.1, 128),
    (0.2, 256),
    (1.0, 256),
])
def test_bracketed_only_noise(f_rng: NpGenerator, w: float, n: int) -> None:
    noise = w * f_rng.standard_normal(n)
    signal = list(Bracketed(MultiGoertzel(noise, 9600, [2400, 1200]), 32))
    assert len(signal) == 0

