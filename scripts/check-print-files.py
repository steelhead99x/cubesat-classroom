#!/usr/bin/env python3
"""Check rebuilt hardware, tolerating minor retriangulation of known print assets."""
from collections import Counter
from io import BytesIO
from pathlib import Path
import math
import struct
import subprocess
import sys
from xml.etree import ElementTree as ET
from zipfile import BadZipFile, ZipFile

import numpy as np
from trimesh.triangles import closest_point

VERTEX_TOL = 0.0005  # mm, known builds differ by up to 0.00032 at vertices
# The two known builds flip a few near-coplanar faces. The largest observed
# distance at their edge midpoints and centroids was 0.00183 mm.
SURFACE_TOL = 0.002  # mm
STLS = {f'hardware/{case}/stl/coupon-latch-{part}.stl'
        for case in ('outdoor-station', 'outdoor-xiao') for part in ('box', 'lid')}
PROJECTS = {
    'hardware/amsat-cubesatsim/p1s/cubesat-test-coupons.3mf': (2,),
    'hardware/outdoor-station/p1s/outdoor-test-coupons.3mf': (2, 3, 5),
    'hardware/outdoor-xiao/p1s/outdoor-xiao-test-coupons.3mf': (3, 4, 6),
}
NS = '{http://schemas.microsoft.com/3dmanufacturing/core/2015/02}'


def require(condition, message):
    if not condition:
        raise ValueError(message)


def stl(data):
    count, = struct.unpack_from('<I', data, 80)
    require(len(data) == 84 + 50 * count, 'invalid binary STL length')
    facets = np.frombuffer(data, dtype=[('normal', '<f4', (3,)),
                                        ('vertices', '<f4', (3, 3)), ('attributes', '<u2')], offset=84)
    require(np.all(facets['attributes'] == 0), 'nonzero STL facet attributes')
    normals = facets['normal'].astype(float)
    require(np.all(np.abs(np.linalg.norm(normals, axis=1) - 1) <= 1e-6), 'invalid STL facet normal')
    a, b, c = facets['vertices'].astype(float).transpose(1, 0, 2)
    winding = np.cross(b - a, c - a)
    # Normals are written before the vertices are rounded to float32.
    require(np.all(np.einsum('ij,ij->i', normals, winding) >=
                   (1 - 1e-6) * np.linalg.norm(winding, axis=1)), 'STL facet normal disagrees with winding')
    return [tuple(map(tuple, triangle)) for triangle in facets['vertices'].tolist()]


def compare_stls(original, rebuilt):
    require(original[:80] == rebuilt[:80], 'STL header changed')
    compare_surfaces(stl(original), stl(rebuilt))


def require_closed(edges, message):
    require(all(n == 1 and edges[b, a] == 1 for (a, b), n in edges.items()), message)


def oriented(triangles):
    """Index by coordinates, retaining winding and checking the closed surface."""
    faces = set()
    edges = Counter()
    for a, b, c in triangles:
        require(all(math.isfinite(q) for p in (a, b, c) for q in p), 'nonfinite vertex')
        require(len({a, b, c}) == 3, 'degenerate triangle')
        face = min((a, b, c), (b, c, a), (c, a, b))
        require(face not in faces, 'duplicate triangle')
        faces.add(face)
        edges.update(((a, b), (b, c), (c, a)))
    require_closed(edges, 'open or inconsistently wound mesh')
    mesh = np.asarray(triangles, dtype=float)
    require(mesh.shape == (len(triangles), 3, 3) and len(triangles), 'empty mesh')
    a, b, c = mesh.transpose(1, 0, 2)
    areas = np.linalg.norm(np.cross(b - a, c - a), axis=1) / 2
    require(np.all(areas > 0), 'zero-area triangle')
    volume = math.fsum(np.einsum('ij,ij->i', a, np.cross(b, c))) / 6
    require(volume > 0, 'nonpositive signed volume')
    return faces, math.fsum(areas), volume


def compare_surfaces(left, right):
    faces_a, area_a, volume_a = oriented(left)
    faces_b, area_b, volume_b = oriented(right)
    require(math.isclose(volume_a, volume_b, rel_tol=1e-7, abs_tol=0.001),
            f'mesh volume changed ({volume_a:.6f} to {volume_b:.6f} mm³)')
    require(math.isclose(area_a, area_b, rel_tol=1e-7, abs_tol=0.001),
            f'mesh surface area changed ({area_a:.6f} to {area_b:.6f} mm²)')

    # Exactly matching oriented triangles already cover each other. Check
    # vertices, edge midpoints and interior centroids on every changed face,
    # in both directions, against the opposite mesh (including unchanged faces).
    for changed, target in ((faces_a - faces_b, right), (faces_b - faces_a, left)):
        if not changed:
            continue
        triangles = np.asarray(target, dtype=float)
        lower = triangles.min(axis=1) - SURFACE_TOL
        upper = triangles.max(axis=1) + SURFACE_TOL
        for face in changed:
            corners = np.asarray(face, dtype=float)
            mids = (corners + np.roll(corners, 1, axis=0)) / 2
            samples = np.vstack((corners, mids, corners.mean(axis=0)))
            for index, point in enumerate(samples):
                tolerance = VERTEX_TOL if index < 3 else SURFACE_TOL
                choices = triangles[np.all((lower <= point) & (point <= upper), axis=1)]
                require(len(choices) > 0, 'surface sample lies outside the opposite mesh')
                nearest = closest_point(choices, np.broadcast_to(point, (len(choices), 3)))
                require(np.min(np.linalg.norm(nearest - point, axis=1)) <= tolerance,
                        f'surface sample moved beyond {tolerance} mm')


