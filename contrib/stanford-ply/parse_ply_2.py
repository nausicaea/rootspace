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

number = pp.pyparsing_common.number()
identifier = pp.pyparsing_common.identifier()
lit = {l: pp.CaselessKeyword(l) for l in (
    "ascii", "binary_little_endian", "binary_big_endian", "vertex", "face", "edge", "char",
    "uchar", "short", "ushort", "int", "uint", "float", "double", "format", "comment",
    "element", "property", "list", "ply", "end_header"
)}

format_type = lit["ascii"] | lit["binary_little_endian"] | lit["binary_big_endian"]
element_type = lit["vertex"] | lit["face"] | lit["edge"] | identifier
property_type = lit["char"] | lit["uchar"] | lit["short"] | lit["ushort"] | lit["int"] | lit["uint"] | lit["float"] | lit["double"]

format_decl = pp.Suppress(lit["format"]) + format_type("file_type") + number("version")
comment_decl = lit["comment"] + pp.restOfLine
element_decl = pp.Suppress(lit["element"]) + element_type("name") + number("number")
property_decl = pp.Suppress(lit["property"]) + (
    (property_type("data_type") + identifier("name")) |
    (pp.Suppress(lit["list"]) + property_type("idx_type") + property_type("data_type") + identifier("name"))
)

declarations = format_decl("format") + pp.OneOrMore(pp.Group(
     element_decl + pp.OneOrMore(pp.Group(property_decl))("properties")
))("declarations")
# header = pp.Suppress(lit["ply"]) + declarations + pp.Suppress(lit["end_header"])
header = pp.Suppress(lit["ply"]) + declarations + pp.Suppress(lit["end_header"])

body = pp.OneOrMore(number)
body = pp.Forward()

ply_grammar = pp.Group(header)("header") + body("body")

def construct_body_expr(s,l,t):
    body_expr = []
    for decl in t.declarations:
        if decl.name == 'vertex':
            props = decl.properties
            body_expr.append(pp.Group(
                pp.Group(number(props[0].name) + number(props[1].name) + number(props[2].name)) * decl.number
            )(decl.name))
        elif decl.name == 'face':
            body_expr.append(pp.Group(
                pp.countedArray(number) * decl.number
            )(decl.name))
    body << pp.Group(pp.And(body_expr))

header.addParseAction(construct_body_expr)


ply_grammar.ignore(comment_decl)


def main():
    data = """ply
format ascii 1.0
comment made by Greg Turk
comment this file is a cube
element vertex 8
property float x
property float y
property float z
element face 6
property list uchar int vertex_index
end_header
0 0 0
0 0 1
0 1 1
0 1 0
1 0 0
1 0 1
1 1 1
1 1 0
4 0 1 2 3
4 7 6 5 4
4 0 4 5 1
4 1 5 6 2
4 2 6 7 3
4 3 7 4 0
"""
    try:
        ply_grammar.validate()
        tokens = ply_grammar.parseString(data)
        # tokens.pprint()
        print(tokens.dump())
        print(len(tokens.body))
    except pp.ParseException as e:
        print(e.line)
        print(" " * (e.column - 1) + "^")
        print(e)


if __name__ == "__main__":
    main()
