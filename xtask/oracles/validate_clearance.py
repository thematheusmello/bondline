# SPDX-License-Identifier: MIT OR Apache-2.0
"""Independent oracle qualifications; no product imports or mutations."""
from pathlib import Path
import argparse,importlib.util,hashlib,json,math,random
SCRIPT_DIR=Path(__file__).resolve().parent
REPO=next(parent for parent in SCRIPT_DIR.parents if (parent/'assets/fonts/Roboto-Regular.ttf').is_file())
parser=argparse.ArgumentParser(description=__doc__)
parser.add_argument('--vectors',type=Path,default=REPO/'target/connected-labels/reproduced-clearance-vectors.json')
parser.add_argument('--output',type=Path,default=REPO/'target/connected-labels/clearance-self-checks.json')
args=parser.parse_args()
spec=importlib.util.spec_from_file_location('clearance_oracle',SCRIPT_DIR/'clearance_oracle.py')
oracle=importlib.util.module_from_spec(spec);spec.loader.exec_module(oracle)
r=random.Random(20261118)
for n in range(300):
    a=tuple(r.uniform(-10,10) for _ in range(2));b=tuple(r.uniform(-10,10) for _ in range(2))
    o=r.uniform(-2,2);w=r.uniform(.1,2);margin=[0,.125,2][n%3]
    best=oracle.line_max(a,b,o,w,margin)
    samples=[v for i in range(2049) if (v:=oracle.point_cut(oracle.evaluate((a,b),i/2048),o,w,margin)) is not None]
    if samples:assert best is not None and max(samples)<=best[0]+1e-9
for n in range(150):
    curve=tuple(tuple(r.uniform(-10,10) for _ in range(2)) for _ in range(3))
    o=r.uniform(-2,2);w=r.uniform(.1,2);margin=[0,.125,2][n%3]
    envelope=oracle.envelope(curve,o,w,margin)
    samples=[v for i in range(2049) if (v:=oracle.point_cut(oracle.evaluate(curve,i/2048),o,w,margin)) is not None]
    if samples:assert envelope is not None and max(samples)<=envelope[1]+1e-9
analytic=[]
for name,curve,half,margin,wanted in [
    ('tilted-line-sqrt2',((0,.5),(1,1.5)),.5,1,math.sqrt(2)),
    ('along-axis-quadratic-arch',((0,0),(4,0),(0,0)),.5,0,2),
    ('closed-zero-margin-tangent',((0,.75),(1,.25),(2,.75)),.5,0,1),
]:
    bracket,work,depth=oracle.bounded_cut([curve],0,half,margin,1e-5)
    assert bracket[0]<=wanted<=bracket[1],(name,wanted,bracket)
    assert bracket[1]-bracket[0]<=1e-5
    analytic.append({'id':name,'expected':wanted,'bracket':bracket,'work':work,'depth':depth})
vectors=json.loads((args.vectors).read_text())
family_samples=0
for case in vectors['cases']:
    curves=oracle.placed_curves(case['glyphs'],case['anchor_pt'],case['direction'])
    for offset,interval in zip(case['rail_offsets_pt'],case['per_rail_cut_pt_intervals']):
        samples=[v for curve in curves for i in range(257)
                 if (v:=oracle.point_cut(oracle.evaluate(curve,i/256),offset,case['line_width_pt']/2,case['margin_pt'])) is not None]
        family_samples+=len(curves)*257
        if samples:assert interval is not None and max(samples)<=interval[1]+1e-9,(case['id'],offset,max(samples),interval)

def aabb_exit(case):
    box=case['bounding_box_pt'];anchor=case['anchor_pt'];u=case['direction'];margin=case['margin_pt']
    return min((box[2]+margin-anchor[0])/u[0] if u[0]>0 else math.inf,
               (box[3]+margin-anchor[1])/u[1] if u[1]>0 else math.inf)
rounded=next(c for c in vectors['cases'] if c['id']=='authored-O-12-45')
cut=rounded['common_cut_pt_interval'];box=aabb_exit(rounded)
assert box>cut[1]+.1
multiple=next(c for c in vectors['cases'] if c['id']=='authored-O-triple-M0-0')
assert any(v is None for v in multiple['per_rail_cut_pt_intervals']) and multiple['common_cut_pt_interval'][0]>1
round_extra=rounded['line_width_pt']/2
assert round_extra>.1
output={'vector_sha256':hashlib.sha256((args.vectors).read_bytes()).hexdigest(),
        'random_seed':20261118,'line_dense_cases':300,'quadratic_dense_envelope_cases':150,'dense_samples_each':2049,
        'family_cases':len(vectors['cases']),'family_outline_samples':family_samples,'analytic_controls':analytic,
        'negative_controls':{'aabb_O45':{'incorrect':box,'correct_interval':cut},
          'staggered_multiple':{'incorrect_per_rail':multiple['per_rail_cut_pt_intervals'],'correct_common':multiple['common_cut_pt_interval']},
          'round_cap_extra_axial_radius':{'incorrect_extra':round_extra,'correct_extra':0}},
        'initial_oracle_qualification':'A first zero-margin line boundary evaluation fell outside its analytically closed strip by f64 rounding. Corrected analytic closed domain with bounded evaluation and feasible inward lower samples. No producer defect or weakened assertion.',
        'status':'passed','product_execution':False}
p=args.output;p.parent.mkdir(parents=True,exist_ok=True);p.write_text(json.dumps(output,indent=2)+'\n')
print(json.dumps({'status':'passed','analytic_controls':analytic,'family_cases':len(vectors['cases']),'family_outline_samples':family_samples,'negative_controls':output['negative_controls'],'sha256':hashlib.sha256(p.read_bytes()).hexdigest()}))