def model(data):
    root = ET.fromstring(data)
    require(root.get('unit') == 'millimeter', 'unexpected 3MF units')
    meshes = {}
    for obj in root.findall(f'{NS}resources/{NS}object'):
        mesh = obj.find(f'{NS}mesh')
        require(mesh is not None, 'unexpected 3MF component in mesh file')
        vertices = mesh.find(f'{NS}vertices')
        triangles = mesh.find(f'{NS}triangles')
        require(vertices is not None and triangles is not None, 'incomplete 3MF mesh')
        require(not vertices.attrib and not triangles.attrib
                and all(v.tag == f'{NS}vertex' and set(v.attrib) == {'x', 'y', 'z'} and not len(v) for v in vertices)
                and all(t.tag == f'{NS}triangle' and set(t.attrib) == {'v1', 'v2', 'v3'} and not len(t) for t in triangles),
                'unexpected 3MF vertex or triangle attributes')
        points = [tuple(float(v.get(axis)) for axis in 'xyz') for v in vertices]
        faces = []
        edges = Counter()
        for t in triangles:
            a, b, c = (int(t.get(k)) for k in ('v1', 'v2', 'v3'))
            require(all(0 <= i < len(points) for i in (a, b, c)), 'invalid triangle index')
            faces.append((points[a], points[b], points[c]))
            edges.update(((a, b), (b, c), (c, a)))
        require_closed(edges, 'open or inconsistently wound indexed 3MF mesh')
        key = obj.get('id')
        require(key is not None and key not in meshes, 'missing or duplicate mesh ID')
        meshes[key] = faces
        mesh.remove(vertices)
        mesh.remove(triangles)
    return ET.tostring(root), meshes


def config(data, counts):
    root = ET.fromstring(data)
    for obj in root.findall('object'):
        for part in obj.findall('part'):
            stats = part.find('mesh_stat')
            if stats is not None and (key := part.get('id')) in counts:
                require(int(stats.attrib.pop('face_count')) == counts[key], 'incorrect 3MF part face count')
        total = obj.find('metadata[@face_count]')
        ids = [part.get('id') for part in obj.findall('part')]
        if total is not None and any(key in counts for key in ids):
            yield obj.get('id'), ids, total
    yield root


def compare_projects(original, rebuilt, allowed_parts):
    allowed = {f'3D/Objects/object_{part}.model' for part in allowed_parts}
    with ZipFile(BytesIO(original)) as a, ZipFile(BytesIO(rebuilt)) as b:
        require(len(a.namelist()) == len(b.namelist()) == len(set(a.namelist()))
                and set(a.namelist()) == set(b.namelist()), '3MF archive entries changed')
        counts = [{}, {}]
        settings = 'Metadata/model_settings.config'
        for name in a.namelist():
            old, new = a.read(name), b.read(name)
            if old == new or name == settings:
                continue
            require(name in allowed, f'3MF entry changed: {name}')
            meta_a, meshes_a = model(old)
            meta_b, meshes_b = model(new)
            require(meta_a == meta_b and meshes_a.keys() == meshes_b.keys(),
                    f'3MF object metadata changed: {name}')
            for key in meshes_a:
                compare_surfaces(meshes_a[key], meshes_b[key])
                counts[0][key], counts[1][key] = len(meshes_a[key]), len(meshes_b[key])

        # Only discard derived counts for verified meshes. A combined object
        # count must change by exactly the verified part count delta.
        normalized = [list(config(archive.read(settings), count))
                      for archive, count in zip((a, b), counts)]
        require(len(normalized[0]) == len(normalized[1]), '3MF face count entries changed')
        for old, new in zip(normalized[0][:-1], normalized[1][:-1]):
            require(old[:2] == new[:2], '3MF combined object changed')
            delta = sum(counts[1].get(key, 0) - counts[0].get(key, 0) for key in old[1])
            before, after = (int(entry[2].attrib.pop('face_count')) for entry in (old, new))
            require(after - before == delta, 'incorrect 3MF combined face count')
        require(ET.tostring(normalized[0][-1]) == ET.tostring(normalized[1][-1]),
                '3MF slicer settings changed')


def git_paths(*args):
    return set(filter(None, subprocess.check_output(['git', *args, '-z', '--', 'hardware']).decode().split('\0')))


def main():
    changed = git_paths('diff', '--name-only', 'HEAD') | git_paths('ls-files', '--others', '--exclude-standard')
    failures = []
    for path in sorted(changed):
        try:
            require(path in STLS or path in PROJECTS, 'regenerated file differs')
            original = subprocess.check_output(['git', 'show', f'HEAD:{path}'])
            rebuilt = Path(path).read_bytes()
            if path in STLS:
                compare_stls(original, rebuilt)
            else:
                compare_projects(original, rebuilt, PROJECTS[path])
            print(f'Equivalent print geometry: {path}')
        except (ValueError, OSError, KeyError, BadZipFile, subprocess.CalledProcessError,
                ET.ParseError, struct.error) as exc:
            failures.append(f'{path}: {exc}')
    if failures:
        sys.exit('\n'.join(failures) + '\nRegenerate the changed hardware outputs and review the diff.')


if __name__ == '__main__':
    main()
