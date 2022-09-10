import gdb.printing


class StringPrinter:
    def __init__(self, val):
        self.val = val

    def to_string(self):
        length = int(self.val["vec"]["len"])
        s = [
            chr(int(self.val["vec"]["buf"]["ptr"]["pointer"]["pointer"][i]))
            for i in range(length)
        ]
        return '"' + "".join(s) + '"'


class OptionPrinter:
    def __init__(self, val):
        self.val = val

    def to_string(self):
        try:
            return f'{self.val["Some"]["__0"]}'
        except:
            return "None"


class VecPrinter:
    def __init__(self, val):
        self.val = val

    def to_string(self):
        length = int(self.val["len"])
        els = [
            str(self.val["buf"]["ptr"]["pointer"]["pointer"][i]) for i in range(length)
        ]
        return "[" + ",".join(els) + "]"


class PassThroughPrinter:
    def __init__(self, val):
        self.val = val

    def to_string(self):
        return f"{self.val[0]}"


class SignatureItemPrinter:
    def __init__(self, val):
        self.val = val

    def to_string(self):
        try:
            return f'{self.val["Item"]["__0"]}'
        except:
            return f'{self.val["Folder"]["__0"]["name"]}'


class OrientationPrinter:
    def __init__(self, val):
        self.val = val

    def to_string(self):
        variant = self.val.format_string(raw=True).split("::")[-1]
        return (
            variant.replace("Positive", "+")
            .replace("Negative", "-")
            .replace("Zero", "0")
        )


class GeneratorInfoPrinter:
    def __init__(self, val):
        self.val = val

    def to_string(self):
        if bool(self.val["oriented"]):
            O = "O"
        else:
            O = ""
        if bool(self.val["invertible"]):
            I = "I"
        else:
            I = ""
        return f'{self.val["name"]}, {self.val["generator"]}{O}{I}'

    def children(self):
        return [("diagram", self.val["diagram"])]


class GeneratorPrinter:
    def __init__(self, val):
        self.val = val

    def to_string(self):
        return f'({self.val["id"]}:{self.val["dimension"]}:{self.val["orientation"]})'


class CospanPrinter:
    def __init__(self, val):
        self.val = val

    def to_string(self):
        return "Cospan"

    def children(self):
        return [("fwd", self.val["forward"]), ("bwd", self.val["backward"])]


class RewritePrinter:
    def __init__(self, val):
        self.val = val

    def to_string(self):
        try:
            return f'{self.val["Rewrite0"]["__0"]}'
        except:
            return f'{self.val["RewriteN"]["__0"]["__0"]}'


class Rewrite0Printer:
    def __init__(self, val):
        self.val = val

    def to_string(self):
        return "R0"

    def children(self):
        return [
            ("src", self.val["__0"]),
            ("tgt", self.val["__0"]),
        ]


class RewriteInternalPrinter:
    def __init__(self, val):
        self.val = val

    def to_string(self):
        return "RInt"

    def children(self):
        return [
            ("dim", self.val["dimension"]),
            ("cones", self.val["cones"]),
        ]


class ConePrinter:
    def __init__(self, val):
        self.val = val

    def to_string(self):
        return "Cone"

    def children(self):
        return [
            ("idx", self.val["index"]),
            ("int", self.val["internal"]),
        ]


class ConeInternalPrinter:
    def __init__(self, val):
        self.val = val

    def to_string(self):
        return "ConeInt"

    def children(self):
        return [
            ("src", self.val["source"]),
            ("tgt", self.val["target"]),
            ("slices", self.val["slices"]),
        ]


class DiagramPrinter:
    def __init__(self, val):
        self.val = val

    def to_string(self):
        try:
            return f'{self.val["Diagram0"]["__0"]}'
        except:
            return f'{self.val["DiagramN"]["__0"]["__0"]}'


class DiagramInternalPrinter:
    def __init__(self, val):
        self.val = val

    def to_string(self):
        return "DiagInt"

    def children(self):
        return [
            ("src", self.val["source"]),
            ("cospans", self.val["cospans"]),
        ]


class LabelPrinter:
    def __init__(self, val):
        self.val = val

    def to_string(self):
        return f'Label({self.val["__0"]})'


class SliceIndexPrinter:
    def __init__(self, val):
        self.val = val

    def to_string(self):
        return f'{self.val["Boundary"]["__0"]}'


