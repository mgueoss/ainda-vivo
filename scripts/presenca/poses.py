"""Rig de deformação e poses estáticas, com um aceno curto para testar o player."""

from math import radians, sqrt

import bpy
from mathutils import Quaternion, Vector


def rig_for(sculpt, body):
    armature = bpy.data.armatures.new("Presença • esqueleto de deformação")
    rig = bpy.data.objects.new("Presença · rig", armature)
    bpy.context.collection.objects.link(rig)
    bpy.context.view_layer.objects.active = rig
    body.select_set(False)
    rig.select_set(True)
    rig.show_in_front = True
    armature.display_type = "OCTAHEDRAL"
    bpy.ops.object.mode_set(mode="EDIT")
    for name, (head, tail, parent) in sculpt.bones.items():
        bone = armature.edit_bones.new(name)
        bone.head, bone.tail = head, tail
        if parent:
            bone.parent = armature.edit_bones[parent]
    bpy.ops.object.mode_set(mode="OBJECT")
    skeleton = body.modifiers.new("Presença • deformação", "ARMATURE")
    skeleton.object = rig
    # O Bevy usa a mesma combinação linear de pesos exportada no GLB.
    skeleton.use_deform_preserve_volume = False
    body.parent = rig
    return rig


def reset(rig):
    for bone in rig.pose.bones:
        bone.rotation_mode = "QUATERNION"
        bone.location = (0, 0, 0)
        bone.rotation_quaternion = (1, 0, 0, 0)
        bone.scale = (1, 1, 1)
    bpy.context.view_layer.update()


def rotate(rig, name, axis, degrees):
    bone = rig.pose.bones[name]
    bone.rotation_quaternion = bone.rotation_quaternion @ Quaternion(Vector(axis), radians(degrees))


def assign(rig, name, head, direction, roll=0):
    bone = rig.pose.bones[name]
    ref = bone.bone.matrix_local
    direction = Vector(direction).normalized()
    angle = (ref.to_3x3() @ Vector((0, 1, 0))).rotation_difference(direction)
    alignment = Quaternion(direction, radians(roll)) @ angle @ ref.to_quaternion()
    goal = alignment.to_matrix().to_4x4()
    goal.translation = Vector(head)
    bone.matrix = goal
    bpy.context.view_layer.update()
    return bone.tail.copy()


def aim(rig, name, target, roll=0):
    head = rig.pose.bones[name].head.copy()
    return assign(rig, name, head, Vector(target) - head, roll)


def joint(root, tip, a, b, pole):
    offset = tip - root
    distance = min(max(offset.length, abs(a - b) + .001), a + b - .001)
    direction = offset.normalized()
    along = (.5 / distance) * (distance ** 2 + a ** 2 - b ** 2)
    pole = Vector(pole) - root
    bend = (pole - pole.dot(direction) * direction).normalized()
    return root + direction * along + bend * sqrt(max(0, a * a - along * along)), root + direction * distance


def align_limb(rig, upper, lower, tip, pole, roll=0):
    first, second = rig.pose.bones[upper], rig.pose.bones[lower]
    elbow, end = joint(first.head.copy(), Vector(tip), first.length, second.length, pole)
    aim(rig, upper, elbow, 0)
    aim(rig, lower, end, roll)


def inspecting(rig):
    rotate(rig, "pescoco", (0, 0, 1), -6)
    rotate(rig, "cabeca", (0, 0, 1), -23)
    rotate(rig, "cabeca", (0, 1, 0), 13)


def peeking(rig):
    rotate(rig, "lombar", (0, 0, 1), -9)
    rotate(rig, "torax", (0, 0, 1), -14)
    rotate(rig, "peito", (0, 1, 0), 18)
    rotate(rig, "pescoco", (0, 0, 1), -10)
    rotate(rig, "cabeca", (0, 1, 0), -13)
    bpy.context.view_layer.update()
    align_limb(rig, "braco.L", "antebraco.L", (.44, -.24, 1.39), (.61, .03, 1.21))
    aim(rig, "mao.L", (.44, -.34, 1.49), -21)


def quadruped(rig):
    tail = assign(rig, "pelvis", (0, .35, .733), (-.035, -.97, .10))
    tail = assign(rig, "lombar", tail, (-.03, -.99, .08))
    tail = assign(rig, "torax", tail, (.06, -.98, -.20))
    tail = assign(rig, "peito", tail, (.04, -.91, -.43))
    tail = assign(rig, "pescoco", tail, (-.03, -.80, .54))
    assign(rig, "cabeca", tail, (.09, -.31, .94), 8)
    for s, suffix in ((1, ".L"), (-1, ".R")):
        # O apoio fica adiantado ao ombro e os cotovelos dobram para trás.
        rotate(rig, "clavicula" + suffix, (1, 0, 0), 5)
        bpy.context.view_layer.update()
        align_limb(rig, "braco" + suffix, "antebraco" + suffix,
                   (s * .275, -.503 + s * .043, .047), (s * .56, -.15, .59), s * 6)
        wrist = rig.pose.bones["mao" + suffix].head.copy()
        hand_length = rig.pose.bones["mao" + suffix].length
        drop = min(.78, max(0, (wrist.z - .006) / hand_length))
        assign(rig, "mao" + suffix, wrist, (s * .11, -sqrt(1 - drop * drop) * .994, -drop))
        for finger in ("indicador", "medio", "anelar", "minimo"):
            bone = rig.pose.bones[f"{finger}.1{suffix}"]
            assign(rig, bone.name, bone.head.copy(), (s * (.02 if finger == "minimo" else 0), -.995, -.02))
            for part in (2, 3):
                bone = rig.pose.bones[f"{finger}.{part}{suffix}"]
                assign(rig, bone.name, rig.pose.bones[f"{finger}.{part - 1}{suffix}"].tail.copy(),
                       (s * .015, -.99, -.12 if part == 2 else -.075))
        align_limb(rig, "coxa" + suffix, "canela" + suffix,
                   (s * .223, .82 + s * .075, .116), (s * .36, -.06, .41), s * 4)
        ankle = rig.pose.bones["pe" + suffix].head.copy()
        aim(rig, "pe" + suffix, ankle + Vector((s * .032, -.158, -.067)))


