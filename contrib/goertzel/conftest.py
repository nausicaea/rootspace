from modulate import Spec, modulate, normalize, center
from dataclasses import dataclass
import numpy as np
from numpy import ndarray, array, concat, zeros
from numpy.random import default_rng
from numpy.random import Generator as NpGenerator
from typing import NamedTuple
from pytest import FixtureRequest, fixture


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
    modulated = array(list(modulate(f_spec, np.concat([f_preamble, f_bits]))))
    return FModulated(f_bits, f_preamble, modulated)


@fixture(params=[0, 20, 40])
def f_padded(f_modulated: FModulated, request: FixtureRequest) -> FModulated:
    bits, preamble, modulated = f_modulated
    padding: int = request.param
    s = concat([zeros((padding,)), modulated, zeros((padding,))])
    return FModulated(bits, preamble, s)


@fixture(params=[0.0, 0.1, 0.2])
def f_noisy(
    f_rng: NpGenerator, f_padded: FModulated, request: FixtureRequest
) -> FModulated:
    bits, preamble, s = f_padded
    noise_weight: float = request.param
    n = noise_weight * f_rng.standard_normal(len(s))
    return FModulated(bits, preamble, s + n)


@fixture
def f_cleaned(f_noisy: FModulated) -> FModulated:
    bits, preamble, s = f_noisy
    return FModulated(bits, preamble, normalize(center(s)))
