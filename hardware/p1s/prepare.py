"""Build the Rev A case's three millimetre-scale P1S test plates from the STLs."""

from pathlib import Path
from xml.etree.ElementTree import Element, SubElement, tostring
from zipfile import ZIP_DEFLATED, ZipFile, ZipInfo

import numpy as np
import trimesh

HERE = Path(__file__).resolve().parent
PRINTER_VOLUME = np.array([256, 256, 256])
EDGE_MARGIN = 10

PLATES = [
    ("01-m3-fit-coupon", "m3-fit-coupon", (48, 14, 10)),
    ("02-case-shell", "bench-shell", (100, 100, 109.5)),
    ("03-camera-lid", "camera-lid", (100, 100, 6)),
]


def pack(plate, part, dimensions):
    mesh = trimesh.load_mesh(HERE.parent / "stl" / f"{part}.stl")
    if not mesh.is_volume or not np.allclose(mesh.extents, dimensions):
        raise ValueError(f"{part}: invalid STL or unexpected dimensions")

    # Keep the print orientation and center the part on the 256 mm bed.
    center = mesh.bounds.mean(axis=0)
    xy = PRINTER_VOLUME[:2] / 2 - center[:2]
    mesh.apply_translation((*xy, -mesh.bounds[0, 2]))
    if (np.any(mesh.bounds[0, :2] < EDGE_MARGIN)
            or np.any(mesh.bounds[1, :2] > PRINTER_VOLUME[:2] - EDGE_MARGIN)):
        raise ValueError(f"{part}: footprint exceeds the P1S plate margin")
    if mesh.bounds[1, 2] >= PRINTER_VOLUME[2]:
        raise ValueError(f"{part}: part is taller than the P1S print volume")

    model = Element("model", {
        "unit": "millimeter",
        "xmlns": "http://schemas.microsoft.com/3dmanufacturing/core/2015/02",
    })
    resources = SubElement(model, "resources")
    item = SubElement(resources, "object", {"id": "1", "type": "model", "name": part})
    geometry = SubElement(item, "mesh")
    vertices = SubElement(geometry, "vertices")
    for point in mesh.vertices:
        SubElement(vertices, "vertex", dict(zip("xyz", (f"{v:.7f}" for v in point))))
    triangles = SubElement(geometry, "triangles")
    for face in mesh.faces:
        SubElement(triangles, "triangle", dict(zip(("v1", "v2", "v3"), map(str, face))))
    SubElement(SubElement(model, "build"), "item", {"objectid": "1"})

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
    with ZipFile(HERE / f"{plate}.3mf", "w") as output:
        for name, content in package.items():
            output.writestr(ZipInfo(name), content, compress_type=ZIP_DEFLATED)

if __name__ == "__main__":
    for plate, part, dimensions in PLATES:
        pack(plate, part, dimensions)
