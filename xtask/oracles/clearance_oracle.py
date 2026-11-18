# SPDX-License-Identifier: MIT OR Apache-2.0
"""Independent bounded polyline envelopes for Root's original Euclidean butt-strip rule."""
from pathlib import Path
import hashlib
import heapq
import json
import math
import time
import argparse

import fontTools
from fontTools.pens.basePen import BasePen
from fontTools.pens.boundsPen import BoundsPen
from fontTools.ttLib import TTFont

SCRIPT_DIR = Path(__file__).resolve().parent
REPO = next(parent for parent in SCRIPT_DIR.parents if (parent / "assets/fonts/Roboto-Regular.ttf").is_file())
DEFAULT_VECTORS = REPO / "target/connected-labels/reproduced-clearance-vectors.json"
FONT = REPO / "assets/fonts/Roboto-Regular.ttf"
FONT_SHA = "56a45233d29f11b4dfb86d248e921939d115778f87325e7ae8cc108383d6664d"
assert hashlib.sha256(FONT.read_bytes()).hexdigest() == FONT_SHA
assert fontTools.__version__ == "4.61.1"
font = TTFont(FONT)
glyphset = font.getGlyphSet()
cmap = font.getBestCmap()
UPEM = font["head"].unitsPerEm


class Curves(BasePen):
    def __init__(self):
        super().__init__(glyphset)
        self.curves = []
        self.start = None
    def _moveTo(self, p): self.start = p
    def _lineTo(self, p):
        a = self._getCurrentPoint()
        if a != p: self.curves.append((a, p))
    def _qCurveToOne(self, c, p): self.curves.append((self._getCurrentPoint(), c, p))
    def _curveToOne(self, *_): raise AssertionError("pinned Roboto must remain quadratic")
    def _closePath(self):
        a = self._getCurrentPoint()
        if a != self.start: self.curves.append((a, self.start))
    def _endPath(self): raise AssertionError("font outlines must be closed")


def glyph_bounds(ch):
    pen = BoundsPen(glyphset)
    glyphset[cmap[ord(ch)]].draw(pen)
    return pen.bounds


def kern(a, b):
    # FontTools evaluates DFLT pair positioning independently of the Rust parser.
    table = font["GPOS"].table
    lang = next(r.Script.DefaultLangSys for r in table.ScriptList.ScriptRecord if r.ScriptTag == "DFLT")
    answer = 0
    for feature in lang.FeatureIndex:
        record = table.FeatureList.FeatureRecord[feature]
        if record.FeatureTag != "kern": continue
        for i in record.Feature.LookupListIndex:
            lookup = table.LookupList.Lookup[i]
            assert lookup.LookupType == 2 and lookup.LookupFlag == 0
            for sub in lookup.SubTable:
                ga, gb = cmap[ord(a)], cmap[ord(b)]
                if ga not in sub.Coverage.glyphs: continue
                if sub.Format == 1:
                    pair = next((p for p in sub.PairSet[sub.Coverage.glyphs.index(ga)].PairValueRecord if p.SecondGlyph == gb), None)
                    if pair is None: continue
                    answer += pair.Value1.XAdvance
                else:
                    assert sub.Format == 2
                    answer += sub.Class1Record[sub.ClassDef1.classDefs.get(ga, 0)].Class2Record[sub.ClassDef2.classDefs.get(gb, 0)].Value1.XAdvance
                break
    return answer


def union_box(parts):
    return [min(p[0] for p in parts), min(p[1] for p in parts), max(p[2] for p in parts), max(p[3] for p in parts)]


def placed_box(glyphs):
    parts = []
    for g in glyphs:
        l,b,r,t = glyph_bounds(g["character"])
        x,y = g["origin_pt"]
        f = g["size_pt"] / UPEM
        parts.append([x+l*f,y-t*f,x+r*f,y-b*f])
    return union_box(parts)


def authored(symbol, h, size, anchor):
    text = symbol + ("H" + (str(h) if h > 1 else "") if h else "")
    glyphs, x = [], 0
    for i,ch in enumerate(text):
        if i: x += kern(text[i-1], ch) * size / UPEM
        glyphs.append({"character":ch,"origin_pt":[x,0],"size_pt":size})
        x += font["hmtx"][cmap[ord(ch)]][0] * size / UPEM
    element_box = placed_box(glyphs[:len(symbol)])
    shift = [anchor[0]-(element_box[0]+element_box[2])/2,anchor[1]-(element_box[1]+element_box[3])/2]
    for g in glyphs: g["origin_pt"] = [g["origin_pt"][i]+shift[i] for i in (0,1)]
    return text,glyphs


