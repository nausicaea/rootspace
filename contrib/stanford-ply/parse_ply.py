# -*- coding: utf-8 -*-

"""
Parse PLY Stanford Polygon Files
--------------------------------

Context free grammar:
ply_grammar     ::= header body
header          ::= "ply" declaration+ "end_header"
declaration     ::= format | element | property
format          ::= "format" format_type NUMBER
element         ::= "element" element_type NUMBER
property        ::= ("property" property_type IDENT) | ("property" "list" property_type property_type IDENT)
format_type     ::= "ascii" | "binary_little_endian" | "binary_big_endian"
element_type    ::= "vertex" | "face" | "edge" | IDENT
property_type   ::= "char" | "uchar" | "short" | "ushort" | "int" | "uint" | "float" | "double"
body            ::= statement+
statement       ::= NUMBER+
"""

import pathlib

import pyparsing as pp
import numpy


def all_equal(iterable):
    """
    Return True if all elements of the iterable are equal or if the iterable is emtpy.

    :param iterable:
    :return:
    """
    return len(set(iterable)) <= 1


def create_ply_parser():
    data_types = {
        "char": numpy.int8,
        "uchar": numpy.uint8,
        "short": numpy.int16,
        "ushort": numpy.uint16,
        "int": numpy.int32,
        "uint": numpy.uint32,
        "float": numpy.float,
        "double": numpy.double
    }

    number = pp.pyparsing_common.number()
    identifier = pp.pyparsing_common.identifier()
    lit = {l: pp.CaselessKeyword(l) for l in (
        "ascii", "binary_little_endian", "binary_big_endian", "vertex", "face", "edge", "char",
        "uchar", "short", "ushort", "int", "uint", "float", "double", "format", "comment",
        "element", "property", "list", "ply", "end_header"
    )}

    format_type = lit["ascii"] | lit["binary_little_endian"] | lit["binary_big_endian"]
    element_type = lit["vertex"] | lit["face"] | lit["edge"] | identifier
    data_type = pp.Or((lit[k].addParseAction(pp.replaceWith(v)) for k, v in data_types.items()))

    comment_decl = lit["comment"] + pp.restOfLine

    format_decl = pp.Group(
        pp.Suppress(lit["format"]) +
        format_type("file_type") +
        number("version")
    )("format")

    property_simple = pp.Group(
        pp.Suppress(lit["property"]) +
        data_type("data_type") +
        identifier("name")
    )("simple")

    property_list = pp.Group(
        pp.Suppress(lit["property"]) + pp.Suppress(lit["list"]) +
        data_type("index_type") +
        data_type("data_type") +
        identifier("name")
    )("list")

    element_decl = pp.Group(
        pp.Suppress(lit["element"]) +
        element_type("name") +
        number("count") +
        pp.Group(
            pp.OneOrMore(property_simple) | property_list
        )("properties")
    )

    declarations = pp.Group(
        format_decl +
        pp.Group(
            pp.OneOrMore(element_decl)
        )("elements")
    )("declarations")

    header_start = pp.Suppress(lit["ply"])

    header_stop = pp.Suppress(lit["end_header"])

    header = header_start + declarations + header_stop

    body = pp.Forward()("data")

    ply_grammar = (header + body).ignore(comment_decl)

    def construct_body_expr(source, location, tokens):
        body_expr = list()
        for el_decl in tokens.declarations.elements:
            if el_decl.properties[0].getName() == "list":
                structure = el_decl.properties[0].data_type
                element = pp.countedArray(number)
            else:
                structure = [(p.name, p.data_type) for p in el_decl.properties]
                element = pp.Group(
                    pp.And(number(p.name) for p in el_decl.properties)
                )

            body_expr.append(pp.Group(element * el_decl.count)(el_decl.name))

        body << pp.And(body_expr)

    header.addParseAction(construct_body_expr)

    return ply_grammar


def main():
    file_paths = pathlib.Path(".").glob("*.ply")
    for file_path in file_paths:
        try:
            ply_grammar = create_ply_parser()
            tokens = ply_grammar.parseString(file_path.read_text())
            #tokens.pprint()
            print(tokens.dump())
            #print(len(tokens.body))
        except pp.ParseException as e:
            print(e.line)
            print(" " * (e.column - 1) + "^")
            print(e)


if __name__ == "__main__":
    main()
