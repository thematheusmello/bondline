"""Independent test-only FreeType masks; tolerance chosen before comparing outputs."""
import json
import sys
from PIL import Image, ImageDraw, ImageFont, __version__

assert __version__ == "12.3.0", __version__
data = json.load(sys.stdin)
width,height=data.get('width',400),data.get('height',400)
position=data.get('position',[200,200])
actual = Image.new("L", (width, height))
draw = ImageDraw.Draw(actual)
for x, y, width, height in data["boxes"]:
    assert 0 <= x < x + width <= actual.width and 0 <= y < y + height <= actual.height
    draw.rectangle((x, y, x + width - 1, y + height - 1), fill=255)
expected = Image.new("L", actual.size)
draw = ImageDraw.Draw(expected)
main = ImageFont.truetype(sys.argv[1], 96)
sub = ImageFont.truetype(sys.argv[1], 64)
# Independent fontTools 4.61.1 vectors: units 2048; C ink x=119..1240,
# y=-20..1476, advance=1333; H advance=1460; 3 advance=1150.
origin = position[0] - (119 + 1240) * (96 / 2048) / 2
baseline = position[1] + (-20 + 1476) * (96 / 2048) / 2
hydrogens=data.get('hydrogens',3)
assert isinstance(hydrogens,int) and 0<=hydrogens<=65535
glyphs=[("C",origin,baseline,main)]
if hydrogens:
    digits=str(hydrogens) if hydrogens>1 else ''
    digit_glyphs=[]
    if data['toward_right']:
        cursor=origin
        for digit in reversed(digits):
            cursor-=sub.getlength(digit)
            digit_glyphs.append((digit,cursor,baseline+24,sub))
        digit_glyphs.reverse()
        h_x=cursor-main.getlength('H')
    else:
        h_x=origin+main.getlength('C')
        cursor=h_x+main.getlength('H')
        for digit in digits:
            digit_glyphs.append((digit,cursor,baseline+24,sub))
            cursor+=sub.getlength(digit)
    glyphs.append(('H',h_x,baseline,main))
    glyphs.extend(digit_glyphs)
for char, x, y, font in glyphs:
    draw.text((x, y), char, fill=255, font=font, anchor="ls")
reference = [value >= 128 for value in expected.get_flattened_data()]
pixels = [value != 0 for value in actual.get_flattened_data()]
intersection = sum(a and b for a, b in zip(pixels, reference))
union = sum(a or b for a, b in zip(pixels, reference))
# Different rasterizers/hinting; require >=90% ink intersection-over-union at 96 em.
assert union and intersection / union >= 0.90, (intersection, union, intersection / union)
print(json.dumps({"freetype_ink_iou": intersection / union}))
