"""Generate dimensioned orthographic views from the checked-in Rev B STL meshes."""

from pathlib import Path

import numpy as np
import trimesh

ROOT = Path(__file__).resolve().parent
SCALE = 2.5


def load(part):
    return trimesh.load_mesh(ROOT / "stl" / f"rev-b-{part}.stl")


def outline(mesh, axes, origin, *, face_only=False):
    """Project crease edges. The camera view uses its exterior face at z=0."""
    creases = mesh.face_adjacency_edges[mesh.face_adjacency_angles > np.deg2rad(1)]
    edges = mesh.vertices[creases]
    if face_only:
        edges = edges[np.all(np.isclose(edges[:, :, 2], mesh.bounds[0, 2]), axis=1)]
    points = edges[:, :, axes].copy()
    points -= mesh.bounds[0, list(axes)]
    points[:, :, 0] *= SCALE
    points[:, :, 1] *= -SCALE
    points += origin
    segments = set()
    for first, second in points:
        endpoints = tuple(sorted((tuple(np.round(first, 3)), tuple(np.round(second, 3)))))
        if endpoints[0] != endpoints[1]:
            segments.add(endpoints)
    return " ".join(f"M{x1:g},{y1:g}L{x2:g},{y2:g}" for (x1, y1), (x2, y2) in sorted(segments))


def make_drawing():
    frame, camera = load("rail-frame"), load("camera-plate")
    width, depth, height = frame.extents
    plate_width, _, plate_height = camera.extents
    face = camera.vertices[np.isclose(camera.vertices[:, 2], camera.bounds[0, 2])]
    bore = face[np.linalg.norm(face[:, :2] - plate_width / 2, axis=1) < plate_width / 4]
    diameter = np.ptp(bore[:, 0])
    plan_levels = np.unique(frame.vertices[:, 0].round(4))
    rail_width = plan_levels[(plan_levels > 0) & (plan_levels < width / 2)].max()
    ground = 445
    frame_x, plate_x = 125, 530
    frame_right, frame_top = frame_x + width * SCALE, ground - height * SCALE
    plate_right, plate_top = plate_x + plate_width * SCALE, ground - plate_width * SCALE
    return f"""<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 900 590" role="img" aria-labelledby="title desc">
<title id="title">Rev B CubeSat rail-frame and camera-plate drawing</title>
<desc id="desc">Front elevation of the rail frame and exterior plan of the camera plate, projected from their STL meshes with nominal millimetre dimensions.</desc>
<rect width="900" height="590" fill="#fff"/>
<g font-family="system-ui, sans-serif" fill="#182b41">
<text x="38" y="53" font-size="31" font-weight="700">Rev B · Rail-fit bench model</text>
<text x="38" y="83" font-size="18">Orthographic views from the printable meshes</text>
<path d="M38 103H862" fill="none" stroke="#a4b5c7"/>
<text x="{frame_x}" y="133" font-size="17" font-weight="700">Rail frame · front</text>
<text x="{plate_x}" y="183" font-size="17" font-weight="700">Camera plate · exterior face</text>
<g fill="none" stroke="#182b41" stroke-width="1.7" stroke-linejoin="round">
<path d="{outline(frame, (0, 2), (frame_x, ground))}"/>
<path d="{outline(camera, (0, 1), (plate_x, ground), face_only=True)}"/>
</g>
<g fill="none" stroke="#386497" stroke-width="1.15">
<path d="M{frame_x} {ground+5}v36M{frame_right:g} {ground+5}v36M{frame_x} {ground+30}H{frame_right:g}M{frame_x-6} {frame_top:g}H63M{frame_x-6} {ground}H63M72 {frame_top:g}V{ground}"/>
<path d="M{plate_x} {ground+5}v36M{plate_right:g} {ground+5}v36M{plate_x} {ground+30}H{plate_right:g}"/>
<path d="M120 480l10 -10m240 10 10 -10M525 480l10 -10m223.25 10 10 -10M67 {frame_top+5:g}l10 -10m-10 {height*SCALE:g} 10 -10"/>
<path d="M{plate_x+plate_width*SCALE/2+diameter*SCALE/2:g} {ground-plate_width*SCALE/2:g}h28l23 -35h46"/>
</g>
<g font-size="15" font-weight="700" fill="#294e79">
<text x="{(frame_x+frame_right)/2:g}" y="501" text-anchor="middle">{width:g} mm</text>
<text transform="translate(56 {(frame_top+ground)/2:g}) rotate(-90)" text-anchor="middle">{height:g} mm</text>
<text x="{(plate_x+plate_right)/2:g}" y="501" text-anchor="middle">{plate_width:.1f} mm</text>
<text x="722" y="{plate_top+72:g}">Ø{diameter:.0f} mm</text>
</g>
<path d="M38 523H862" fill="none" stroke="#a4b5c7"/>
<text x="38" y="551" font-size="15">Frame depth {depth:g} mm · Rails {rail_width:g} mm square · Camera plate with locators {plate_height:.1f} mm thick</text>
<text x="38" y="574" font-size="14" fill="#435974">Nominal geometry for bench fit checks · Do not measure from this drawing</text>
</g>
</svg>
"""


if __name__ == "__main__":
    (ROOT / "rev-b-drawing.svg").write_text(make_drawing(), encoding="utf-8")
