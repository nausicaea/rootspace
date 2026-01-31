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


def to_decibel(p: np.ndarray, p_0: np.ndarray) -> np.ndarray:
    return 10.0 * np.log10((p + 1e-10) / (p_0 + 1e-10))


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
class Tracker:
    _attrs: dict[str, list[Any]] = field(default_factory=dict)

    def insert(self, key: str, value: Any) -> None:
        if key not in self._attrs:
            self._attrs[key] = list()
        self._attrs[key].append(value)

    def get(self, key: str) -> ndarray:
        return array(self._attrs[key])


def classify(power_delta: float, threshold: float) -> bool:
    return power_delta > threshold


def classify_with_hysteresis(prev_bit: bool, power_delta: float, threshold: float, hysteresis: float) -> bool:
    if not prev_bit and power_delta > (threshold + hysteresis):
        return True
    elif prev_bit and power_delta < (threshold - hysteresis):
        return False
    else:
        return prev_bit


@dataclass
class WithPreamble:
    signal: MultiGoertzel
    bit_width: InitVar[int]
    preamble: list[bool] = field(default_factory=lambda: [True, False, True, False])
    total_power_threshold: float = field(default=25)
    delta_power_threshold: float = field(default=0)
    delta_power_hysteresis: float = field(default=5)
    tracker: Tracker | None = field(default=None)
    buffer: deque[bool] = field(init=False)
    nco: NumericallyControlledOscillator = field(init=False)
    preamble_matched: bool = field(init=False, default=False)

    def reset(self) -> None:
        self.signal.reset()

    def _track(self, key: str, value_fn: Callable[[], Any]) -> None:
        if self.tracker is not None:
            self.tracker.insert(key, value_fn())

    def _is_after_preamble(self, sample: Snapshot) -> bool:
        self._track('nco', lambda: self.nco.counter)
        self._track('buffer', lambda: list(self.buffer))
        self._track('preamble_matched', lambda: self.preamble_matched)
        self._track('power', lambda: list(sample.power()))
        self._track('total_power', lambda: sample.total_power())

        # Early return if the preamble has been found
        if self.preamble_matched:
            return True

        # At the middle of the clock period, classify the signal as a bit, and
        # search for the preamble. Proceed only if the signal power is large enough.
        total_power = sample.total_power()
        if self.nco.is_at_half() and total_power > self.total_power_threshold:
            mark_power, space_power = tuple(islice(sample.power(), 2))
            power_delta = mark_power - space_power
            bit_candidate = classify_with_hysteresis(self.buffer[-1], power_delta, self.delta_power_threshold, self.delta_power_hysteresis)
            self._track('bit_candidate', lambda: bit_candidate)
            self.buffer.append(bit_candidate)

            # If the preamble was found in its entirety
            if all(l == r for l, r in zip(self.buffer, self.preamble)):
                self.preamble_matched = True

        if self.nco.is_at_end():
            # In any case, reset the filter state
            self.signal.reset()

        # Increment the counter always
        next(self.nco)

        return self.preamble_matched

    def __post_init__(self, w: int) -> None:
        self.buffer = deque(list(repeat(False, len(self.preamble))), maxlen=len(self.preamble))
        self.nco = NumericallyControlledOscillator(w)

    def __iter__(self) -> 'WithPreamble':
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


def difference(a: ndarray, total: ndarray) -> ndarray:
    return (2 * a.T - total).T


def ratio(a: ndarray, total: ndarray) -> ndarray:
    return to_decibel(a.T, total - a.T).T


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


def plot(filename: str, signal: ndarray, output: Iterable[Snapshot], tracker: Tracker) -> None:
    fig = plt.figure(figsize=(10,12), layout='constrained')
    axs = fig.subplot_mosaic([['signal'], ['output'], ['nco'], ['buffer'], ['preamble_matched'], ['power'], ['total_power'], ['power_delta'], ['power_ratio'], ['bit_candidate']])
    axs['signal'].set_title('Original Signal (normalized)')
    axs['signal'].plot(signal)
    axs['output'].set_title('WithPreamble Output Power')
    axs['output'].plot(array([tuple(o.power()) for o in output]))
    axs['nco'].set_title('NCO Clock State')
    axs['nco'].plot(tracker.get('nco'))
    axs['buffer'].set_title('Preamble Buffer')
    axs['buffer'].imshow(tracker.get('buffer').T, cmap=ListedColormap(['red', 'green']), origin='lower', aspect='auto', interpolation='nearest')
    axs['preamble_matched'].set_title('Preamble Matched')
    axs['preamble_matched'].plot(tracker.get('preamble_matched'))
    axs['power'].set_title('Post-Goertzel Signal Power')
    axs['power'].plot(tracker.get('power'))
    axs['total_power'].set_title('Total Power')
    axs['total_power'].plot(tracker.get('total_power'))
    axs['power_delta'].set_title('Power Delta')
    axs['power_delta'].plot(difference(tracker.get('power'), tracker.get('total_power')))
    axs['power_ratio'].set_title('Power Ratio')
    axs['power_ratio'].plot(ratio(tracker.get('power'), tracker.get('total_power')))
    axs['bit_candidate'].set_title('Bit Candidate')
    axs['bit_candidate'].plot(tracker.get('bit_candidate'))
    plt.savefig(filename)


@mark.parametrize('n', [128, 256])
def test_with_preamble_ly_noise(f_rng: NpGenerator, n: int) -> None:
    """
    The preamble shall not be detected in a noise-only signal.
    """
    tracker = Tracker()
    signal = f_rng.standard_normal(n)
    output = list(WithPreamble(MultiGoertzel(signal, 9600, [2400, 1200]), 32, tracker=tracker))
    filename = f'with-preamble-only-noise-{n}len.png'
    plot(filename, signal, output, tracker)

    assert (tracker.get('preamble_matched') == False).all()
    assert (tracker.get('buffer') != f_preamble).all()


@mark.parametrize('p,w', [
    (0, 0.0), 
    (20, 0.0), 
    (40, 0.0),
    (0, 0.1), 
    (20, 0.1), 
    #(40, 0.1),
])
def test_with_preamble_only_preamble(f_spec: Spec, f_rng: NpGenerator, f_preamble: ndarray, p: int, w: float) -> None:
    """
    The preamble must be detected if it is present.
    """

    tracker = Tracker()
    modulated = np.array(list(modulate(f_spec, f_preamble)))
    s = concat([zeros((p,)), modulated, zeros((p,))])
    n = f_rng.standard_normal(len(s))
    signal = normalize(center(s + w * n))
    output = list(WithPreamble(
        MultiGoertzel(signal, f_spec.sample_rate, [f_spec.mark_frequency, f_spec.space_frequency]),
        f_spec.bit_width(),
        preamble=[bool(v) for v in f_preamble],
        tracker=tracker,
    ))

    pmble = "".join("1" if v else "0" for v in f_preamble)
    filename = f'with-preamble-only-preamble-{pmble}preamble-{p}padding-{w}noise.png'

    plot(filename, signal, output, tracker)

    assert tracker.get('preamble_matched')[-1] == True
    assert (tracker.get('buffer')[-1,:] == f_preamble).all()

