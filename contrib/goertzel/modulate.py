from dataclasses import dataclass
from collections.abc import Sequence, Generator, Iterable
from math import tau, cos, ceil
import numpy as np
from numpy import ndarray, log10


def to_decibel(p: ndarray, p_0: ndarray) -> ndarray:
    return 10.0 * log10((p + 1e-10) / (p_0 + 1e-10))


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


def test_interpolate() -> None:
    assert list(interpolate(2, list(range(0, 4)))) == [0, 0, 0.5, 1, 1.5, 2, 2.5, 3]


def test_integrate() -> None:
    assert list(integrate([0, 3, 2])) == [0, 3, 5]


def test_modulate(f_spec: Spec) -> None:
    assert len(list(modulate(f_spec, [True, False]))) % f_spec.bit_width() == 0

