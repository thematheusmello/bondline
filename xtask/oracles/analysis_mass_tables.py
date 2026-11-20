"""Audit the pinned MIT package against public CIAAW and NIST files (offline)."""
import argparse
from decimal import Decimal
import html
from pathlib import Path
import re


def package_values(root):
    numbers = dict(re.findall(r"crate::Element::(\w+) => (\d+),",
                              (root / "src/from/integer.rs").read_text()))
    weights = dict(re.findall(r"Self::(\w+) => ([\d.]+),",
                              (root / "src/standard_atomic_weight.rs").read_text()))
    isotopes = {}
    modules = re.findall(r"^pub mod (\w+);", (root / "src/isotopes.rs").read_text(), re.MULTILINE)
    assert len(modules) == 118
    for name in modules:
        # The archive also contains alternate spellings outside the compiled module tree.
        file = root / "src/isotopes" / (name + ".rs")
        code = file.read_text()
        owner = re.search(r"fn element\(&self\) -> crate::Element \{\s*crate::Element::(\w+)", code)
        assert owner is not None, name
        z = int(numbers[owner[1]])
        mass_numbers = dict(re.findall(r"Self::(\w+) => (\d+)u16,", code))
        # Examine the production mass table independently of examples and tests.
        mass_fn = code.split("fn relative_atomic_mass(&self) -> f64 {", 1)[1].split("\n}",1)[0]
        for variant, mass in re.findall(r"Self::(\w+) => ([\d.eE+-]+)f64,", mass_fn):
            key = (z, int(mass_numbers[variant]))
            assert key not in isotopes, (file,variant,key,mass,isotopes.get(key))
            isotopes[key] = Decimal(mass)
    return weights, isotopes


def check_weights(weights, document):
    checked = set()
    for _, symbol, raw in re.findall(r"<tr><td>(\d+)</td><td>(\w+)</td><td>.*?</td><td>(.*?)</td>", document):
        cell = "".join(html.unescape(re.sub(r"<[^>]+>", "", raw)).split())
        stated = re.fullmatch(r"([\d.]+)±([\d.]+)", cell)
        if stated:
            central, uncertainty = map(Decimal, stated.groups())
            assert abs(Decimal(weights[symbol]) - central) <= uncertainty, symbol
            assert symbol not in checked
            checked.add(symbol)
    assert len(checked) == 84
    print("CIAAW 2024: all 84 conventional weights within the published abridged uncertainty.")


def check_isotopes(isotopes, document):
    nist = {}
    for z, number, mass in re.findall(
        r"Atomic Number = (\d+)\s+Atomic Symbol = .*\s+Mass Number = (\d+)\s+Relative Atomic Mass = ([\d.]+)", document
    ):
        key = int(z), int(number)
        assert key not in nist
        nist[key] = Decimal(mass)
    overlap = isotopes.keys() & nist.keys()
    same = sum(abs(isotopes[key] - nist[key]) <= Decimal("1e-10") for key in overlap)
    result = (len(isotopes), len(nist), len(overlap), same, len(overlap)-same,
              len(nist.keys()-isotopes.keys()), len(isotopes.keys()-nist.keys()))
    assert result == (3414, 3352, 3264, 3129, 135, 88, 150), result
    print("NIST: package, reference, shared, matching, different, NIST-only, package-only:", *result)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("package", type=Path)
    parser.add_argument("ciaaw_html", type=Path)
    parser.add_argument("nist_ascii_html", type=Path)
    args = parser.parse_args()
    weights, isotopes = package_values(args.package)
    check_weights(weights, args.ciaaw_html.read_text())
    check_isotopes(isotopes, args.nist_ascii_html.read_text())
