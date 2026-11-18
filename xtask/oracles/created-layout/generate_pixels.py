# SPDX-License-Identifier: MIT OR Apache-2.0
"""Independent binary butt-strip spans and FreeType glyph spans from frozen numeric tuples."""
from pathlib import Path
import argparse,hashlib,json,math
from PIL import Image,ImageDraw,ImageFont,__version__
ROOT=Path(__file__).resolve().parent;REPO=ROOT.parents[2]
parser=argparse.ArgumentParser(description=__doc__)
parser.add_argument('--layout',type=Path,default=ROOT/'created-layout-vectors.json')
parser.add_argument('--output',type=Path,default=REPO/'target/label-creation/reproduced/created-painted-spans.json')
parser.add_argument('--images',type=Path,default=REPO/'target/label-creation/reproduced/pixel-oracles')
args=parser.parse_args();OUTPUT=args.output;OUTPUT.parent.mkdir(parents=True,exist_ok=True)
DATA=json.loads(args.layout.read_text());FONT=REPO/'assets/fonts/Roboto-Regular.ttf'
assert __version__=='12.3.0' and hashlib.sha256(FONT.read_bytes()).hexdigest()==DATA['font_sha256']
OUT=args.images;OUT.mkdir(parents=True,exist_ok=True)
def spans(mask,width,height):
    result=[]
    for y in range(height):
        x=0
        while x<width:
            if not mask[y*width+x]:x+=1;continue
            begin=x
            while x<width and mask[y*width+x]:x+=1
            result.append([begin,y,x-begin,1])
    return result
rows=[]
chosen=[('created_3_black','fit1x'),('created_3_blue14','fit2x'),('created_3_nh','physical2x'),
        ('created_all_black','fit2x'),('created_3_selected_p12_5_m7_25','physical2x')]
for name,ident in chosen:
    case=next(c for c in DATA['cases'] if c['source_case']==name and c['view']['id']==ident)
    v=case['view'];w,h=v['width_px'],v['height_px'];s=v['scale'];origin=v['origin_px']
    certain=bytearray(w*h);possible=bytearray(w*h)
    for b in case['bonds']:
        p=b['untrimmed_points_pt'][0];anchor=[p[i]*s+origin[i] for i in (0,1)]
        u=b['direction'];n=[-u[1],u[0]];half=b['width_canvas']/2;length=b['length_pt']*s
        end=b['untrimmed_points_pt'][1]
        allowance=8*(2**-23)*max(1,*map(abs,anchor),*(abs(end[i]*s+origin[i]) for i in (0,1)))
        lo0,hi0=[x*s for x in b['begin_cut_pt_interval']];lo1,hi1=[x*s for x in b['end_cut_pt_interval']]
        for y in range(h):
            for x in range(w):
                dx,dy=x+.5-anchor[0],y+.5-anchor[1]
                axial,normal=dx*u[0]+dy*u[1],dx*n[0]+dy*n[1]
                if lo0-allowance<=axial<=length-lo1+allowance and abs(normal)<=half+allowance:possible[y*w+x]=1
                if hi0+allowance<=axial<=length-hi1-allowance and abs(normal)<=half-allowance:certain[y*w+x]=1
    layers=[]
    for model_id,gs in case['authored_glyphs'].items():
        ss=4;reference=Image.new('L',(w*ss,h*ss),0);draw=ImageDraw.Draw(reference)
        for g in gs:
            size=g['size_pt']*s*ss;assert size==round(size)
            font=ImageFont.truetype(FONT,round(size),layout_engine=ImageFont.Layout.BASIC)
            point=[(g['origin_pt'][i]*s+origin[i])*ss for i in (0,1)]
            draw.text(point,g['character'],font=font,fill=255,anchor='ls')
        glyph=bytearray(int(x>=128) for x in reference.resize((w,h),Image.Resampling.BOX).get_flattened_data())
        assert not any(a and b for a,b in zip(certain,glyph))
        rgb=[round(v*255) for v in case['glyph_rgb'][model_id]]
        layers.append({'model_atom':int(model_id),'rgb':rgb,'freetype_glyph_spans':spans(glyph,w,h)})
    probes={}
    for key,p in case['hidden_canvas_anchors'].items():
        hits=[[x,y] for y in range(math.floor(p[1])-1,math.ceil(p[1])+1) for x in range(math.floor(p[0])-1,math.ceil(p[0])+1) if 0<=x<w and 0<=y<h and certain[y*w+x]]
        assert hits,'hidden anchor must retain actual bond ink rather than a fabricated white mask'
        probes[key]={'anchor':p,'certain_black_near_anchor':hits}
    image=Image.new('RGB',(w,h),'white');pixels=image.load()
    for y in range(h):
        for x in range(w):
            i=y*w+x
            if certain[i]:pixels[x,y]=(0,0,0)
    for layer in layers:
        for x,y,length,_ in layer['freetype_glyph_spans']:
            for dx in range(length):pixels[x+dx,y]=tuple(layer['rgb'])
    target=OUT/f'{name}-{ident}.png';image.save(target)
    rows.append({'source_case':name,'view':ident,'dimensions':[w,h],'hidden_anchor_probes':probes,
                 'certain_black_butt_spans':spans(certain,w,h),'possible_black_butt_spans':spans(possible,w,h),
                 'glyph_layers':layers,'uncertain_black_pixels':sum(bool(a and not b) for a,b in zip(possible,certain)),
                 'png_sha256':hashlib.sha256(target.read_bytes()).hexdigest()})
out={'schema_version':1,'pillow':__version__,'font_sha256':DATA['font_sha256'],
     'layout_sha256':hashlib.sha256(args.layout.read_bytes()).hexdigest(),
     'rule':'Exact pixel centres in finite common-cut constant-width BUTT strips; no hidden glyph, white mask or round cap. Certain/possible bands include locked f32 output allowance.',
     'glyph_scope':'Independent4x FreeType reference only; cross-raster glyph IoU>=0.90. Not product/reference capture or approved golden.',
     'cases':rows}
OUTPUT.write_text(json.dumps(out,separators=(',',':'))+'\n')
print(json.dumps({'cases':len(rows),'uncertain_black_pixels':sum(r['uncertain_black_pixels'] for r in rows),
                  'sha256':hashlib.sha256(OUTPUT.read_bytes()).hexdigest()}))
