#!/usr/bin/env python3
"""Generate Braille ASCII frames of Tendou Aris teabagging.

Like momoisay, Aris is rendered ALONE (background removed) as filled Braille dots.
A UNION bounding box per source keeps her from being clipped or jittering; a dot
is filled where the pixel is opaque AND dark. Detached blobs such as her halo
bracket are kept (all interior blobs survive the chroma key) and are filled as a
silhouette so the pale halo stays visible at Braille resolution.

Sources (in refs/):
  * aris2.gif    — transparent-background teabag loop (its halo reads at Braille res)
  * yt_src.webm  — green-screen clip around 1:05-1:27 (keyframe slop shifts it a few
        seconds earlier); motion_a = 8.5-13s = the single-Aris bob after the clones
  * yt_src2.mp4  — green-screen clip of 0:01-0:06 (the opening, single Aris);
        motion_c = the whole file = the fist-pump dance bob that turns around halfway
  * yt_src3.mp4  — green-screen clip around 1:14-1:30;
        motion_b = 5.75-11.3s = the BOTH-HANDS-UP bob (the 1:20-1:24 bit): first
        three and last two checkHU2 cells dropped, ends before a second Aris
        walks in

Reproducible: `python3 tools/gen_frames.py`   (needs: Pillow, numpy, scipy, ffmpeg)

Emits frames/<name>.txt (frames joined by form-feed \\x0c) + frames/static.txt.
"""
import os
import subprocess
import tempfile

import numpy as np
from PIL import Image
from scipy import ndimage

WCELLS = 34
DARK = 0.55          # luminance below this (over white) -> dot
MIN_BLOB = 60        # keep chroma-keyed blobs at least this many px (halo ~hundreds)
PAD = 4
SEP = "\x0c"

# name -> dict describing how to get RGBA frames + alpha/minor masks
SOURCES = {
    "aris":     {"kind": "gif",   "path": "refs/aris2.gif"},
    "motion_a": {"kind": "green", "path": "refs/yt_src.webm",  "ss": 8.5,  "to": 13,   "fps": 12},
    "motion_b": {"kind": "green", "path": "refs/yt_src3.mp4",  "ss": 5.75, "to": 11.3, "fps": 12},
    # motion_c's union box is tall (high halo + a deep bow mid-turn) — at 34 cells it
    # would be 25 rows and `animate` would stop fitting 80x24 terminals, so render it
    # 30 cells wide (22 rows) and center it on the shared 34-cell canvas.
    "motion_c": {"kind": "green", "path": "refs/yt_src2.mp4",  "ss": 0,    "to": 6,    "fps": 12, "wcells": 30},
}
STATIC_FROM = "aris"

_DOTS = [(0, 0, 0), (1, 0, 1), (2, 0, 2), (0, 1, 3),
         (1, 1, 4), (2, 1, 5), (3, 0, 6), (3, 1, 7)]


def green_alpha(rgb):
    """Chroma-key. Returns (alpha, minor) as uint8 masks.

    alpha: every non-green interior blob >= MIN_BLOB px (body, halo, ...).
    minor: those blobs except the largest (the halo etc.) — force-filled later
    so pale detached parts stay visible.
    """
    r, g, b = rgb[..., 0].astype(int), rgb[..., 1].astype(int), rgb[..., 2].astype(int)
    green = (g > 100) & (g > r + 30) & (g > b + 30)
    nong = ~green
    lbl, _ = ndimage.label(nong)  # type: ignore[misc]
    border = set(lbl[0, :]) | set(lbl[-1, :]) | set(lbl[:, 0]) | set(lbl[:, -1])
    border.discard(0)
    interior = nong & ~np.isin(lbl, list(border))
    l2, n2 = ndimage.label(interior)  # type: ignore[misc]
    alpha = np.zeros(rgb.shape[:2], bool)
    minor = np.zeros(rgb.shape[:2], bool)
    if n2 > 0:
        sizes = ndimage.sum(np.ones_like(l2), l2, range(1, n2 + 1))
        main = int(np.argmax(sizes)) + 1
        for i, sz in enumerate(sizes, start=1):
            if sz >= MIN_BLOB:
                blob = l2 == i
                alpha |= blob
                if i != main:
                    minor |= blob
    return (alpha * 255).astype("uint8"), (minor * 255).astype("uint8")


