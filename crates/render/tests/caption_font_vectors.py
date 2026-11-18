"""Independent, manually run OpenType vector extraction; no renderer imports."""
import hashlib
import json
import sys
from pathlib import Path

import fontTools
from fontTools.ttLib import TTFont

assert fontTools.__version__ == "4.61.1", fontTools.__version__
source = Path(sys.argv[1])
assert hashlib.sha256(source.read_bytes()).hexdigest() == (
    "56a45233d29f11b4dfb86d248e921939d115778f87325e7ae8cc108383d6664d"
)
font = TTFont(source)
cmap = font.getBestCmap()
characters = "AV ToHgλΩé\u00a0"
metrics = {}
for char in characters:
    name = cmap[ord(char)]
    glyph = font["glyf"][name]
    advance, lsb = font["hmtx"][name]
    metrics[char] = {
        "glyph_id": font.getGlyphID(name),
        "advance": advance,
        "lsb": lsb,
        "bounds": [glyph.xMin, glyph.yMin, glyph.xMax, glyph.yMax]
        if glyph.numberOfContours else None,
    }
kern = {}
gpos = font["GPOS"].table
default = next(r.Script.DefaultLangSys for r in gpos.ScriptList.ScriptRecord if r.ScriptTag == "DFLT")
lookups = [lookup for i in default.FeatureIndex
           if gpos.FeatureList.FeatureRecord[i].FeatureTag == "kern"
           for lookup in gpos.FeatureList.FeatureRecord[i].Feature.LookupListIndex]
for pair in sorted({a + b for text in ["AV To", "VA", "Hg", "λΩé", " H"]
                    for a, b in zip(text, text[1:])} | {"oH"}):
    names = tuple(cmap[ord(c)] for c in pair)
    amount = 0
    for index in lookups:
        lookup = gpos.LookupList.Lookup[index]
        assert lookup.LookupType == 2 and lookup.LookupFlag == 0
        for subtable in lookup.SubTable:
            if names[0] not in subtable.Coverage.glyphs:
                continue
            if subtable.Format == 1:
                records = subtable.PairSet[subtable.Coverage.glyphs.index(names[0])].PairValueRecord
                record = next((r for r in records if r.SecondGlyph == names[1]), None)
                if record is None:
                    continue
                amount += record.Value1.XAdvance
            else:
                assert subtable.Format == 2
                a = subtable.ClassDef1.classDefs.get(names[0], 0)
                b = subtable.ClassDef2.classDefs.get(names[1], 0)
                amount += subtable.Class1Record[a].Class2Record[b].Value1.XAdvance
            break
    kern[pair] = amount
print(json.dumps({
    "source": "existing Apache-2.0 Roboto Regular; original metric extraction",
    "font_sha256": hashlib.sha256(source.read_bytes()).hexdigest(),
    "fonttools": fontTools.__version__, "units_per_em": font["head"].unitsPerEm,
    "metrics": metrics, "kern": kern, "legacy_kern_present": "kern" in font,
    "gpos_dflt_kern_lookups": lookups,
}, ensure_ascii=False, indent=2))
