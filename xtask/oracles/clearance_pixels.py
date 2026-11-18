# SPDX-License-Identifier: MIT OR Apache-2.0
"""Independent binary butt-strip pixel centres and supersampled FreeType glyph spans."""
from pathlib import Path
import argparse,hashlib,json,math
from PIL import Image,ImageDraw,ImageFont,__version__

SCRIPT_DIR=Path(__file__).resolve().parent
REPO=next(parent for parent in SCRIPT_DIR.parents if (parent/'assets/fonts/Roboto-Regular.ttf').is_file())
parser=argparse.ArgumentParser(description=__doc__)
parser.add_argument('--vectors',type=Path,default=REPO/'target/connected-labels/reproduced-clearance-vectors.json')
parser.add_argument('--output-dir',type=Path,default=REPO/'target/connected-labels/clearance-preview')
args=parser.parse_args()
ROOT=args.output_dir;ROOT.mkdir(parents=True,exist_ok=True)
DATA=json.loads(args.vectors.read_text())
FONT=REPO/'assets/fonts/Roboto-Regular.ttf' 
assert __version__=='12.3.0'
assert hashlib.sha256(FONT.read_bytes()).hexdigest()==DATA['font_sha256']

def spans(mask,width,height):
    result=[]
    for y in range(height):
        x=0
        while x<width:
            if not mask[y*width+x]:x+=1;continue
            start=x
            while x<width and mask[y*width+x]:x+=1
            result.append([start,y,x-start,1])
    return result

ids=['authored-O-12-45','authored-H-12-30','authored-F-12-0','authored-Cl-12-60',
     'authored-O-triple-M0-0','authored-O-double-M0.125-45','authored-CH65535-12-45',
     'physical-F-12-scale1.25','native-NH3-above-90','native-NH3-left-180']
rows=[]
for ident in ids:
    case=next(c for c in DATA['cases'] if c['id']==ident)
    scale=case['canvas_scale'];origin=case['canvas_origin'];anchor=[origin[i]+case['anchor_pt'][i]*scale for i in (0,1)]
    u=case['direction'];n=[-u[1],u[0]];width=height=768
    lower,upper=case['common_cut_canvas_interval'];half=case['line_width_pt']*scale/2
    black=bytearray(width*height);uncertain=bytearray(width*height)
    # No white mask/eraser. The same common axial cut applies to all component strips.
    for y in range(height):
        for x in range(width):
            dx,dy=x+.5-anchor[0],y+.5-anchor[1]
            a,c=dx*u[0]+dy*u[1],dx*n[0]+dy*n[1]
            if a>130*scale:continue
            if not any(abs(c-o*scale)<=half for o in case['rail_offsets_pt']):continue
            if a>=upper:black[y*width+x]=1
            elif a>=lower:uncertain[y*width+x]=1
    ss=4;reference=Image.new('L',(width*ss,height*ss),0);draw=ImageDraw.Draw(reference)
    for glyph in case['glyphs']:
        size=glyph['size_pt']*scale*ss
        assert size==round(size)
        face=ImageFont.truetype(FONT,round(size),layout_engine=ImageFont.Layout.BASIC)
        point=[(origin[i]+glyph['origin_pt'][i]*scale)*ss for i in (0,1)]
        draw.text(point,glyph['character'],font=face,fill=255,anchor='ls')
    reference=reference.resize((width,height),Image.Resampling.BOX)
    blue=bytearray(int(v>=128) for v in reference.get_flattened_data())
    overlap=sum(bool(a and b) for a,b in zip(black,blue))
    image=Image.new('RGB',(width,height),'white');pixels=image.load()
    for y in range(height):
        for x in range(width):
            i=y*width+x
            if black[i]:pixels[x,y]=(0,0,0)
            if blue[i]:pixels[x,y]=(51,102,204)
    target=ROOT/(ident+'-euclidean-oracle.png');image.save(target)
    rows.append({'id':ident,'dimensions':[width,height],'black_butt_spans':spans(black,width,height),
                 'freetype_glyph_spans':spans(blue,width,height),'endpoint_interval_uncertain_spans':spans(uncertain,width,height),
                 'freeType_binary_alias_overlap_pixels':overlap,'png_sha256':hashlib.sha256(target.read_bytes()).hexdigest(),
                 'pixel_scope':'Independent original-rule oracle only; not app/reference capture or approved golden. Glyph IoU >=0.90 remains a separate cross-raster check.'})
doc={'schema_version':1,'pillow':__version__,'font_sha256':DATA['font_sha256'],
     'vector_sha256':hashlib.sha256(args.vectors.read_bytes()).hexdigest(),
     'rule':'pixel centres in finite common-cut constant-width BUTT strips, no round radius or white eraser; separate FreeType glyph masks',
     'output_f32_rounding':'Consumer additionally accounts for the locked8*f32EPS*maxCoord bound, not silently absorbed into endpoint interval',
     'cases':rows}
out=ROOT/'clearance-painted-spans.json';out.write_text(json.dumps(doc,separators=(',',':'))+'\n')
print(json.dumps({'cases':len(rows),'uncertain_pixels':sum(sum(s[2] for s in row['endpoint_interval_uncertain_spans']) for row in rows),
                  'glyph_alias_overlap_pixels':{row['id']:row['freeType_binary_alias_overlap_pixels'] for row in rows},
                  'sha256':hashlib.sha256(out.read_bytes()).hexdigest(),'bytes':out.stat().st_size}))
