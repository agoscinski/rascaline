#!/usr/bin/env python3
# -*- coding: utf-8 -*-
import os
import sys

from pycparser import c_ast, parse_file

ROOT = os.path.dirname(__file__)
FAKE_INCLUDES = os.path.join(ROOT, "include")
RASCALINE_HEADER = os.path.relpath(
    os.path.join(ROOT, "..", "..", "rascaline-c-api", "rascaline.h")
)


class Function:
    def __init__(self, name, restype):
        self.name = name
        self.restype = restype
        self.args = []

    def add_arg(self, arg):
        self.args.append(arg)


class Struct:
    def __init__(self, name):
        self.name = name
        self.members = {}

    def add_member(self, name, type):
        self.members[name] = type


class Enum:
    def __init__(self, name):
        self.name = name
        self.values = {}

    def add_value(self, name, value):
        self.values[name] = value


class FuncDefVisitor(c_ast.NodeVisitor):
    def __init__(self):
        self.functions = []
        self.enums = []
        self.structs = []

    def visit_Decl(self, node):
        if not node.name.startswith("rascal_"):
            return

        function = Function(node.name, node.type.type)
        for parameter in node.type.args.params:
            function.add_arg(parameter.type)
        self.functions.append(function)

    def visit_Typedef(self, node):
        if not node.name.startswith("rascal_"):
            return

        # Get name and value for enum
        if isinstance(node.type.type, c_ast.Enum):
            enum = Enum(node.name)
            for enumerator in node.type.type.values.enumerators:
                enum.add_value(enumerator.name, enumerator.value.value)
            self.enums.append(enum)

        elif isinstance(node.type.type, c_ast.Struct):
            struct = Struct(node.name)
            for _, member in node.type.type.children():
                struct.add_member(member.name, member.type)

            self.structs.append(struct)


def parse(file):
    cpp_args = ["-E", "-I", FAKE_INCLUDES]
    ast = parse_file(file, use_cpp=True, cpp_path="gcc", cpp_args=cpp_args)

    v = FuncDefVisitor()
    v.visit(ast)
    return v


def c_type_name(name):
    if name.startswith("rascal_"):
        # enums are represente as int
        if name == "rascal_indexes":
            return "ctypes.c_int"
        else:
            return name
    elif name == "uintptr_t":
        return "c_uintptr_t"
    elif name == "void":
        return "None"
    else:
        return "ctypes.c_" + name


def _typedecl_name(type):
    assert isinstance(type, c_ast.TypeDecl)
    if isinstance(type.type, c_ast.Struct):
        return type.type.name
    elif isinstance(type.type, c_ast.Enum):
        return type.type.name
    else:
        assert len(type.type.names) == 1
        return type.type.names[0]


def type_to_ctypes(type, ndpointer=False):
    if isinstance(type, c_ast.PtrDecl):
        if isinstance(type.type, c_ast.PtrDecl):
            if isinstance(type.type.type, c_ast.TypeDecl):
                name = _typedecl_name(type.type.type)
                if name == "char":
                    return "POINTER(ctypes.c_char_p)"

                name = c_type_name(name)
                if ndpointer:
                    return f"POINTER(ndpointer({name}, flags='C_CONTIGUOUS'))"
                else:
                    return f"POINTER(POINTER({name}))"

        elif isinstance(type.type, c_ast.TypeDecl):
            name = _typedecl_name(type.type)
            if name == "void":
                return "ctypes.c_void_p"
            elif name == "char":
                return "ctypes.c_char_p"
            else:
                return f"POINTER({c_type_name(name)})"

        elif isinstance(type.type, c_ast.FuncDecl):
            restype = type_to_ctypes(type.type.type, ndpointer)
            args = [type_to_ctypes(t.type, ndpointer) for t in type.type.args.params]

            return f'CFUNCTYPE({restype}, {", ".join(args)})'

    else:
        # not a pointer
        if isinstance(type, c_ast.TypeDecl):
            return c_type_name(_typedecl_name(type))
        if isinstance(type, c_ast.ArrayDecl):
            if isinstance(type.dim, c_ast.Constant):
                size = type.dim.value
            else:
                raise Exception("dynamically sized arrays are not supported")

            return f"{type_to_ctypes(type.type)} * {size}"

    raise Exception("Unknown type")


def generate_enums(file, enums):
    for enum in enums:
        file.write(f"\n\nclass {enum.name}(enum.Enum):\n")
        for name, value in enum.values.items():
            file.write(f"    {name} = {value}\n")


def generate_structs(file, structs):
    for struct in structs:
        file.write(f"\n\nclass {struct.name}(ctypes.Structure):\n")
        if len(struct.members) == 0:
            file.write("    pass\n")
            continue

        file.write("    _fields_ = [\n")
        for name, type in struct.members.items():
            file.write(f'        ("{name}", {type_to_ctypes(type, True)}),\n')
        file.write("    ]\n")


def generate_functions(file, functions):
    file.write(f"\n\ndef setup_functions(lib):\n")
    file.write("    from .status import _check_rascal_status_t\n")

    for function in functions:
        file.write(f"\n    lib.{function.name}.argtypes = [\n        ")
        args = [type_to_ctypes(arg) for arg in function.args]

        # functions taking void parameter in C don't have any parameter
        if args == ["None"]:
            args = []
        file.write(",\n        ".join(args))
        file.write("\n    ]\n")

        restype = type_to_ctypes(function.restype)
        if restype == "rascal_status_t":
            restype = "_check_rascal_status_t"

        file.write(f"    lib.{function.name}.restype = {restype}\n")


def generate_declarations():
    data = parse(RASCALINE_HEADER)

    outpath = os.path.join(ROOT, "..", "rascaline", "_rascaline.py")
    with open(outpath, "w") as file:
        file.write(
            """# -*- coding: utf-8 -*-
'''
Automatically-generated file, do not edit!!!
'''
# flake8: noqa

import enum
import platform

import ctypes
from ctypes import POINTER, CFUNCTYPE
from numpy.ctypeslib import ndpointer

arch = platform.architecture()[0]
if arch == "32bit":
    c_uintptr_t = ctypes.c_uint32
elif arch == "64bit":
    c_uintptr_t = ctypes.c_uint64
"""
        )
        generate_enums(file, data.enums)
        generate_structs(file, data.structs)
        generate_functions(file, data.functions)


if __name__ == "__main__":
    generate_declarations()
