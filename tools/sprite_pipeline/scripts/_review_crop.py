#!/usr/bin/env python3
"""Crop one frame from sprite-strip idle.png for review."""
import sys
from pathlib import Path
from PIL import Image
def main():
    src = Path(sys.argv[1]); track = sys.argv[2]; frame_idx = int(sys.argv[3]) if len(sys.argv) > 3 else 4
    sz = 768 if track == 'anime' else 256
    img = Image.open(src)
    w,h = img.size
    nframes = w // sz
    fi = min(frame_idx, nframes-1)
    crop = img.crop((fi*sz, 0, (fi+1)*sz, sz))
    out = src.parent / f"_frame{fi}.png"
    crop.save(out)
    print(out)
if __name__ == "__main__": main()
