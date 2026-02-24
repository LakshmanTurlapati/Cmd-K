"""
Invert K.png logo colors: black symbols -> white symbols.
Preserves the existing transparent background and alpha channel.
Output: K-white.png in the project root.
"""

from pathlib import Path
from PIL import Image

ROOT = Path(__file__).resolve().parent.parent
src = ROOT / "K.png"
dst = ROOT / "K-white.png"

img = Image.open(src).convert("RGBA")
pixels = img.load()

for y in range(img.height):
    for x in range(img.width):
        r, g, b, a = pixels[x, y]
        # Invert RGB, keep alpha untouched
        pixels[x, y] = (255 - r, 255 - g, 255 - b, a)

img.save(dst)
print(f"Saved {dst}")
