#!/usr/bin/env python3
# -*- coding: utf-8 -*-

import collections
import logging
import re
import uuid
import warnings
import weakref
from typing import Optional, Any, Union

import attr

FIRST_CAP_RE = re.compile(r"(.)([A-Z][a-z]+)")
ALL_CAP_RE = re.compile(r"([a-z0-9])([A-Z])")
Loggers = collections.namedtuple("Loggers", ("project", "py_warnings"))


def underscore_to_camelcase(name: str) -> str:
    """
    Convert underscored_text to CamelCase text.

    :param str name:
    :return:
    """
    return "".join(x.capitalize() or "_" for x in name.split("_"))


def camelcase_to_underscore(name: str) -> str:
    """
    Convert CamelCase text to underscored_text.

    :param str name:
    :return:
    """
    s1 = FIRST_CAP_RE.sub(r"\1_\2", name)
    return ALL_CAP_RE.sub(r"\1_\2", s1).lower()


def to_ref(value: Any) -> Optional[weakref.ReferenceType]:
    """
    Convert the input value using weakref.ref. This function is idempotent and
    passes None unmodified.

    :param value:
    :return:
    """
    if isinstance(value, weakref.ReferenceType):
        return value
    elif value is None:
        return None
    else:
        return weakref.ref(value)


def to_uuid(value: Optional[Union[str, uuid.UUID]]) -> Optional[uuid.UUID]:
    """
    Convert the input to a UUID value. This function is idempotent and passes None unmodified.

    :param value:
    :return:
    """
    if isinstance(value, uuid.UUID):
        return value
    elif isinstance(value, str):
        return uuid.UUID(value)
    elif value is None:
        return None
    else:
        raise TypeError("Expected 'value' to be either a UUID, a string or None.")


def get_log_level(verbosity: int) -> int:
    """
    Determine the logging level from the verbose and debug flags.
    """
    if verbosity <= 0:
        log_level = logging.ERROR
    elif verbosity == 1:
        log_level = logging.WARN
    elif verbosity == 2:
        log_level = logging.INFO
    elif verbosity >= 3:
        log_level = logging.DEBUG
    else:
        raise ValueError("Only four verbosity levels are understood: 0, 1, 2 and 3.")

    return log_level


def configure_logger(name: str, log_level: int, log_path: Optional[str] = None, with_warnings: Optional[bool] = True) -> Loggers:
    """
    Configure the project logger of the specified name
    using colorlog.

    :param name:
    :param log_level:
    :param log_path:
    :param with_warnings:
    :return:
    """
    default_handler = None
    if log_path is not None:
        default_handler = logging.FileHandler(log_path)
        default_handler.setLevel(log_level)
        plain_formatter = logging.Formatter(
            "{levelname:8s} @{name}: {message}",
            style="{"
        )
        default_handler.setFormatter(plain_formatter)
    else:
        default_handler = logging.StreamHandler()
        default_handler.setLevel(log_level)
        plain_formatter = logging.Formatter(
            "{levelname:8s} @{name}: {message}",
            style="{"
        )
        default_handler.setFormatter(plain_formatter)

    # Configure the rootspace logger
    project_logger = logging.getLogger(name)
    project_logger.setLevel(log_level)
    project_logger.addHandler(default_handler)

    py_warnings = None
    if with_warnings:
        # Configure the warnings logger
        warnings.simplefilter("default")
        logging.captureWarnings(True)
        py_warnings = logging.getLogger("py.warnings")
        py_warnings.setLevel(log_level)
        py_warnings.addHandler(default_handler)

    return Loggers(project_logger, py_warnings)


@attr.s(repr=False, slots=True)
class SubclassValidator(object):
    cls = attr.ib()

    def __call__(self, instance, attribute, value):
        if not issubclass(value, self.cls):
            raise TypeError(
                "'{name}' must be {cls!r} (got {value!r} that is a "
                "{actual!r})."
                    .format(name=attribute.name, cls=self.cls,
                            actual=value.__class__, value=value),
                attribute, self.cls, value
            )

    def __repr__(self):
        return (
            "<subclass_of validator for class {cls!r}>"
                .format(cls=self.cls)
        )


def subclass_of(cls):
    """
    Return a validator that evaluates issubclass(.) on
    the supplied attribute. To be used with attrs.

    :param cls:
    :return:
    """
    return SubclassValidator(cls)


@attr.s(repr=False, slots=True)
class IterableValidator(object):
    cls_container = attr.ib()
    cls_element = attr.ib()

    def __call__(self, instance, attribute, value):
        if not (isinstance(value, self.cls_container) and all(isinstance(el, self.cls_element) for el in value)):
            raise TypeError(
                "'{name}' must be {cls_container!r} and elements thereof must be {cls_element!r} (got {value!r})."
                .format(name=attribute.name, cls_container=self.cls_container, cls_element=self.cls_element,
                        value=value),
                attribute, self.cls_container, self.cls_element, value
            )

    def __repr__(self):
        return (
            "<iterable_of validator for type {clsc!r} and {clse!r}>"
                .format(clsc=self.cls_container, clse=self.cls_element)
        )


def iterable_of(container, element):
    """
    Return a validator that evaluates isinstance(.) on the supplied attribute and its elements.
    To be used with attrs.

    :param container:
    :param element:
    :return:
    """
    return IterableValidator(container, element)