def load_source(spec):
    """Return a list of (rgba_image, alpha_uint8, minor_uint8) per frame."""
    if spec["kind"] == "gif":
        im = Image.open(spec["path"])
        out = []
        for i in range(getattr(im, "n_frames", 1)):
            im.seek(i)
            f = im.convert("RGBA")
            a = np.asarray(f)[..., 3]
            out.append((f, a, np.zeros_like(a)))
        return out
    # green-screen video segment
    with tempfile.TemporaryDirectory() as td:
        subprocess.run(["ffmpeg", "-y", "-ss", str(spec["ss"]), "-to", str(spec["to"]),
                        "-i", spec["path"], "-vf", f"fps={spec['fps']}", os.path.join(td, "f%04d.png")],
                       check=True, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        out = []
        for name in sorted(os.listdir(td)):
            im = Image.open(os.path.join(td, name)).convert("RGB")
            rgb = np.asarray(im)
            alpha, minor = green_alpha(rgb)
            # The halo bracket in this source is pale teal — too bright for the
            # dark-fill — so force-fill teal-ish opaque pixels alongside the
            # detached blobs. (Gif sources are untouched.)
            r, g, b = rgb[..., 0].astype(int), rgb[..., 1].astype(int), rgb[..., 2].astype(int)
            teal = (b > r + 25) & (g > r + 15) & (alpha > 0)
            minor = np.maximum(minor, (teal * 255).astype("uint8"))
            out.append((im.convert("RGBA"), alpha, minor))
        return out


def union_box(frames):
    x0 = y0 = 1 << 30
    x1 = y1 = 0
    for _, a, _ in frames:
        ys, xs = np.where(a > 128)
        if len(xs):
            x0, x1 = min(x0, int(xs.min())), max(x1, int(xs.max()))
            y0, y1 = min(y0, int(ys.min())), max(y1, int(ys.max()))
    w, h = frames[0][0].size
    return (max(0, x0 - PAD), max(0, y0 - PAD), min(w, x1 + PAD), min(h, y1 + PAD))


def to_braille(rgba, alpha, minor, box, wcells=WCELLS):
    im = rgba.crop(box)
    a = np.asarray(Image.fromarray(alpha).crop(box), np.float32) / 255.0
    mn = np.asarray(Image.fromarray(minor).crop(box), np.float32) / 255.0
    rgb = np.asarray(im.convert("RGBA"), np.float32)[..., :3] / 255.0
    white = rgb * a[..., None] + (1 - a[..., None])
    w, h = im.size
    wpx = wcells * 2
    hpx = max(4, round(wpx * h / w))
    lum = Image.fromarray((white * 255).astype("uint8")).convert("L").resize((wpx, hpx), Image.Resampling.LANCZOS)
    opa = Image.fromarray((a * 255).astype("uint8")).resize((wpx, hpx), Image.Resampling.LANCZOS)
    mno = Image.fromarray((mn * 255).astype("uint8")).resize((wpx, hpx), Image.Resampling.LANCZOS)
    g = np.asarray(lum, np.float32) / 255.0
    op = np.asarray(opa, np.float32) / 255.0
    mr = np.asarray(mno, np.float32) / 255.0
    mask = ((op > 0.5) & (g < DARK)) | (mr > 0.5)

    H = (hpx + 3) // 4 * 4
    W = (wpx + 1) // 2 * 2
    m = np.zeros((H, W), bool)
    m[:hpx, :wpx] = mask
    lines = []
    for by in range(0, H, 4):
        row = []
        for bx in range(0, W, 2):
            bits = sum((1 << bit) for r, c, bit in _DOTS if m[by + r, bx + c])
            row.append(chr(0x2800 + bits))
        lines.append("".join(row))
    if wcells < WCELLS:  # center narrower art on the shared WCELLS-wide canvas
        lpad = "⠀" * ((WCELLS - wcells) // 2)
        rpad = "⠀" * (WCELLS - wcells - (WCELLS - wcells) // 2)
        lines = [lpad + ln + rpad for ln in lines]
    return "\n".join(lines)


if __name__ == "__main__":
    os.makedirs("frames", exist_ok=True)
    first = {}
    for name, spec in SOURCES.items():
        frames = load_source(spec)
        box = union_box(frames)
        arts = [to_braille(f, a, mn, box, spec.get("wcells", WCELLS)) for f, a, mn in frames]
        open(f"frames/{name}.txt", "w").write(SEP.join(arts))
        first[name] = arts[0]
        print(f"{name}: {len(arts)} frames  box={box}")
    open("frames/static.txt", "w").write(first[STATIC_FROM])
    print("static: 1 frame")
