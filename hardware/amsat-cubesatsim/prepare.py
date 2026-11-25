"""Pack the AMSAT frame and the printed payload kit into six P1S plates (3MF).

Run generate.py first. Each plate is centered on the 256 mm bed with every part
in its print orientation. GPL-3.0, like the rest of this folder.
"""

from pathlib import Path
from xml.etree.ElementTree import Element, SubElement, tostring
from zipfile import ZIP_DEFLATED, ZipFile, ZipInfo

import numpy as np
import trimesh

HERE = Path(__file__).resolve().parent
BED = 256
GAP = 10
MARGIN = 5
FLIP = trimesh.transformations.rotation_matrix(np.pi, [1, 0, 0])

PLATES = {
    "01-snap-test": [("snap-test-peg", "stl/snap-test-peg.stl", None),
                     ("snap-test-tab", "stl/snap-test-tab.stl", None)],
    "02-sides": [("side", "upstream/cubesatsim_frame_side.v2.0.stl", None),
                 ("side-camera", "upstream/cubesatsim_frame_side_camera.v2.0.stl", None)],
    "03-top-and-bottom": [("top", "upstream/cubesatsim_frame_top_bottom.v2.0.stl", FLIP),
                          ("bottom-with-columns", "stl/bottom-with-columns.stl", None)],
    "04-payload-plate": [("payload-plate", "stl/payload-plate.stl", None),
                         ("lower-clamp", "stl/lower-clamp.stl", None)],
    "05-fit-plate": [("fit-plate", "stl/fit-plate.stl", None),
                     ("lower-clamp", "stl/lower-clamp.stl", None)],
    "06-clamps-and-screws": [("uno-clamp-0", "stl/uno-clamp-0.stl", None),
                             ("uno-clamp-1", "stl/uno-clamp-1.stl", None),
                             ("uno-clamp-2", "stl/uno-clamp-2.stl", None)]
                            + [(f"screw-{i}", "stl/screw.stl", None) for i in range(1, 6)],
}


def pack(plate, parts):
    meshes = []
    for name, path, transform in parts:
        mesh = trimesh.load_mesh(HERE / path)
        if not mesh.is_volume:
            raise ValueError(f"{path}: not a closed solid")
        if transform is not None:
            mesh.apply_transform(transform)
        mesh.apply_translation(-mesh.bounds[0])
        meshes.append((name, mesh))

    width = sum(m.extents[0] for _, m in meshes) + GAP * (len(meshes) - 1)
    x = (BED - width) / 2
    model = Element("model", {"unit": "millimeter",
                              "xmlns": "http://schemas.microsoft.com/3dmanufacturing/core/2015/02"})
    resources = SubElement(model, "resources")
    build = SubElement(model, "build")
    for index, (name, mesh) in enumerate(meshes, 1):
        mesh.apply_translation((x, (BED - mesh.extents[1]) / 2, 0))
        x += mesh.extents[0] + GAP
        if (mesh.bounds[0, :2].min() < MARGIN or mesh.bounds[1, :2].max() > BED - MARGIN
                or mesh.bounds[1, 2] > BED):
            raise ValueError(f"{plate}: {name} does not fit the P1S plate")
        item = SubElement(resources, "object", {"id": str(index), "type": "model", "name": name})
        geometry = SubElement(item, "mesh")
        vertices = SubElement(geometry, "vertices")
        for point in mesh.vertices:
            SubElement(vertices, "vertex", dict(zip("xyz", (f"{v:.5f}" for v in point))))
        triangles = SubElement(geometry, "triangles")
        for face in mesh.faces:
            SubElement(triangles, "triangle", dict(zip(("v1", "v2", "v3"), map(str, face))))
        SubElement(build, "item", {"objectid": str(index)})

    package = {
        "[Content_Types].xml": (
            '<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">'
            '<Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>'
            '<Default Extension="model" ContentType="application/vnd.ms-package.3dmanufacturing-3dmodel+xml"/>'
            '</Types>'
        ),
        "_rels/.rels": (
            '<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">'
            '<Relationship Id="rel0" Target="/3D/3dmodel.model" '
            'Type="http://schemas.microsoft.com/3dmanufacturing/2013/01/3dmodel"/>'
            '</Relationships>'
        ),
        "3D/3dmodel.model": tostring(model, encoding="utf-8", xml_declaration=True),
    }
    folder = HERE / "p1s"
    folder.mkdir(exist_ok=True)
    with ZipFile(folder / f"{plate}.3mf", "w") as output:
        for name, content in package.items():
            output.writestr(ZipInfo(name), content, compress_type=ZIP_DEFLATED)
    print(f"{plate}: {', '.join(n for n, _ in meshes)}")


if __name__ == "__main__":
    for plate, parts in PLATES.items():
        pack(plate, parts)
