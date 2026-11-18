# SPDX-License-Identifier: MIT OR Apache-2.0
"""Independent original mixed scene tuples, using frozen source facts and FontTools curves."""
from pathlib import Path
import argparse,hashlib,importlib.util,json,math
ROOT=Path(__file__).resolve().parent;REPO=ROOT.parents[2]
parser=argparse.ArgumentParser(description=__doc__)
parser.add_argument('--output',type=Path,default=REPO/'target/mixed-labels/reproduced/mixed-layout-vectors.json')
args=parser.parse_args();OUTPUT=args.output;OUTPUT.parent.mkdir(parents=True,exist_ok=True)
FACTS=REPO/'crates/io/tests/fixtures/mixed-labels/mixed-label-facts.json'
spec=importlib.util.spec_from_file_location('independent_font',REPO/'xtask/oracles/clearance_oracle.py')
font=importlib.util.module_from_spec(spec);spec.loader.exec_module(font)
facts=json.loads(FACTS.read_text());cases=[]
views=[{'id':'fit1x','width_px':576,'height_px':576,'scale':1.0,'origin_px':[32.0,32.0]},
       {'id':'fit2x','width_px':1088,'height_px':1088,'scale':2.0,'origin_px':[32.0,32.0]},
       {'id':'physical2x','logical_width':576,'logical_height':576,'physical_scale':2.0,'phase':[0.25,0.75],
        'width_px':1152,'height_px':1152,'scale':2.0,'origin_px':[64.25,64.75]}]
for name,source in facts.items():
    if not source['mixed_expected']:continue
    by_id={n['id']:n for n in source['nodes']};glyphs={}
    for n in source['nodes']:
        if n['label'] is None:continue
        symbol={6:'C',7:'N',8:'O'}[n['element']]
        text,gs=font.authored(symbol,n['hydrogens'],n['label']['size'],n['point'])
        assert text==n['label']['text'] and font.placed_box(gs)==n['label']['bounding_box']
        glyphs[n['id']]=gs
    for view in views:
        bonds=[]
        for b in source['bonds']:
            a,z=by_id[b['begin']],by_id[b['end']]
            dx,dy=[z['point'][i]-a['point'][i] for i in (0,1)];length=math.hypot(dx,dy);direction=[dx/length,dy/length]
            cuts=[];work=0;depth=0
            for n,sign in [(a,1),(z,-1)]:
                if n['label'] is None:
                    assert n['model_id'] in source['hidden_model_atoms'];cuts.append([0.0,0.0]);continue
                curves=font.placed_curves(glyphs[n['id']],n['point'],[v*sign for v in direction])
                interval,used,deep=font.bounded_cut(curves,0,source['style']['LineWidth']/2,source['style']['MarginWidth'],1e-5/view['scale'])
                cuts.append([0.0,0.0] if interval is None else [max(0.0,v) for v in interval]);work+=used;depth=max(depth,deep)
            assert sum(c[1] for c in cuts)<length
            ends=[]
            for node,sign,cut in [(a,1,cuts[0]),(z,-1,cuts[1])]:
                point=[]
                for i in (0,1):
                    point.append(sorted((node['point'][i]+sign*v*direction[i])*view['scale']+view['origin_px'][i] for v in cut))
                ends.append(point)
            bonds.append({'source_id':b['id'],'model_id':b['model_id'],'begin_model_atom':a['model_id'],'end_model_atom':z['model_id'],
                          'untrimmed_points_pt':[a['point'],z['point']],'direction':direction,'length_pt':length,
                          'begin_cut_pt_interval':cuts[0],'end_cut_pt_interval':cuts[1],
                          'begin_canvas_xy_intervals':ends[0],'end_canvas_xy_intervals':ends[1],
                          'width_canvas':source['style']['LineWidth']*view['scale'],'oracle_work_nodes':work,'oracle_max_depth':depth})
        cases.append({'source_case':name,'view':view,'hidden_model_atoms':source['hidden_model_atoms'],
                      'hidden_canvas_anchors':{str(n['model_id']):[n['point'][i]*view['scale']+view['origin_px'][i] for i in (0,1)] for n in source['nodes'] if n['label'] is None},
                      'authored_glyphs':{str(by_id[sid]['model_id']):gs for sid,gs in glyphs.items()},'bonds':bonds,
                      'output_f32_allowance':'8*f32::EPSILON*max(1,abs(canvas coordinate))','oracle_tolerance_canvas':1e-5})
out={'base':'f16b79f','contract':'b30f0d6+5f58276','method':'Frozen original Expat/Decimal facts, independent FontTools curves and bounded butt-strip margin oracle; no Bondline output',
     'fonttools':font.fontTools.__version__,'font_sha256':font.FONT_SHA,'source_facts_sha256':hashlib.sha256(FACTS.read_bytes()).hexdigest(),'cases':cases}
OUTPUT.write_text(json.dumps(out,indent=2)+'\n')
print(json.dumps({'scenes':len(cases),'bonds':sum(len(c['bonds']) for c in cases),'sha256':hashlib.sha256(OUTPUT.read_bytes()).hexdigest()}))