def native(symbol, h, layout, anchor):
    # Independent font-metric reconstruction of the existing documented native layout tuples.
    size, factor = 12, 12/UPEM
    letters, x = [], 0
    for ch in symbol:
        letters.append((ch,x))
        x += font["hmtx"][cmap[ord(ch)]][0]*factor
    left = layout == "left"
    chosen, prefix = letters[-1] if left else letters[0]
    l,b,r,t = glyph_bounds(chosen)
    origin = anchor[0]-prefix-(l+r)*factor/2
    baseline = anchor[1]+(b+t)*factor/2
    gs = [{"character":ch,"origin_pt":[origin+offset,baseline],"size_pt":size} for ch,offset in letters]
    if h:
        digits = "" if h == 1 else str(h)
        if layout in ("above","below"):
            l,b,r,t = glyph_bounds("H")
            hx = anchor[0]-(l+r)*factor/2
            hy = anchor[1]+(-size if layout=="above" else size)+(b+t)*factor/2
        elif left:
            digit_width = sum(font["hmtx"][cmap[ord(c)]][0]*(8/UPEM) for c in digits)
            hx = origin-digit_width-font["hmtx"][cmap[ord("H")]][0]*factor
            hy = baseline
        else:
            hx,hy = origin+x,baseline
        gs.append({"character":"H","origin_pt":[hx,hy],"size_pt":size})
        cursor = hx+font["hmtx"][cmap[ord("H")]][0]*factor
        for digit in digits:
            gs.append({"character":digit,"origin_pt":[cursor,hy+3],"size_pt":8})
            cursor += font["hmtx"][cmap[ord(digit)]][0]*(8/UPEM)
    return gs


def placed_curves(glyphs, anchor, direction):
    normal = [-direction[1],direction[0]]
    result = []
    for g in glyphs:
        pen = Curves()
        glyphset[cmap[ord(g["character"])]].draw(pen)
        factor = g["size_pt"]/UPEM
        for curve in pen.curves:
            points = []
            for x,y in curve:
                q = [g["origin_pt"][0]+x*factor-anchor[0],g["origin_pt"][1]-y*factor-anchor[1]]
                points.append((sum(q[i]*direction[i] for i in (0,1)),sum(q[i]*normal[i] for i in (0,1))))
            result.append(tuple(points))
    return result


def evaluate(curve,t):
    if len(curve)==2: return tuple((1-t)*a+t*b for a,b in zip(*curve))
    a,b,c=curve
    return tuple((1-t)**2*a[i]+2*t*(1-t)*b[i]+t*t*c[i] for i in (0,1))


def point_cut(q,offset,half,margin):
    d=max(0,abs(q[1]-offset)-half)
    return None if d>margin else q[0]+math.sqrt(max(0,margin*margin-d*d))


def line_max(a,b,offset,half,margin):
    # Exact stationary candidates for a line plus a circular clearance envelope.
    da,dc=b[0]-a[0],b[1]-a[1]
    lo,hi=0.0,1.0
    if dc:
        enter,leave=sorted(((offset-half-margin-a[1])/dc,(offset+half+margin-a[1])/dc))
        lo,hi=max(lo,enter),min(hi,leave)
        if lo>hi: return None
    elif abs(a[1]-offset)>half+margin:
        return None
    mid=(lo+hi)/2
    ts=[lo,hi,mid]
    if dc:
        ts += [(offset+side*(half+extra)-a[1])/dc for side in (-1,1) for extra in (0,margin)]
        for side in (-1,1):
            dz=side*dc
            if da*dz>=0:
                z=margin*abs(da)/math.hypot(da,dz)
                ts.append((offset+side*(half+z)-a[1])/dc)
    best=None
    for t in ts:
        if not lo<=t<=hi: continue
        q=((1-t)*a[0]+t*b[0],(1-t)*a[1]+t*b[1])
        d=max(0,abs(q[1]-offset)-half)
        # The parameter is analytically inside the closed domain. Bound only its f64 evaluation.
        error=64*math.ulp(max(1,abs(a[1]),abs(b[1]),abs(offset),half,margin))
        if d>margin+error: continue
        d=min(d,margin)
        value=q[0]+math.sqrt(max(0,margin*margin-d*d))
        if best is None or value>best[0]: best=(value,t,mid)
    return best