def farewell(rig, angle=0):
    rotate(rig, "cabeca", (0, 0, 1), -13)
    align_limb(rig, "braco.R", "antebraco.R", (-.49 + angle * .001, -.195, 1.77),
               (-.63, -.22, 1.28), -18)
    wrist = rig.pose.bones["mao.R"].head.copy()
    aim(rig, "mao.R", wrist + Vector((-.029 - angle * .003, -.021, .130)), 4 + angle)
    for name, bend in (("indicador", 4), ("medio", 8), ("anelar", 13), ("minimo", 21)):
        rotate(rig, f"{name}.2.R", (1, 0, 0), bend)


def key_pose(rig, frame):
    for bone in rig.pose.bones:
        for channel in ("location", "rotation_quaternion", "scale"):
            bone.keyframe_insert(data_path=channel, frame=frame, group=bone.name)


def build_actions(rig):
    bpy.context.scene.render.fps = 30
    rig.animation_data_create()
    clips = [
        ("01 - Imovel", [(1, lambda _: None), (31, lambda _: None)]),
        ("02 - Cabeca inclinada", [(1, inspecting), (31, inspecting)]),
        ("03 - Espiando", [(1, peeking), (31, peeking)]),
        ("04 - Apoio quadrupede", [(1, quadruped), (31, quadruped)]),
        ("05 - Despedida", [(1, farewell), (31, farewell)]),
        ("06 - Neutra do rig", [(1, lambda _: None), (31, lambda _: None)]),
        ("07 - Aceno de teste", [(1, farewell), (21, lambda r: farewell(r, -16)),
                                (47, lambda r: farewell(r, 19)), (69, farewell)]),
    ]
    for name, frames in clips:
        action = bpy.data.actions.new(name)
        action.use_fake_user = True
        rig.animation_data.action = action
        for frame, pose in frames:
            reset(rig)
            pose(rig)
            bpy.context.view_layer.update()
            key_pose(rig, frame)
        slot = rig.animation_data.action_slot
        rig.animation_data.action = None
        track = rig.animation_data.nla_tracks.new()
        track.name = name
        strip = track.strips.new(name, 1, action)
        strip.action_slot = slot
        track.mute = True
    reset(rig)
    bpy.context.scene.frame_set(1)


def audit_poses(rig, body):
    """Registra apoios reais da malha deformada e aponta estiramentos locais."""
    from collections import defaultdict
    import json

    world = bpy.context.evaluated_depsgraph_get()
    rest = [v.co.copy() for v in body.data.vertices]
    edges = [tuple(e.vertices) for e in body.data.edges]
    weights = []
    for v in body.data.vertices:
        member = max(v.groups, key=lambda g: g.weight)
        name = body.vertex_groups[member.group].name
        weights.append(name)
    audit = {}
    for track in rig.animation_data.nla_tracks:
        strip = track.strips[0]
        rig.animation_data.action = strip.action
        rig.animation_data.action_slot = strip.action_slot
        for at in ([1, 21, 47, 69] if track.name.startswith("07") else [1]):
            bpy.context.scene.frame_set(at)
            obj = body.evaluated_get(world)
            result = [obj.matrix_world @ v.co for v in obj.data.vertices]
            support = defaultdict(list)
            for i, name in enumerate(weights):
                if name.startswith(("pe.", "dedos_pe.")) or any(
                    name.startswith(digit) for digit in ("medio.3", "indicador.3", "anelar.3", "minimo.3")
                ):
                    key = ("pe" if name.startswith(("pe.", "dedos_pe.")) else "mao") + name[-2:]
                    support[key].append(result[i].z)
            scored = sorted(((result[i] - result[j]).length / (rest[i] - rest[j]).length, i, j)
                            for i, j in edges if (rest[i] - rest[j]).length > .002)
            ratios = [r for r, _, _ in scored]
            if track.name.startswith(("03", "04")):
                for r, i, j in scored[-3:]:
                    print(f"POSE_EDGE {track.name}: {r:.2f} {weights[i]} {weights[j]} "
                          f"z={rest[i].z:.3f} / {rest[j].z:.3f}")
            audit[f"{track.name} / {at}"] = {
                "z_m": [round(min(v.z for v in result), 4), round(max(v.z for v in result), 4)],
                "apoio_z_m": {key: round(min(values), 4) for key, values in support.items()},
                "alongamento_aresta_p99": round(ratios[int(len(ratios) * .99)], 3),
                "alongamento_aresta_max": round(ratios[-1], 3),
            }
    rig.animation_data.action = None
    bpy.context.scene.frame_set(1)
    reset(rig)
    print("POSES_AUDIT " + json.dumps(audit, ensure_ascii=False))
