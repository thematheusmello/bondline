#!/usr/bin/env python3
# SPDX-License-Identifier: MIT OR Apache-2.0
"""Independently constructed ordinary cyclobutane; verify fixtures by default."""
import argparse
import hashlib
from pathlib import Path
import struct

HERE = Path(__file__).resolve().parent
IDS = [19, 2, 300, 77]
POINTS = [(128, 128), (256, 128), (256, 256), (128, 256)]


def prop(tag, payload):
    return struct.pack('<HH', tag, len(payload)) + payload


def obj(tag, identity, *body):
    return struct.pack('<HI', tag, identity) + b''.join(body) + bytes(2)


def raw(case):
    nodes = []
    for i, ((x, y), identity) in enumerate(zip(POINTS, IDS, strict=True)):
        fields = [prop(0x0200, struct.pack('<ii', y*65536, x*65536))]
        if i == 0 and case in ('active', 'dormant'):
            fields.append(prop(0x0443 if case == 'active' else 0x0442, b''))
        if i == 0 and case == 'h-inconsistent':
            fields.append(prop(0x042B, struct.pack('<H', 3)))
        # Element, NumHydrogens and NodeType are deliberately absent otherwise.
        nodes.append(obj(0x8004, identity, *fields))
    bonds = [obj(0x8005, 700+i, prop(0x0600, struct.pack('<H', 1)),
                 prop(0x0604, struct.pack('<I', IDS[i])),
                 prop(0x0605, struct.pack('<I', IDS[(i+1) % 4]))) for i in range(4)]
    page = obj(0x8001, 9001, prop(0x0811, b'\1'),
               prop(0x0812, struct.pack('<i', 512*65536)),
               prop(0x0813, struct.pack('<i', 512*65536)),
               obj(0x8003, 91, *nodes, *bonds))
    palette = prop(0x0300, struct.pack('<H6H', 2, 65535, 65535, 65535, 0, 0, 0))
    return (b'VjCD0100' + struct.pack('<I', 0x01020304) + bytes(16) + palette
            + prop(0x0807, struct.pack('<i', 4*65536))
            + prop(0x0808, struct.pack('<i', 2*65536)) + page + bytes(2))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output-dir', type=Path)
    output = parser.parse_args().output_dir
    if output:
        output.mkdir(parents=True, exist_ok=True)
    for case in ('cycle', 'dormant', 'active', 'h-inconsistent'):
        data = raw(case)
        path = (output or HERE)/(case+'.cdx')
        if output:
            path.write_bytes(data)
        else:
            assert path.read_bytes() == data, path
        print(case, len(data), hashlib.sha256(data).hexdigest())


if __name__ == '__main__':
    main()
