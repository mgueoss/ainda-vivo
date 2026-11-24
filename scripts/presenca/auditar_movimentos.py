"""Mede a superfície deformada e os apoios das ações com Blender em segundo plano."""

import json
from pathlib import Path
import sys

import bpy
import numpy as np


LANDMARKS = {
    "foot_left": "dedos_pe.L", "foot_right": "dedos_pe.R",
    "hand_left": "medio.3.L", "hand_right": "medio.3.R",
}


def coordinates(mesh):
    points = np.empty((len(mesh.vertices), 3), np.float32)
    mesh.vertices.foreach_get("co", points.ravel())
    return points


def rig_and_body(scene):
    rig, = (obj for obj in scene.objects if obj.type == "ARMATURE")
    body, = (obj for obj in scene.objects if obj.type == "MESH")
    return rig, body


def mesh_signature(body, rig):
    return {
        "vertices": len(body.data.vertices),
        "faces": len(body.data.polygons),
        "groups": [group.name for group in body.vertex_groups],
        "weights": [tuple((g.group, g.weight) for g in point.groups) for point in body.data.vertices],
        "rig": [(b.name, tuple(b.head_local), tuple(b.tail_local)) for b in rig.data.bones],
    }


def inspect(directory, base):
    scene = bpy.context.scene
    rig, body = rig_and_body(scene)
    signature = mesh_signature(body, rig)
    rest = coordinates(body.data)
    members = [
        body.vertex_groups[max(v.groups, key=lambda group: group.weight).group].name
        for v in body.data.vertices
    ]
    points = {
        key: np.asarray([index for index, name in enumerate(members)
                         if name.endswith(bone[-2:]) and
                         name.startswith(("pe.", "dedos_pe.") if key.startswith("foot") else
                                         ("mao.", "indicador.3", "medio.3", "anelar.3", "minimo.3"))])
        for key, bone in LANDMARKS.items()
    }
    pairs = np.asarray([tuple(edge.vertices) for edge in body.data.edges])
    lengths = np.linalg.norm(rest[pairs[:, 0]] - rest[pairs[:, 1]], axis=1)
    pairs, lengths = pairs[lengths > .002], lengths[lengths > .002]
    results = []
    for clip in json.loads((directory / "movimentos.json").read_text(encoding="utf-8")):
        track = rig.animation_data.nla_tracks[clip["label"]]
        strip = track.strips[0]
        rig.animation_data.action = strip.action
        rig.animation_data.action_slot = strip.action_slot
        frames = round(clip["duration"] * clip["fps"])
        feet, stretch, maximum, height, penetration, by_phase = [], [], [], [], [], []
        stride = clip["speed"]
        for frame in range(2 * frames + 1):
            at = frame / (2 * frames)
            scene.frame_set(frame // 2, subframe=(frame % 2) * .5)
            mesh = body.evaluated_get(bpy.context.evaluated_depsgraph_get()).data
            current = coordinates(mesh)
            assert np.isfinite(current).all(), (clip["label"], frame)
            ratios = np.linalg.norm(current[pairs[:, 0]] - current[pairs[:, 1]], axis=1) / lengths
            stretch.append(float(np.percentile(ratios, 99)))
            maximum.append(float(ratios.max()))
            penetration.append(float(current[:, 2].min()))
            height.append(float(current[:, 2].max()))
            supports = {key: float(current[indices, 2].min()) for key, indices in points.items()}
            markers = {
                key: rig.pose.bones[bone].tail.xy.copy() for key, bone in LANDMARKS.items()
            }
            distance = at * clip["duration"] * (stride[0] + .5 * (stride[1] - stride[0]) * at)
            for marker in markers.values():
                if clip["direction"] == "side":
                    marker.x += distance
                else:
                    marker.y += distance if clip["direction"] == "back" else -distance
            by_phase.append((at, supports, markers))
        for span in clip["stance"]:
            key = span["limb"]
            readings = [(height[key], markers[key])
                        for at, height, markers in by_phase if span["start"] <= at <= span["end"]]
            if len(readings) < 2:
                continue
            positions = [pos for _, pos in readings]
            drift = max((a - b).length for a in positions for b in positions)
            feet.append({
                "apoio": key, "início": span["start"], "fim": span["end"],
                "mínimo_mm": round(min(z for z, _ in readings) * 1000, 2),
                "máximo_mm": round(max(z for z, _ in readings) * 1000, 2),
                "deslize_mm": round(drift * 1000, 2),
            })
        results.append({
            "clipe": clip["label"], "amostras": 2 * frames + 1,
            "mínimo_mm": round(min(penetration) * 1000, 2),
            "quadro_mínimo": penetration.index(min(penetration)) / 2,
            "altura_máxima_m": round(max(height), 3),
            "alongamento_p99_pior_quadro": round(max(stretch), 3),
            "alongamento_máximo": round(max(maximum), 3),
            "apoios": feet,
        })
    bpy.ops.wm.open_mainfile(filepath=str(base), load_ui=False)
    old_rig, old_body = rig_and_body(bpy.context.scene)
    old = mesh_signature(old_body, old_rig)
    assert old == signature and np.array_equal(rest, coordinates(old_body.data)), "Malha, pesos ou rig mudaram"
    return {
        "blender": bpy.app.version_string, "mesma_malha_rig_pesos_do_original": True,
        "frequência_de_amostragem_hz": 60, "clipes": results,
    }


def main():
    directory, base, report = (Path(value).resolve() for value in sys.argv[sys.argv.index("--")+1:])
    bpy.ops.wm.open_mainfile(filepath=str(directory / "presenca.blend"), load_ui=False)
    result = inspect(directory, base)
    report.write_text(json.dumps(result, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    for take in result["clipes"]:
        useful = tuple((s["mínimo_mm"], s["máximo_mm"], s["deslize_mm"]) for s in take["apoios"])
        print(take["clipe"], take["mínimo_mm"], take["alongamento_p99_pior_quadro"], useful)


if __name__ == "__main__":
    main()
