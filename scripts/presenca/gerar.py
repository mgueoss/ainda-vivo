"""blender --background --factory-startup --python-exit-code 1 --python scripts/presenca/gerar.py"""

import json
from pathlib import Path
import sys

import bpy
from mathutils import Vector

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
from anatomia import anatomy
from poses import audit_poses, build_actions, rig_for
from superficie import material_for


def audit_mesh(body, rig):
    from collections import Counter, defaultdict
    mesh = body.data
    mesh.calc_loop_triangles()
    adjacency = defaultdict(set)
    counts = Counter()
    for face in mesh.polygons:
        counts.update(face.edge_keys)
    for a, b in counts:
        adjacency[a].add(b)
        adjacency[b].add(a)
    unseen = set(range(len(mesh.vertices)))
    components = 0
    while unseen:
        components += 1
        stack = [unseen.pop()]
        while stack:
            for other in adjacency[stack.pop()] & unseen:
                unseen.remove(other)
                stack.append(other)
    height = max(v.co.z for v in mesh.vertices) - min(v.co.z for v in mesh.vertices)
    audit = {
        "blender": bpy.app.version_string, "triangulos": len(mesh.loop_triangles), "vertices": len(mesh.vertices),
        "altura_m": round(height, 4), "componentes": components,
        "arestas_nao_manifold": sum(count != 2 for count in counts.values()),
        "ossos": len(rig.data.bones),
        "pesos_por_vertice_max": max(len(v.groups) for v in mesh.vertices),
        "erro_max_normalizacao": max(abs(1 - sum(g.weight for g in v.groups)) for v in mesh.vertices),
    }
    print("MESH_AUDIT " + json.dumps(audit), flush=True)
    assert 8000 <= audit["triangulos"] <= 20000 and 2.17 <= height <= 2.23
    assert components == 1 and audit["arestas_nao_manifold"] == 0
    assert audit["erro_max_normalizacao"] < .00001 and audit["pesos_por_vertice_max"] <= 4


def main():
    # A raiz opcional permite preparar e validar uma geração antes de substituir o recurso.
    args = sys.argv[sys.argv.index("--") + 1:] if "--" in sys.argv else []
    root = Path(args[0]).resolve() if args else HERE.parents[1]
    source = root / "art" / "presenca"
    export = root / "assets" / "models" / "presenca.glb"
    source.mkdir(parents=True, exist_ok=True)
    export.parent.mkdir(parents=True, exist_ok=True)
    bpy.ops.wm.read_factory_settings(use_empty=True)
    bpy.context.scene.unit_settings.system = "METRIC"
    bpy.context.scene.render.fps = 30
    bpy.context.scene.frame_end = 69
    bpy.context.preferences.filepaths.save_version = 0
    sculpt = anatomy()
    body = sculpt.build()
    rig = rig_for(sculpt, body)
    audit_mesh(body, rig)
    material_for(body, source / "texturas")
    build_actions(rig)
    audit_poses(rig, body)
    bpy.context.view_layer.objects.active = body
    bpy.ops.object.select_all(action="DESELECT")
    body.select_set(True)
    for screen in bpy.data.screens:
        for area in screen.areas:
            if area.type == "VIEW_3D":
                space = area.spaces.active
                space.shading.type = "SOLID"
                space.shading.color_type = "MATERIAL"
                space.shading.show_shadows = True
                space.shading.show_cavity = True
                space.overlay.show_floor = False
                space.overlay.show_extras = False
                view = space.region_3d
                view.view_location = (0, 0, 1.09)
                view.view_distance = 3.35
                view.view_rotation = Vector((.88, -2.85, .37)).to_track_quat("Z", "Y")
                view.view_perspective = "ORTHO"
    body["altura"] = "2,20 m"
    body["origem"] = "Estação 07 • primeiro estudo da Presença"
    bpy.ops.wm.save_as_mainfile(filepath=str(source / "presenca.blend"), compress=True)
    bpy.ops.export_scene.gltf(
        filepath=str(export), export_format="GLB", export_texcoords=True, export_normals=True,
        export_tangents=True, export_materials="EXPORT", export_skins=True, export_animations=True,
        export_animation_mode="ACTIONS", export_all_influences=False, export_apply=False,
        export_extras=True,
    )
    print(f"ARQUIVOS {source / 'presenca.blend'} | {export}", flush=True)


if __name__ == "__main__":
    main()