def envelope(curve,offset,half,margin):
    epsilon=0.0 if len(curve)==2 else math.hypot(*(curve[1][i]-(curve[0][i]+curve[2][i])/2 for i in (0,1)))/2
    upper=line_max(curve[0],curve[-1],offset,half+epsilon,margin)
    if upper is None: return None
    lower=None
    candidates=[0,.5,1,upper[1],upper[2]]
    t=upper[1]
    for _ in range(8):
        t=math.nextafter(t,upper[2]);candidates.append(t)
    if half>=epsilon:
        inner=line_max(curve[0],curve[-1],offset,half-epsilon,margin)
        if inner is not None: candidates.extend(inner[1:])
    for t in candidates:
        value=point_cut(evaluate(curve,t),offset,half,margin)
        if value is not None: lower=value if lower is None else max(lower,value)
    numeric=64*math.ulp(max(1,*(abs(v) for point in curve for v in point),abs(offset),margin,half))
    pad=numeric+math.sqrt(2*margin*numeric+numeric*numeric)
    return None if lower is None else lower-pad,upper[0]+epsilon+pad


def bounded_cut(curves,offset,half,margin,tolerance):
    queue,lower,work,serial=[],None,0,0
    for curve in curves:
        result=envelope(curve,offset,half,margin);work+=1
        if result is None: continue
        lo,hi=result
        if lo is not None: lower=lo if lower is None else max(lower,lo)
        heapq.heappush(queue,(-hi,serial,0,curve));serial+=1
    if not queue: return None,work,0
    max_depth=0
    while lower is None or -queue[0][0]-lower>tolerance:
        neg,_,depth,curve=heapq.heappop(queue)
        if lower is not None and -neg<=lower+tolerance: continue
        assert len(curve)==3 and depth<64 and work<2_000_000,(work,depth,lower,-neg)
        a,c,b=curve;ac=tuple((x+y)/2 for x,y in zip(a,c));cb=tuple((x+y)/2 for x,y in zip(c,b));mid=tuple((x+y)/2 for x,y in zip(ac,cb))
        for child in ((a,ac,mid),(mid,cb,b)):
            result=envelope(child,offset,half,margin);work+=1
            if result is None: continue
            lo,hi=result
            if lo is not None: lower=lo if lower is None else max(lower,lo)
            heapq.heappush(queue,(-hi,serial,depth+1,child));serial+=1
        max_depth=max(max_depth,depth+1)
        if not queue: break
    upper=max(lower,-queue[0][0] if queue else lower)
    return [lower,upper],work,max_depth


def one_case(ident,symbol,element,h,size,angle,margin,offsets,width=1.5,scale=1.0,layout="authored_flat",anchor=(160,160)):
    angle=math.radians(angle);direction=[math.cos(angle),math.sin(angle)]
    direction=[0.0 if abs(v)<1e-15 else v for v in direction]
    text,gs=authored(symbol,h,size,anchor)
    if layout!="authored_flat": gs=native(symbol,h,layout,anchor)
    curves=placed_curves(gs,anchor,direction)
    tolerance=1e-5/scale
    intervals,work,depth=[],0,0
    for o in offsets:
        interval,used,deep=bounded_cut(curves,o,width/2,margin,tolerance)
        intervals.append(interval);work+=used;depth=max(depth,deep)
    present=[p for p in intervals if p is not None]
    common=[max(p[0] for p in present),max(p[1] for p in present)] if present else [0.0,0.0]
    raw_box=placed_box(gs)
    label={"position":None,"bounding_box":raw_box,"justification":"left","label_justification":"left","label_alignment":None,
           "runs":[{"text":text,"style":{"font_family":"Roboto","size_pt":size,"face":0,"rgb":[.2,.4,.8]}}]}
    return {"id":ident,"element":element,"symbol":symbol,"explicit_hydrogens":h,"text":text,"font_size_pt":size,"layout":layout,
            "glyphs":gs,"anchor_pt":list(anchor),"bounding_box_pt":placed_box(gs),"direction":direction,
            "rail_offsets_pt":offsets,"line_width_pt":width,"margin_pt":margin,"bond_length_pt":40,"bond_spacing_percent":15,
            "canvas_scale":scale,"canvas_origin":[32.25,31.75],"common_cut_pt_interval":common,"per_rail_cut_pt_intervals":intervals,
            "common_cut_canvas_interval":[v*scale for v in common],"oracle_work_nodes":work,"oracle_max_depth":depth,
            "oracle_tolerance_canvas":1e-5,"output_f32_allowance_formula":"8*f32::EPSILON*max(1,absolute canvas coordinate)",
            "authored_source_case":layout=="authored_flat","authored_label":label if layout=="authored_flat" else None}


