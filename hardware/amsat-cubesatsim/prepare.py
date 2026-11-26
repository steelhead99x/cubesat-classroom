"""Pack the AMSAT frame and the printed payload kit into Bambu Studio projects.

Writes p1s/cubesat-kit.3mf (payload deck, clamp and latches; frame), p1s/cubesat-fit-plate.3mf
(the open-truss fit plate with its own clamp and latches) and p1s/cubesat-test-coupons.3mf (one of
each joint, about 5 g each), for a Bambu Lab printer with a 256 mm bed
(P1S, P1P, X1C, X1E, A1). Every part is placed in its print orientation as its own object
(so Bambu Studio keeps it on its plate), small parts that flex are set to 100% infill, and each
plate has a preview picture. The file follows Bambu Studio's own project layout
(src/libslic3r/Format/bbs_3mf.cpp): per-object meshes in 3D/Objects/, plates and object
settings in Metadata/model_settings.config. It carries no printer or filament profile,
so Bambu Studio keeps the ones you have selected.

Run generate.py first. Only numpy, trimesh and the standard library are needed, and the
output is the same byte for byte on every run. GPL-3.0, like the rest of this folder.
"""

import math
import struct
import zlib
from pathlib import Path
from xml.sax.saxutils import escape
from zipfile import ZIP_DEFLATED, ZipFile, ZipInfo

import numpy as np
import trimesh

HERE = Path(__file__).resolve().parent
BED = 256.0                     # Bambu Lab P1S / P1P / X1 / A1 build plate
PLATE_STRIDE = BED * 1.2        # Bambu Studio lays plates out with a 1/5 plate gap
GAP, ROW_GAP, MARGIN = 6.0, 8.0, 8.0
FLIP = trimesh.transformations.rotation_matrix(np.pi, [1, 0, 0])
APP = "BambuStudio-02.00.00.00"
DATE = "2026-11-25"
SOLID = {"sparse_infill_density": "100%"}

# name: (file, transform, settings, preview colour)
PARTS = {
    "Payload plate": ("stl/payload-plate.stl", None, {}, (0.62, 0.65, 0.70)),
    "Fit plate": ("stl/fit-plate.stl", None, {}, (0.22, 0.24, 0.27)),
    "Nano clamp": ("stl/lower-clamp.stl", None, {}, (0.90, 0.55, 0.18)),
    "Latch clip": ("stl/latch-clip.stl", None, SOLID, (0.92, 0.93, 0.95)),
    "Lock key": ("stl/lock-key.stl", None, SOLID, (0.85, 0.15, 0.12)),
    "Pin coupon base": ("stl/coupon-pin-base.stl", None, SOLID, (0.55, 0.60, 0.68)),
    "Pin coupon tab": ("stl/coupon-pin-tab.stl", None, SOLID, (0.62, 0.65, 0.70)),
    "Latch coupon base": ("stl/coupon-latch-base.stl", None, {}, (0.55, 0.60, 0.68)),
    "Coupon board": ("stl/coupon-board.stl", None, {}, (0.10, 0.45, 0.35)),
    "Side": ("upstream/cubesatsim_frame_side.v2.0.stl", None, {}, (0.62, 0.65, 0.70)),
    "Side with camera hole": ("upstream/cubesatsim_frame_side_camera.v2.0.stl", None, {}, (0.62, 0.65, 0.70)),
    "Top": ("upstream/cubesatsim_frame_top_bottom.v2.0.stl", FLIP, {}, (0.62, 0.65, 0.70)),
    "Bottom with columns": ("stl/bottom-with-columns.stl", None, {}, (0.62, 0.65, 0.70)),
}

LATCH_ROWS = [["Latch clip"] * 3 + ["Lock key"] * 3]

# File: (title, plates). Each plate: name and rows of parts, top to bottom.
PROJECTS = {
    "cubesat-kit.3mf": ("CubeSat Classroom kit", [
        ("Payload deck, clamp and latches", [["Payload plate", "Nano clamp"]] + LATCH_ROWS),
        ("Frame", [["Side", "Side with camera hole"], ["Top", "Bottom with columns"]]),
    ]),
    "cubesat-fit-plate.3mf": ("CubeSat Classroom fit plate", [
        ("Fit plate, clamp and latches", [["Fit plate", "Nano clamp"]] + LATCH_ROWS),
    ]),
    "cubesat-test-coupons.3mf": ("CubeSat Classroom test coupons", [
        ("Test coupons", [["Pin coupon base", "Pin coupon tab"],
                          ["Latch coupon base", "Coupon board", "Latch clip", "Lock key"]]),
    ]),
}


