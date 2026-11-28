"""Regression fixtures for the print freshness geometry exception."""
from pathlib import Path
from runpy import run_path
import struct
import unittest
from xml.etree import ElementTree as ET
from zipfile import ZipFile

checker = run_path(str(Path(__file__).with_name('check-print-files.py')))
compare = checker['compare_surfaces']
hardware = Path(__file__).resolve().parents[1] / 'hardware'


def cube(alternate=False):
    corners = [(0, 0, 0), (1, 0, 0), (1, 1, 0), (0, 1, 0),
               (0, 0, 1), (1, 0, 1), (1, 1, 1), (0, 1, 1)]
    faces = [(0, 3, 2, 1), (4, 5, 6, 7), (0, 1, 5, 4),
             (1, 2, 6, 5), (2, 3, 7, 6), (3, 0, 4, 7)]
    triangles = []
    for i, j, k, l in faces:
        first, second = ((i, j, l), (j, k, l)) if alternate else ((i, j, k), (i, k, l))
        triangles.extend(tuple(corners[p] for p in triangle) for triangle in (first, second))
    return triangles


class PrintFilesTest(unittest.TestCase):
    def test_equivalent_diagonals(self):
        compare(cube(), cube(alternate=True))

    def test_moved_surface_of_same_area_and_volume(self):
        shifted = [tuple((x + 0.001, y, z) for x, y, z in triangle) for triangle in cube()]
        with self.assertRaisesRegex(ValueError, 'surface sample'):
            compare(cube(), shifted)

    def test_open_mesh_and_reversed_winding(self):
        original = cube()
        with self.assertRaisesRegex(ValueError, 'open or inconsistently wound'):
            compare(original, original[1:])
        flipped = [original[0][::-1], *original[1:]]
        with self.assertRaisesRegex(ValueError, 'open or inconsistently wound'):
            compare(original, flipped)
        inside_out = [triangle[::-1] for triangle in original]
        with self.assertRaisesRegex(ValueError, 'nonpositive signed volume'):
            compare(original, inside_out)

    def test_stl_metadata_and_normals(self):
        original = (hardware / 'outdoor-station/stl/coupon-latch-box.stl').read_bytes()
        checker['compare_stls'](original, original)
        for change, message in (
            (b'X' + original[1:], 'STL header changed'),
            (original[:132] + b'\xff\xff' + original[134:], 'nonzero STL facet attributes'),
            (original[:84] + struct.pack('<f', float('nan')) + original[88:], 'invalid STL facet normal'),
        ):
            with self.subTest(message=message), self.assertRaisesRegex(ValueError, message):
                checker['compare_stls'](original, change)
        opposite = tuple(-x for x in struct.unpack_from('<3f', original, 84))
        with self.assertRaisesRegex(ValueError, 'normal disagrees with winding'):
            checker['compare_stls'](original, original[:84] + struct.pack('<3f', *opposite) + original[96:])

    def test_disconnected_3mf_vertex(self):
        with ZipFile(hardware / 'amsat-cubesatsim/p1s/cubesat-test-coupons.3mf') as archive:
            original = archive.read('3D/Objects/object_2.model')
        checker['model'](original)
        root = ET.fromstring(original)
        ns = checker['NS']
        mesh = root.find(f'.//{ns}mesh')
        vertices = mesh.find(f'{ns}vertices')
        first = mesh.find(f'{ns}triangles/{ns}triangle')
        original_vertex = vertices[int(first.get('v1'))]
        first.set('v1', str(len(vertices)))
        ET.SubElement(vertices, original_vertex.tag, original_vertex.attrib)
        with self.assertRaisesRegex(ValueError, 'open or inconsistently wound indexed 3MF mesh'):
            checker['model'](ET.tostring(root))


if __name__ == '__main__':
    unittest.main()