def main(output=None):
    started=time.monotonic();cases=[]
    for symbol,element in [("O",8),("H",1),("I",53),("N",7),("F",9),("Cl",17),("Br",35)]:
        for angle in (0,30,45,60,90,180,270):
            cases.append(one_case(f"authored-{symbol}-12-{angle}",symbol,element,0,12,angle,2,[0]))
    for text,symbol,element,h in [("OH2","O",8,2),("NH3","N",7,3),("CH65535","C",6,65535)]:
        for size in (10,12,24):
            cases.append(one_case(f"authored-{text}-{size}-45",symbol,element,h,size,45,.5,[0]))
    for margin in (0,.125,.5,2):
        for offsets,name in [([0],"single"),([-3,3],"double"),([-6,0,6],"triple")]:
            for angle in (0,45,90):
                cases.append(one_case(f"authored-O-{name}-M{margin}-{angle}","O",8,0,12,angle,margin,offsets))
    for scale in (.75,1,1.25,2):
        for size in (12,24):
            cases.append(one_case(f"physical-F-{size}-scale{scale}","F",9,0,size,30,.125,[-3,3],scale=scale))
    for layout in ("right","left","above","below"):
        for angle in (0,45,90,180,270):
            cases.append(one_case(f"native-NH3-{layout}-{angle}","N",7,3,12,angle,.5,[0],layout=layout))
    short=[]
    for angle in (30,45,60):
        a=one_case(f"short-OO-{angle}-a","O",8,0,12,angle,2,[0])
        b=one_case(f"short-OO-{angle}-b","O",8,0,12,angle+180,2,[0])
        cases.extend([a,b]);lo=a['common_cut_pt_interval'][0]+b['common_cut_pt_interval'][0];hi=a['common_cut_pt_interval'][1]+b['common_cut_pt_interval'][1]
        short.append({"angle":angle,"begin_case":a['id'],"end_case":b['id'],"consumed_length_pt":lo-.01,"visible_length_pt":hi+2,"sum_cut_pt_interval":[lo,hi]})
    doc={"schema_version":1,"base":"61be8e6e496281e373d59fdc0968617315e37800","font_sha256":FONT_SHA,"fonttools":fontTools.__version__,"units_per_em":UPEM,
         "method":"Independent FontTools exact outlines with certified pointwise quadratic-to-chord envelopes; analytic line/circular-envelope stationary points; adaptive maximum bracket. No producer imports, AABB final geometry or reference fit constants.",
         "quadratic_flatten_error":"max_t ||Bezier(t)-linear(t)|| <= ||control-(begin+end)/2||/2; expand/contract normal strip by that error and add its axial bound",
         "reference_pixel_parity":"unproven; original Root Euclidean policy only","cases":cases,"short_oo_cases":short,"elapsed_seconds":time.monotonic()-started}
    path=Path(output) if output is not None else DEFAULT_VECTORS;path.parent.mkdir(parents=True,exist_ok=True);path.write_text(json.dumps(doc,indent=2)+'\n')
    print(json.dumps({"cases":len(cases),"max_canvas_bracket":max((r['common_cut_canvas_interval'][1]-r['common_cut_canvas_interval'][0]) for r in cases),"max_work":max(r['oracle_work_nodes'] for r in cases),"max_depth":max(r['oracle_max_depth'] for r in cases),"seconds":doc['elapsed_seconds'],"sha256":hashlib.sha256(path.read_bytes()).hexdigest(),"bytes":path.stat().st_size}))


if __name__=="__main__":
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output",type=Path,default=DEFAULT_VECTORS)
    args=parser.parse_args()
    main(args.output)
