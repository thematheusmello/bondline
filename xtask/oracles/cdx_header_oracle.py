"""Original header-embedded Document vectors; stdlib framing and external RDKit.

Public numeric framing facts: Glysade/chemdraw BSD-3-Clause source at
5dd743230d75ecacacdf8dcd0b0e022f13173b9c, CDXIO.cpp CDXWriteHeader and
CDXDocument.cpp CDXWriteDocToStorage, plus SDK IntroCDX object/file ends.
No upstream implementation or native application metadata is copied here.
The existing original chemical scene and its old oracle guards are unchanged.
"""
import hashlib
import struct
import sys

import cdx_oracle as legacy

PREFIX_BYTES = 22


def fixture(order=1):
    original = legacy.fixture(order, explicit=False)
    # The fresh scene stays original. Only its independently specified envelope
    # changes: 22-byte prefix, encoded root, object end, then a separate file end.
    return original[:PREFIX_BYTES] + struct.pack("<HI", 0x8000, 0) + original[28:-2] + bytes(4)


def records(source):
    assert len(source) <= legacy.LIMIT and source[:12] == legacy.HEADER[:12]
    assert len(source) >= 32 and source[22:28] == struct.pack("<HI", 0x8000, 0)
    result, stack, offset = [], [], 22
    while stack or not result:
        assert len(result) < 100_000 and offset + 2 <= len(source)
        start = offset
        tag = struct.unpack_from("<H", source, offset)[0]
        offset += 2
        if tag == 0:
            assert stack, "an object end must close an encoded object"
            matched = stack.pop()
            result.append({"kind":"end", "matched":matched,
                "parent":stack[-1] if stack else None,"start":start,"end":offset})
        elif tag & 0x8000:
            assert len(stack) < 256 and offset + 4 <= len(source)
            identifier = struct.unpack_from("<I", source, offset)[0]
            offset += 4
            result.append({"kind":"start","tag":tag,"id":identifier,
                "parent":stack[-1] if stack else None,"start":start,"end":offset})
            stack.append(len(result)-1)
        else:
            assert stack and offset + 2 <= len(source)
            count = struct.unpack_from("<H", source, offset)[0]
            offset += 2
            encoding = "short"
            if count == 65535:
                assert offset + 4 <= len(source)
                count = struct.unpack_from("<I", source, offset)[0]
                offset += 4
                encoding = "extended"
            payload = offset
            offset += count
            assert offset <= len(source)
            result.append({"kind":"prop","tag":tag,"encoding":encoding,
                "payload_start":payload,"payload_end":offset,"parent":stack[-1],
                "start":start,"end":offset})
    assert len(result) < 100_000
    assert source[offset:] == bytes(2), "one distinct file end and exact EOF required"
    result.append({"kind":"file_end","parent":None,"start":offset,"end":offset+2})
    return result


def describe(source):
    output=[]
    for record in records(source):
        parent="-" if record['parent'] is None else record['parent']
        values=[record['kind']]
        if record['kind']=='start': values += [record['tag'],record['id']]
        elif record['kind']=='prop':
            values += [record['tag'],record['encoding'],record['payload_end']-record['payload_start'],record['payload_start'],record['payload_end']]
        elif record['kind']=='end': values += [record['matched']]
        values += [parent,record['start'],record['end']]
        output.append(' '.join(map(str,values)))
    return ('\n'.join(output)+'\n').encode('ascii')


def verify(source, order):
    from rdkit import Chem,rdBase
    from rdkit.Chem import rdMolDescriptors
    assert rdBase.rdkitVersion=='2026.09.2' and Chem.HasChemDrawCDXSupport()
    assert source==fixture(order), 'original framing/scene bytes changed unexpectedly'
    table=records(source)
    assert table[0]=={'kind':'start','tag':32768,'id':0,'parent':None,'start':22,'end':28}
    assert table[-1]['kind']=='file_end'
    coordinates=[struct.unpack_from('<ii',source,item['payload_start']) for item in table if item.get('tag')==0x0200]
    assert [(x/65536,y/65536) for y,x in coordinates]==[(10.25,-3.5),(26.25,8.5)]
    parameters=Chem.CDXMLParserParams()
    parameters.format=Chem.CDXMLFormat.CDX
    molecules=Chem.MolsFromCDXML(source,parameters)
    assert len(molecules)==1 and molecules[0].GetNumAtoms()==2
    molecule=molecules[0]
    assert [atom.GetAtomicNum() for atom in molecule.GetAtoms()]==[6,6]
    assert molecule.GetNumBonds()==1
    bond=molecule.GetBondWithIdx(0)
    assert (bond.GetBeginAtomIdx(),bond.GetEndAtomIdx(),bond.GetBondTypeAsDouble())==(0,1,order)
    assert Chem.MolToSmiles(molecule)==('CC' if order==1 else 'C=C')
    assert rdMolDescriptors.CalcMolFormula(molecule)==('C2H6' if order==1 else 'C2H4')
    assert [tuple(molecule.GetConformer().GetAtomPosition(index)) for index in range(2)]==[
        (671744.0,229376.0,0.0),(1720320.0,-557056.0,0.0)]
    sys.stdout.buffer.write(b'original header-document scene verified by struct and pinned RDKit\n')


def self_test():
    hashes={1:'94ecd46a9879009f1cc1dd944dea1b0a3f0fdcadfb8a09a1e788e9b234611618',
            2:'8be729d865c1d2c20776bd4ef085f19908ab3f21d48063bfbb4da74bfe10b21c'}
    for order,expected in hashes.items():
        source=fixture(order)
        assert len(source)==65746 and hashlib.sha256(source).hexdigest()==expected
        table=records(source)
        assert len(table)==30 and table[-2]['matched']==0 and table[-1]['end']==len(source)
    # The strict independent decoder must not mistake a document end for the file end.
    for changed in [fixture()[:-2],fixture()+bytes(2),fixture()+b'x']:
        try: records(changed)
        except (AssertionError,struct.error): pass
        else: raise AssertionError('accepted missing/extra file end')
    sys.stdout.buffer.write(b'header-document fixture identities and strict framing verified\n')


if __name__=='__main__':
    mode=sys.argv[1]
    if mode=='fixture': sys.stdout.buffer.write(fixture())
    elif mode=='describe': sys.stdout.buffer.write(describe(sys.stdin.buffer.read(legacy.LIMIT+1)))
    elif mode=='verify': verify(sys.stdin.buffer.read(legacy.LIMIT+1),int(sys.argv[2]))
    elif mode=='self-test': self_test()
    else: raise ValueError('fixture, describe, verify or self-test required')