class BoundaryPrinter:
    def __init__(self, val):
        self.val = val

    def to_string(self):
        return self.val.format_string(raw=True)[
            len("homotopy_core::common::Boundary::") :
        ]


class BoundaryPathPrinter:
    def __init__(self, val):
        self.val = val

    def to_string(self):
        return f'({self.val["__0"]},{self.val["__1"]})'


class HConsedPrinter:
    def __init__(self, val):
        self.val = val

    def to_string(self):
        return f'{self.val["elm"]["ptr"]["pointer"]["data"]}'


class NodePrinter:
    def __init__(self, val):
        self.val = val

    def to_string(self):
        return f'{self.val["__0"]}'


class NodeDataPrinter:
    def __init__(self, val):
        self.val = val

    def to_string(self):
        return f'{self.val["data"]}'


class TreePrinter:
    def __init__(self, val):
        self.val = val

    def to_string(self):
        return "Tree"

    def children(self):
        length = int(self.val["nodes"]["raw"]["len"])
        nodes = [
            (str(i), self.val["nodes"]["raw"]["buf"]["ptr"]["pointer"]["pointer"][i])
            for i in range(length)
        ]
        return [
            ("len", length),
            ("root", f'{self.val["root"]}'),
        ] + nodes


def build_pretty_printer():
    pp = gdb.printing.RegexpCollectionPrettyPrinter("homotopy-rs")
    pp.add_printer("String", "^alloc::string::String$", StringPrinter)
    pp.add_printer("Option", "^core::option::Option<.*>$", OptionPrinter)
    pp.add_printer("Vec", "^alloc::vec::Vec<.*>$", VecPrinter)
    pp.add_printer(
        "SignatureItem",
        "^homotopy_model::proof::signature::SignatureItem$",
        SignatureItemPrinter,
    )
    pp.add_printer(
        "Orientation", "^homotopy_core::common::Orientation$", OrientationPrinter
    )
    pp.add_printer(
        "GeneratorInfo",
        "^homotopy_model::proof::generators::GeneratorInfo$",
        GeneratorInfoPrinter,
    )
    pp.add_printer("Generator", "^homotopy_core::common::Generator$", GeneratorPrinter)
    pp.add_printer("Cospan", "^homotopy_core::rewrite::Cospan$", CospanPrinter)
    pp.add_printer("Rewrite", "^homotopy_core::rewrite::Rewrite$", RewritePrinter)
    pp.add_printer("Rewrite0", "^homotopy_core::rewrite::Rewrite0$", Rewrite0Printer)
    pp.add_printer(
        "RewriteInternal",
        "^homotopy_core::rewrite::RewriteInternal$",
        RewriteInternalPrinter,
    )
    pp.add_printer("Cone", "^homotopy_core::rewrite::Cone$", ConePrinter)
    pp.add_printer(
        "ConeInternal", "^homotopy_core::rewrite::ConeInternal$", ConeInternalPrinter
    )
    pp.add_printer("Diagram", "^homotopy_core::diagram::Diagram$", DiagramPrinter)
    pp.add_printer(
        "DiagramInternal",
        "^homotopy_core::diagram::DiagramInternal$",
        DiagramInternalPrinter,
    )
    pp.add_printer("Label", "^homotopy_core::rewrite::Label$", LabelPrinter)
    pp.add_printer(
        "SliceIndex", "^homotopy_core::common::SliceIndex$", SliceIndexPrinter
    )
    pp.add_printer("Boundary", "^homotopy_core::common::Boundary$", BoundaryPrinter)
    pp.add_printer(
        "BoundaryPath", "^homotopy_core::attach::BoundaryPath$", BoundaryPathPrinter
    )
    pp.add_printer("HConsed", "^hashconsing::HConsed<.*>$", HConsedPrinter)
    pp.add_printer("Node", "^homotopy_common::tree::Node$", NodePrinter)
    pp.add_printer("NodeData", "^homotopy_common::tree::NodeData<.*>$", NodeDataPrinter)
    pp.add_printer("Tree", "^homotopy_common::tree::Tree<.*>$", TreePrinter)
    return pp


gdb.printing.register_pretty_printer(
    gdb.current_objfile(), build_pretty_printer(), True
)