def load_parts():
    meshes = {}
    for name, (path, transform, _, _) in PARTS.items():
        mesh = trimesh.load_mesh(HERE / path)
        if not mesh.is_volume:
            raise ValueError(f"{path}: not a closed solid")
        if transform is not None:
            mesh.apply_transform(transform)
        lo, hi = mesh.bounds
        mesh.apply_translation((-(lo[0] + hi[0]) / 2, -(lo[1] + hi[1]) / 2, -lo[2]))   # centred, on the bed
        meshes[name] = mesh
    return meshes


def layout(rows, meshes):
    """Centre each row on the bed and stack the rows top to bottom. Returns (name, x, y)."""
    heights = [max(meshes[n].extents[1] for n in row) for row in rows]
    total = sum(heights) + ROW_GAP * (len(rows) - 1)
    y = BED / 2 + total / 2
    placed = []
    for row, h in zip(rows, heights):
        width = sum(meshes[n].extents[0] for n in row) + GAP * (len(row) - 1)
        if width > BED - 2 * MARGIN or total > BED - 2 * MARGIN:
            raise ValueError(f"{row} does not fit the {BED:.0f} mm plate")
        x = BED / 2 - width / 2
        for n in row:
            w = meshes[n].extents[0]
            placed.append((n, x + w / 2, y - h / 2))
            x += w + GAP
        y -= h + ROW_GAP
    return placed


def plate_origin(index, count):
    """Bambu Studio's compute_colum_count() and compute_shape_position()."""
    value = math.sqrt(count)
    cols = round(value) + 1 if value > round(value) else round(value)
    return (index % cols) * PLATE_STRIDE, -(index // cols) * PLATE_STRIDE


def fmt(v):
    s = f"{v:.6f}".rstrip("0").rstrip(".")
    return "0" if s in ("-0", "") else s


def mesh_xml(mesh, object_id, uuid):
    out = [f'  <object id="{object_id}" p:UUID="{uuid}" type="model">\n   <mesh>\n    <vertices>\n']
    out += [f'     <vertex x="{fmt(x)}" y="{fmt(y)}" z="{fmt(z)}"/>\n' for x, y, z in mesh.vertices]
    out.append("    </vertices>\n    <triangles>\n")
    out += [f'     <triangle v1="{a}" v2="{b}" v3="{c}"/>\n' for a, b, c in mesh.faces]
    out.append("    </triangles>\n   </mesh>\n  </object>\n")
    return "".join(out)


MODEL_HEAD = ('<?xml version="1.0" encoding="UTF-8"?>\n<model unit="millimeter" xml:lang="en-US" '
              'xmlns="http://schemas.microsoft.com/3dmanufacturing/core/2015/02" '
              'xmlns:BambuStudio="http://schemas.bambulab.com/package/2021" '
              'xmlns:p="http://schemas.microsoft.com/3dmanufacturing/production/2015/06" requiredextensions="p">\n')


# ---------- preview pictures: a small orthographic z-buffer renderer ----------

def _apply(m, v):
    """v @ m.T with plain element-wise arithmetic, so the pictures don't depend on numpy's BLAS."""
    return np.stack([v[:, 0] * m[i, 0] + v[:, 1] * m[i, 1] + v[:, 2] * m[i, 2] for i in range(3)], axis=1)


def render(items, size):
    """items: (mesh in plate coordinates, colour). Isometric view of a 256 mm plate."""
    az, el = math.radians(-35), math.radians(52)
    rot = np.array([[math.cos(az), -math.sin(az), 0], [math.sin(az), math.cos(az), 0], [0, 0, 1]])
    tilt = np.array([[1, 0, 0], [0, math.cos(el), -math.sin(el)], [0, math.sin(el), math.cos(el)]])
    view = np.array([[sum(tilt[i, k] * rot[k, j] for k in range(3)) for j in range(3)] for i in range(3)])
    light = np.array([-0.35, 0.45, 0.82])
    light = light / math.sqrt(float(light[0] ** 2 + light[1] ** 2 + light[2] ** 2))
    bed = trimesh.creation.box(extents=(BED, BED, 1.0))
    bed.apply_translation((BED / 2, BED / 2, -0.5))
    scene = [(bed, (0.17, 0.18, 0.20))] + items
    pts = np.vstack([_apply(view, m.vertices - (BED / 2, BED / 2, 0)) for m, _ in scene])
    span = max(np.ptp(pts[:, 0]), np.ptp(pts[:, 1])) * 1.06
    centre = (pts[:, :2].max(0) + pts[:, :2].min(0)) / 2
    scale = size / span
    img = np.full((size, size, 3), 255.0)
    depth = np.full((size, size), -np.inf)
    for mesh, colour in scene:
        v = _apply(view, mesh.vertices - (BED / 2, BED / 2, 0))
        sx = (v[:, 0] - centre[0]) * scale + size / 2
        sy = size / 2 - (v[:, 1] - centre[1]) * scale
        n = mesh.face_normals
        normals = _apply(view, n)
        shade = 0.45 + 0.55 * np.clip(n[:, 0] * light[0] + n[:, 1] * light[1] + n[:, 2] * light[2], 0, 1)
        for f, face in enumerate(mesh.faces):
            if normals[f, 2] <= 0:
                continue
            xs, ys, zs = sx[face], sy[face], v[face, 2]
            x0, x1 = max(int(xs.min()), 0), min(int(xs.max()) + 1, size - 1)
            y0, y1 = max(int(ys.min()), 0), min(int(ys.max()) + 1, size - 1)
            if x1 < x0 or y1 < y0:
                continue
            gx, gy = np.meshgrid(np.arange(x0, x1 + 1) + 0.5, np.arange(y0, y1 + 1) + 0.5)
            d = (ys[1] - ys[2]) * (xs[0] - xs[2]) + (xs[2] - xs[1]) * (ys[0] - ys[2])
            if abs(d) < 1e-12:
                continue
            a = ((ys[1] - ys[2]) * (gx - xs[2]) + (xs[2] - xs[1]) * (gy - ys[2])) / d
            b = ((ys[2] - ys[0]) * (gx - xs[2]) + (xs[0] - xs[2]) * (gy - ys[2])) / d
            c = 1 - a - b
            inside = (a >= 0) & (b >= 0) & (c >= 0)
            if not inside.any():
                continue
            z = a * zs[0] + b * zs[1] + c * zs[2]
            region = depth[y0:y1 + 1, x0:x1 + 1]
            win = inside & (z > region)
            region[win] = z[win]
            img[y0:y1 + 1, x0:x1 + 1][win] = np.array(colour) * 255 * shade[f]
    return np.clip(img, 0, 255).astype(np.uint8)


def png(img):
    h, w, _ = img.shape
    raw = b"".join(b"\x00" + img[y].tobytes() for y in range(h))

    def chunk(kind, data):
        return struct.pack(">I", len(data)) + kind + data + struct.pack(">I", zlib.crc32(kind + data) & 0xFFFFFFFF)
    return (b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", struct.pack(">IIBBBBB", w, h, 8, 2, 0, 0, 0))
            + chunk(b"IDAT", zlib.compress(raw, 9)) + chunk(b"IEND", b""))


def place_objects(meshes, plates):
    """One object per placed part, as Bambu Studio saves them: its plate list keeps a single
    instance per object, so copies of one object can't be spread across plates. Repeated parts
    get numbered names. Returns [(label, part, plate, x, y)] with plate-relative x, y."""
    placed = [(n, p, x, y) for p, (_, rows) in enumerate(plates) for n, x, y in layout(rows, meshes)]
    totals = {n: sum(1 for m, *_ in placed if m == n) for n in PARTS}
    counters = {n: 0 for n in PARTS}
    objects = []
    for n, p, x, y in placed:
        counters[n] += 1
        label = f"{n} {counters[n]}" if totals[n] > 1 else n
        objects.append((label, n, p, x, y))
    return objects


def write_project(out, title, plates, meshes):
    objects = place_objects(meshes, plates)
    # Bambu numbering: each object's mesh (its one part) takes an id, the object the next one.
    ids = [(2 * i + 1, 2 * i + 2, i + 1) for i in range(len(objects))]   # part id, object id, file number

    files = {}
    rels = []
    for (label, n, *_), (part_id, obj_id, num) in zip(objects, ids):
        path = f"3D/Objects/object_{num}.model"
        rels.append(path)
        files[path] = (MODEL_HEAD + " <resources>\n"
                       + mesh_xml(meshes[n], part_id, f"{num << 16:08x}-81cb-4c03-9d28-80fed5dfa1dc")
                       + " </resources>\n <build/>\n</model>\n")

    model = [MODEL_HEAD]
    meta = {"Application": APP, "BambuStudio:3mfVersion": "1", "Title": title,
            "Designer": "CubeSat Classroom", "Description": "AMSAT CubeSatSim v2.0 frame with an Arduino Uno R3 and "
            "Nano 33 BLE Sense payload. Frame by Alan Johnston, KU2Y (AMSAT). GPL-3.0.",
            "License": "GPL-3.0", "CreationDate": DATE, "ModificationDate": DATE}
    model += [f' <metadata name="{k}">{escape(v)}</metadata>\n' for k, v in sorted(meta.items())]
    model.append(" <resources>\n")
    for part_id, obj_id, num in ids:
        model.append(f'  <object id="{obj_id}" p:UUID="{num:08x}-61cb-4c03-9d28-80fed5dfa1dc" type="model">\n'
                     f'   <components>\n    <component p:path="/3D/Objects/object_{num}.model" objectid="{part_id}" '
                     f'p:UUID="{num << 16:08x}-b206-40ff-9872-83e8017abed1" transform="1 0 0 0 1 0 0 0 1 0 0 0"/>\n'
                     "   </components>\n  </object>\n")
    model.append(' </resources>\n <build p:UUID="2c7c17d8-22b5-4d84-8835-1976022ea369">\n')
    for (label, n, p, x, y), (_, obj_id, _) in zip(objects, ids):
        ox, oy = plate_origin(p, len(plates))
        model.append(f'  <item objectid="{obj_id}" p:UUID="{obj_id:08x}-b1ec-4553-aec9-835e5b724bb4" '
                     f'transform="1 0 0 0 1 0 0 0 1 {fmt(ox + x)} {fmt(oy + y)} 0" printable="1"/>\n')
    model.append(" </build>\n</model>\n")
    files["3D/3dmodel.model"] = "".join(model)

    cfg = ['<?xml version="1.0" encoding="UTF-8"?>\n<config>\n']
    for (label, n, *_), (part_id, obj_id, _) in zip(objects, ids):
        settings = {"name": label, "extruder": "1", **PARTS[n][2]}
        cfg.append(f'  <object id="{obj_id}">\n')
        cfg += [f'    <metadata key="{k}" value="{escape(v)}"/>\n' for k, v in settings.items()]
        cfg.append(f'    <metadata face_count="{len(meshes[n].faces)}"/>\n'
                   f'    <part id="{part_id}" subtype="normal_part">\n'
                   f'      <metadata key="name" value="{escape(label)}"/>\n'
                   '      <metadata key="matrix" value="1 0 0 0 0 1 0 0 0 0 1 0 0 0 0 1"/>\n'
                   f'      <mesh_stat face_count="{len(meshes[n].faces)}" edges_fixed="0" degenerate_facets="0" '
                   'facets_removed="0" facets_reversed="0" backwards_edges="0"/>\n'
                   '    </part>\n  </object>\n')
    for p, (plate_name, _) in enumerate(plates):
        cfg.append("  <plate>\n"
                   f'    <metadata key="plater_id" value="{p + 1}"/>\n'
                   f'    <metadata key="plater_name" value="{escape(plate_name)}"/>\n'
                   '    <metadata key="locked" value="false"/>\n'
                   f'    <metadata key="thumbnail_file" value="Metadata/plate_{p + 1}.png"/>\n')
        for (label, n, q, x, y), (_, obj_id, num) in zip(objects, ids):
            if q != p:
                continue
            cfg.append("    <model_instance>\n"
                       f'      <metadata key="object_id" value="{obj_id}"/>\n'
                       '      <metadata key="instance_id" value="0"/>\n'
                       f'      <metadata key="identify_id" value="{1000 + num}"/>\n'
                       "    </model_instance>\n")
        cfg.append("  </plate>\n")
    cfg.append("</config>\n")
    files["Metadata/model_settings.config"] = "".join(cfg)

    for p in range(len(plates)):
        scene = []
        for label, n, q, x, y in objects:
            if q == p:
                m = meshes[n].copy()
                m.apply_translation((x, y, 0))
                scene.append((m, PARTS[n][3]))
        files[f"Metadata/plate_{p + 1}.png"] = png(render(scene, 512))
        files[f"Metadata/plate_{p + 1}_small.png"] = png(render(scene, 128))

    files["[Content_Types].xml"] = (
        '<?xml version="1.0" encoding="UTF-8"?>\n'
        '<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">\n'
        ' <Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>\n'
        ' <Default Extension="model" ContentType="application/vnd.ms-package.3dmanufacturing-3dmodel+xml"/>\n'
        ' <Default Extension="png" ContentType="image/png"/>\n'
        ' <Default Extension="gcode" ContentType="text/x.gcode"/>\n</Types>')
    files["_rels/.rels"] = (
        '<?xml version="1.0" encoding="UTF-8"?>\n'
        '<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">\n'
        ' <Relationship Target="/3D/3dmodel.model" Id="rel-1" Type="http://schemas.microsoft.com/3dmanufacturing/2013/01/3dmodel"/>\n'
        ' <Relationship Target="/Metadata/plate_1.png" Id="rel-2" Type="http://schemas.openxmlformats.org/package/2006/relationships/metadata/thumbnail"/>\n'
        ' <Relationship Target="/Metadata/plate_1.png" Id="rel-4" Type="http://schemas.bambulab.com/package/2021/cover-thumbnail-middle"/>\n'
        ' <Relationship Target="/Metadata/plate_1_small.png" Id="rel-5" Type="http://schemas.bambulab.com/package/2021/cover-thumbnail-small"/>\n'
        '</Relationships>')
    files["3D/_rels/3dmodel.model.rels"] = (
        '<?xml version="1.0" encoding="UTF-8"?>\n'
        '<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">\n'
        + "".join(f' <Relationship Target="/{r}" Id="rel-{i + 1}" Type="http://schemas.microsoft.com/3dmanufacturing/2013/01/3dmodel"/>\n'
                  for i, r in enumerate(rels))
        + "</Relationships>")

    order = ["[Content_Types].xml", "_rels/.rels", "3D/3dmodel.model", "3D/_rels/3dmodel.model.rels"]
    order += sorted(k for k in files if k not in order)
    out.parent.mkdir(exist_ok=True)
    with ZipFile(out, "w") as archive:
        for name in order:
            data = files[name]
            archive.writestr(ZipInfo(name), data.encode() if isinstance(data, str) else data,
                             compress_type=ZIP_DEFLATED)
    check(out, plates, objects, meshes)
    print(f"wrote {out.relative_to(HERE)}")
    for p, (plate_name, _) in enumerate(plates):
        print(f"  plate {p + 1}, {plate_name}: {sum(1 for o in objects if o[2] == p)} parts")


def main():
    meshes = load_parts()
    for name, (title, plates) in PROJECTS.items():
        write_project(HERE / "p1s" / name, title, plates, meshes)


def check(out, plates, objects, meshes):
    """Read the project back the way Bambu Studio does and check the plate split: every object
    on exactly one plate, once, with a single instance, lying inside that plate's area."""
    import re
    with ZipFile(out) as archive:
        model = archive.read("3D/3dmodel.model").decode()
        cfg = archive.read("Metadata/model_settings.config").decode()
    items = re.findall(r'<item objectid="(\d+)"[^>]*transform="([^"]+)"', model)
    counts = {}
    for obj, _ in items:
        counts[obj] = counts.get(obj, 0) + 1
    if any(c != 1 for c in counts.values()) or len(items) != len(objects):
        raise ValueError("every object must have exactly one instance")
    position = {obj: tuple(map(float, t.split()[9:11])) for obj, t in items}
    names = dict(re.findall(r'<object id="(\d+)">\s*<metadata key="name" value="([^"]+)"', cfg))
    seen = set()
    for p, plate in enumerate(re.findall(r"<plate>(.*?)</plate>", cfg, re.S)):
        ox, oy = plate_origin(p, len(plates))
        for obj in re.findall(r'key="object_id" value="(\d+)"', plate):
            if obj in seen:
                raise ValueError(f"{names[obj]} is on more than one plate")
            seen.add(obj)
            label = names[obj]
            part = next(n for lab, n, *_ in objects if lab == label)
            w, d = meshes[part].extents[:2]
            x, y = position[obj]
            if not (ox + w / 2 <= x <= ox + BED - w / 2 and oy + d / 2 <= y <= oy + BED - d / 2):
                raise ValueError(f"{label} on plate {p + 1} lies outside that plate")
    if seen != set(position):
        raise ValueError("some objects are on no plate")


if __name__ == "__main__":
    main()
