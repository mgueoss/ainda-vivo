"""Abre o .blend da dev.1 e grava as ações, o GLB e os metadados do Dev Lab."""

from dataclasses import dataclass
import json
from math import cos, pi, radians, sin
from pathlib import Path
import struct
import sys

import bpy
from mathutils import Quaternion, Vector

sys.path.insert(0, str(Path(__file__).resolve().parent))
from poses import align_limb, assign, farewell, peeking, reset, rotate

FPS = 30
SIDES = ((1, ".L", "left"), (-1, ".R", "right"))
SPINE = ("pelvis", "lombar", "torax", "peito", "pescoco", "cabeca")
QUAD_SPINE = ((-.04, -.99, .07), (-.035, -.99, .01), (.045, -.94, -.25),
              (.025, -.94, -.30), (-.025, -.81, .51), (.04, -.46, .88))
QUAD_WRIST = (.272, -.380, .059)
QUAD_ANKLE = (.239, .77, .117)
UP = (0, 0, 1)
RIGHT = (1, 0, 0)
BONE_AXIS = (0, 1, 0)


def mix(a, b, t):
    return a + (b - a) * t


def ease(t):
    t = min(1.0, max(0.0, t))
    return t * t * (3 - 2 * t)


def interval(t, a, b):
    return ease((t - a) / (b - a))


def envelope(t, up, down, end=1.0):
    return interval(t, 0, up) * (1 - interval(t, down, end))


def update():
    bpy.context.view_layer.update()


def rest(rig, name):
    return rig.data.bones[name].head_local.copy()


def anchor_pelvis(rig, offset):
    bone = rig.pose.bones["pelvis"]
    bone.location = bone.bone.matrix_local.to_3x3().inverted() @ Vector(offset)


def leg(rig, suffix, ankle, yaw=0.0):
    s = 1 if suffix == ".L" else -1
    align_limb(rig, "coxa" + suffix, "canela" + suffix, ankle,
               (s * .38, -.69, .51), s * 2)
    foot = rig.pose.bones["pe" + suffix]
    matrix = (Quaternion(UP, yaw) @ foot.bone.matrix_local.to_quaternion()).to_matrix().to_4x4()
    matrix.translation = foot.head
    foot.matrix = matrix
    update()


def arm(rig, suffix, target, fold, lift=0.0):
    s = 1 if suffix == ".L" else -1
    hand = rig.pose.bones["mao" + suffix]
    pole = rest(rig, "antebraco" + suffix).lerp(Vector((s * .54, -.005, .60)), fold)
    align_limb(rig, "braco" + suffix, "antebraco" + suffix, target, pole, s * fold * 5)
    wrist = hand.head.copy()
    original = hand.bone.tail_local - hand.bone.head_local
    # O último segmento chega com a polpa dos dedos, seguido pelo punho.
    placed = Vector((s * .035, -.946, -.322 - .52 * lift))
    assign(rig, hand.name, wrist, original.normalized().lerp(placed, fold), s * fold * 6)
    if fold < .001:
        return
    for digit in ("indicador", "medio", "anelar", "minimo"):
        for index in (1, 2, 3):
            bone = rig.pose.bones[f"{digit}.{index}{suffix}"]
            relaxed = bone.matrix.to_3x3() @ Vector(BONE_AXIS)
            support = Vector((s * (.023 if digit == "minimo" else .008), -.995,
                              (-.03, -.105, -.067)[index - 1] - 1.4 * lift))
            assign(rig, bone.name, bone.head.copy(), relaxed.lerp(support, fold))


def human(rig, breath=0, yaw=0, roll=0, nod=0, torso=0, lean=0, dip=0):
    anchor_pelvis(rig, (lean * .11, -.006 * nod, dip))
    rotate(rig, "lombar", RIGHT, 1.15 * breath + .12 * nod)
    rotate(rig, "torax", BONE_AXIS, .6 * torso)
    rotate(rig, "peito", BONE_AXIS, .4 * torso)
    rotate(rig, "peito", RIGHT, -1.5 * breath - .24 * nod)
    rotate(rig, "peito", UP, lean * 15)
    rotate(rig, "pescoco", BONE_AXIS, yaw * .24)
    rotate(rig, "cabeca", BONE_AXIS, yaw * .76)
    rotate(rig, "pescoco", RIGHT, nod * .22)
    rotate(rig, "cabeca", RIGHT, nod * .78)
    rotate(rig, "pescoco", UP, roll * .21)
    rotate(rig, "cabeca", UP, roll * .79)


def lower(rig, t, aborted=False, reversing=False):
    phase = 1 - t if reversing else t
    if aborted:
        phase = .62 * envelope(t, .50, .57)
    fold = interval(phase, .21, .84)
    # Tronco e quadris descem antes de transferir peso pelas mãos.
    pelvis = rest(rig, "pelvis").lerp(Vector((.009, .32, .632)), interval(phase, .17, .91))
    head_tuck = 27 * envelope(phase, .22, .45, .86)
    tail = pelvis
    for index, name in enumerate(SPINE):
        bone = rig.data.bones[name]
        direction = (bone.tail_local - bone.head_local).normalized()
        target = direction.lerp(Vector(QUAD_SPINE[index]).normalized(), fold)
        if name in ("pescoco", "cabeca"):
            target = Quaternion(RIGHT, radians(head_tuck)) @ target
        tail = assign(rig, name, tail, target)
    for s, suffix, _ in SIDES:
        reach = interval(phase, .19 + .035 * s, .73 + .023 * s)
        ankle_to = Vector((s * QUAD_ANKLE[0], QUAD_ANKLE[1] + s * .041, QUAD_ANKLE[2]))
        step_from = .63 if s == 1 else .77
        step_to = step_from + (.19 if s == 1 else .22)
        step = interval(phase, step_from, step_to)
        ankle = rest(rig, "pe" + suffix).lerp(ankle_to, step)
        ankle.z += .118 * sin(pi * step) ** 2
        pole = rest(rig, "canela" + suffix).lerp(Vector((s * .38, -.69, .51)), fold)
        align_limb(rig, "coxa" + suffix, "canela" + suffix, ankle, pole, s * 2 * fold)
        foot = rig.pose.bones["pe" + suffix]
        assign(rig, foot.name, foot.head.copy(), foot.bone.tail_local - foot.bone.head_local)
        target = rest(rig, "mao" + suffix).lerp(Vector((
            s * QUAD_WRIST[0], QUAD_WRIST[1] + s * .036, QUAD_WRIST[2],
        )), reach)
        target.y -= .13 * sin(pi * reach) ** 2
        arm(rig, suffix, target, interval(phase, .26, .69))


@dataclass(frozen=True)
class Gait:
    seconds: float
    start: float
    end: float
    leads: tuple
    stance: float
    lift: float
    quad: bool = False
    direction: str = "forward"
    bank: float = 0.0
    cycle: float = 1

    def __post_init__(self):
        object.__setattr__(self, "seconds", round(self.seconds * FPS) / FPS)

    def travel(self, time):
        within = min(1, max(0, time))
        integral = self.start * within + .5 * (self.end - self.start) * within * within
        return self.seconds * (integral + self.start * min(0,time) + self.end * max(0,time-1))

    def points(self, t, limb):
        # O alvo local compensa exatamente o avanço, durante toda a fase de apoio.
        lead = self.leads[limb]
        phase = (t * self.cycle + lead) % 1
        origin = t - phase / self.cycle
        feet_span = self.stance / self.cycle
        if phase < self.stance:
            along = self.travel(origin + feet_span * .5) - self.travel(t)
            return along, 0.0
        swing = (phase - self.stance) / (1 - self.stance)
        lift_off = self.travel(origin + feet_span * .5)
        step_on = self.travel(origin + (1 + self.stance * .5) / self.cycle)
        along = mix(lift_off, step_on, ease(swing)) - self.travel(t)
        return along, self.lift * sin(pi * swing) ** 2

    def pose(self, rig, t):
        if self.quad:
            self.crawl(rig, t)
        else:
            self.walk(rig, t)

    def walk(self, rig, t):
        angle = 2 * pi * t * self.cycle
        stride = sin(angle)
        sway = .023 * sin(angle - .24)
        anchor_pelvis(rig, (sway, .005, -.045 - .034 * self.start + .013 * cos(angle * 2 + .37)))
        rotate(rig, "pelvis", BONE_AXIS, 3.1 * stride)
        rotate(rig, "lombar", RIGHT, -2 - .67 * self.start)
        rotate(rig, "torax", BONE_AXIS, -4.2 * stride)
        rotate(rig, "cabeca", BONE_AXIS, 1.9 * stride)
        rotate(rig, "cabeca", UP, -2.3 + cos(angle) * 1.1)
        rotate(rig, "braco.R", RIGHT, -3 + 8.8 * stride)
        rotate(rig, "antebraco.R", RIGHT, -5 * max(0, stride))
        rotate(rig, "braco.L", RIGHT, 2 - 2.6 * stride)
        if self.direction == "side":
            rotate(rig, "cabeca", BONE_AXIS, 11)
            rotate(rig, "peito", UP, 5)
        update()
        for i, (s, suffix, _) in enumerate(SIDES):
            along, height = self.points(t, i)
            ankle = rest(rig, "pe" + suffix)
            if self.direction == "side":
                ankle.x += along
                ankle.y -= .04 + s * .047
            else:
                ankle.y -= (-1 if self.direction == "back" else 1) * along
            ankle.z += height
            leg(rig, suffix, ankle)

    def crawl(self, rig, t):
        z = 2 * pi * self.cycle * t
        drive = sin(2 * z - .4)
        spine = Vector((.009 + .027 * sin(z), .32 + .035 * cos(2 * z), .617 + .028 * drive))
        for i, (name, direction) in enumerate(zip(SPINE, QUAD_SPINE)):
            x, y, up = direction
            x += .08 * sin(z + i * .65) * (1 - i / 7)
            up += ((.105 * sin(2 * z - i * .74)) if i < 4 else (.052 * cos(2 * z - .7)))
            x += self.bank * (.30 if i < 5 else .41)
            spine = assign(rig, name, spine, (x, y, up), self.bank * (7 if i < 4 else -4))
        backward = -1 if self.direction == "back" else 1
        for i, (s, suffix, _) in enumerate(SIDES):
            along, heel = self.points(t, i)
            ankle = (s * QUAD_ANKLE[0], QUAD_ANKLE[1] + s * .041 - backward * along,
                     QUAD_ANKLE[2] + heel)
            leg(rig, suffix, ankle)
            along, lift = self.points(t, i + 2)
            target = (s * QUAD_WRIST[0], QUAD_WRIST[1] + s * .036 - backward * along,
                      QUAD_WRIST[2] + lift)
            arm(rig, suffix, target, 1, lift)

    def contacts(self):
        points = []
        for lead, name in zip(self.leads, ("foot_left", "foot_right", "hand_left", "hand_right")):
            for stride in range(int(self.cycle) + 3):
                hit = (stride - lead) / self.cycle
                release = hit + self.stance / self.cycle
                if hit >= 1 or release <= 0:
                    continue
                points.append((max(0, hit), min(1, release), name, hit >= 0))
        return points


def lean(rig, t, contact=False):
    force = envelope(t, .26, .49)
    human(rig, yaw=-5 * force, roll=3 * force, nod=12 * force, dip=-.043 * force)
    rotate(rig, "lombar", RIGHT, 13 * force)
    rotate(rig, "torax", RIGHT, 13 * force)
    update()
    for _, suffix, _ in SIDES:
        leg(rig, suffix, rest(rig, "pe" + suffix))
    if contact:
        wrist = rest(rig, "mao.R").lerp(Vector((-.16, -.74, 1.37)), force)
        arm(rig, ".R", wrist, .27 * force)


def sit(rig, t):
    s = interval(t, .07, .85)
    anchor_pelvis(rig, (.023 * s, .205 * s, -.52 * s))
    rotate(rig, "lombar", RIGHT, -19 * s + 27 * sin(pi * s))
    rotate(rig, "cabeca", RIGHT, 8 * s - 16 * sin(pi * s))
    update()
    for side, suffix, _ in SIDES:
        ankle = rest(rig, "pe" + suffix)
        ankle.y -= .26 * interval(t, .36 + side * .13, .69 + side * .13)
        leg(rig, suffix, ankle)


def turn(rig, t):
    pivot = interval(t, .23, .82)
    yaw = radians(-57 * pivot)
    rotation = Quaternion(UP, yaw)
    anchor_pelvis(rig, (.035 * sin(pi * pivot), 0, -.032 * sin(pi * pivot)))
    # A cabeça assume o ângulo novo quando os pés ainda estão no lugar.
    bone = rig.pose.bones["pelvis"]
    ref = bone.bone.matrix_local.to_quaternion()
    bone.rotation_quaternion = ref.inverted() @ rotation @ ref
    rotate(rig, "cabeca", BONE_AXIS, -26 * envelope(t, .22, .28, .80))
    update()
    for s, suffix, _ in SIDES:
        a, b = (.15, .48) if s > 0 else (.51, .86)
        step = interval(t, a, b)
        ankle = Quaternion(UP, radians(-57) * step) @ rest(rig, "pe" + suffix)
        ankle.z += .048 * sin(pi * step) ** 2
        leg(rig, suffix, ankle, radians(-57) * step)


def watch_quad(rig, t, anticipation=0):
    lower(rig, 1)
    tilt = 3 * sin(pi * 2 * t - .64) + 5 * anticipation
    rotate(rig, "pescoco", RIGHT, tilt * .52)
    rotate(rig, "cabeca", RIGHT, tilt * .68)
    rotate(rig, "cabeca", UP, 2 * sin(pi * t * 2) - 3 * anticipation)


@dataclass
class Clip:
    name: str
    seconds: float
    state: str
    pose: object
    repeat: bool = False
    posture: str = "human"
    markers: tuple = ()
    stance: tuple = ()
    speed: tuple = (0.0, 0.0)
    direction: str = "forward"


def human_clip(name, seconds, state, pose, repeat=False, markers=()):
    return Clip(name, seconds, state, pose, repeat, markers=markers)


def moving(name, state, gait, repeat=True, markers=()):
    return Clip(name, gait.seconds, state, gait.pose, repeat, "quad" if gait.quad else "human",
                markers, tuple(gait.contacts()), (gait.start, gait.end), gait.direction)


def delayed_wave(rig, t):
    shoulder = interval(t, 0, .25) * (1 - interval(t, .72, 1))
    if shoulder <= .0001:
        return
    before = basis(rig)
    human(rig, roll=3 * shoulder)
    farewell(rig, 16 * envelope(t, .39, .57, .71) * sin((t - .25) * 16))
    after = basis(rig)
    blend(rig, before, after, shoulder)


def catalogue():
    steps = (.0, .5)
    four = (.02, .57, .32, .83)
    mod = Gait(1.0, .99, .99, four, .62, .125, True)
    fast = Gait(.5, 2.20, 2.20, (.04, .53, .31, .82), .56, .165, True)
    launch = Gait(.83, 0, 2.1, (.10, .52, .28, .57), .60, .142, True)
    decel = Gait(1.0, 2.15, 0, (.04, .48, .23, .60), .65, .084, True, cycle=2)
    stop = Gait(.47, 2.2, 0, (.19, .65, .37, .53), .73, .076, True)
    all_down = tuple((0., 1., name, False) for name in
                     ("foot_left", "foot_right", "hand_left", "hand_right"))
    return (
        human_clip("08 - Respiração discreta", 4.6, "idle",
                   lambda r, t: human(r, breath=sin(2*pi*t) + .17*sin(4*pi*t)), True),
        human_clip("09 - Observação prolongada", 8.0, "watch",
                   lambda r, t: human(r, breath=.19*sin(2*pi*t), yaw=8*interval(t,.02,.25),
                                      roll=2*interval(t,.63,.88))),
        human_clip("10 - Inclinação lenta", 3.5, "watch",
                   lambda r, t: human(r, roll=-24*envelope(t,.55,.77), yaw=7*interval(t,.28,.55))),
        human_clip("11 - A cabeça gira primeiro", 3.4, "watch",
                   lambda r, t: human(r, yaw=-36*interval(t,.02,.32)+28*interval(t,.43,.82),
                                      torso=-27*interval(t,.43,.87))),
        human_clip("12 - Acompanhamento visual", 5.2, "watch",
                   lambda r, t: human(r, yaw=mix(-27,29,t), roll=-5*sin(pi*t))),
        moving("13 - Caminhada lenta",  "walk", Gait(1.7, .43, .43, steps, .67, .060)),
        moving("14 - Caminhada acelerada", "walk", Gait(1.1, 1.12, 1.12, steps, .60, .093)),
        moving("15 - Caminhada lateral", "walk", Gait(1.4, .29, .29, steps, .69, .058, direction="side")),
        human_clip("16 - Rotação corporal", 2.3, "walk", turn,
                   markers=((.48, "foot_left"), (.86, "foot_right"))),
        human_clip("17 - Espiada pelo vão", 3.8, "peek",
                   lambda r, t: peek(r, envelope(t,.37,.64))),
        moving("18 - Recuo humanoide", "retreat", Gait(1.5, .42, .42, steps, .72, .056, direction="back")),
        moving("19 - Aproximação cautelosa", "walk", Gait(2.0, .31, .31, (.14,.62), .76, .072)),
        human_clip("20 - Sentar", 2.8, "idle", sit, markers=((.69,"impact"),)),
        human_clip("21 - Levantar", 2.1, "idle", lambda r,t: sit(r,1-t)),
        Clip("22 - Ruptura lenta", 4.7, "rupture", lower, posture="fold",
             markers=((.04,"rupture_start"),(.75,"impact")),
             stance=((.755,1,"hand_left",True),(.7,1,"hand_right",True),
                     (.82,1,"foot_left",True),(.99,1,"foot_right",True),
                     (0,.63,"foot_left",False),(0,.77,"foot_right",False))),
        Clip("23 - Ruptura rápida", 1.15, "rupture", lower, posture="fold",
             markers=((.04,"rupture_start"),(.75,"impact")),
             stance=((.755,1,"hand_left",True),(.7,1,"hand_right",True),
                     (.82,1,"foot_left",True),(.99,1,"foot_right",True),
                     (0,.63,"foot_left",False),(0,.77,"foot_right",False))),
        Clip("24 - Ruptura interrompida", 2.6, "rupture", lambda r,t: lower(r,t,aborted=True),
             posture="threat", markers=((.08,"rupture_start"),), stance=all_down[:2]),
        Clip("25 - Preparação quadrúpede", 1.25, "quadruped_idle",
             lambda r,t: watch_quad(r,t,interval(t,.12,.96)), posture="quad", stance=all_down),
        moving("26 - Arrancada", "quadruped_sprint", launch, False, ((.12,"sprint_start"),)),
        moving("27 - Corrida moderada", "quadruped_sprint", mod),
        moving("28 - Corrida intensa", "quadruped_sprint", fast),
        moving("29 - Mudança de direção", "quadruped_sprint",
               Gait(.80, 1.42, 1.42, four, .60, .105, True, bank=.42)),
        moving("30 - Desacelerar", "stop", decel, False),
        moving("31 - Parada brusca", "stop", stop, False, ((.32,"impact"),)),
        Clip("32 - Retorno humanoide", 3.1, "stop", lambda r,t: lower(r,t,reversing=True),
             posture="rise", stance=((0,.245,"hand_left",False),(0,.30,"hand_right",False),
                                     (.37,1,"foot_left",True),(.23,1,"foot_right",True),
                                     (0,.18,"foot_left",False),(0,.01,"foot_right",False))),
        human_clip("33 - Despedida lenta", 5.4, "farewell", delayed_wave),
        human_clip("34 - Avanço do rosto", 1.6, "contact", lean),
        human_clip("35 - Gesto de contato", 2.4, "contact", lambda r,t: lean(r,t,contact=True),
                   markers=((.34,"contact"),)),
        human_clip("36 - Inclinação extrema", 4.2, "watch",
                   lambda r,t: human(r, roll=-44*envelope(t,.59,.78), yaw=8*interval(t,.34,.60))),
        Clip("37 - Vigília quadrúpede", 4.0, "quadruped_idle", watch_quad, True, "quad", stance=all_down),
        moving("38 - Retirada quadrúpede", "retreat",
               Gait(1.2,.70,.70, four,.7,.091,True,direction="back")),
        Clip("39 - Imóvel após ameaça", 5.0, "quadruped_idle", lambda r,t: lower(r,1),
             True, "quad", stance=all_down),
    )


def basis(rig):
    return [(b.location.copy(), b.rotation_quaternion.copy()) for b in rig.pose.bones]


def apply(rig, transforms):
    for bone, (position, rotation) in zip(rig.pose.bones, transforms):
        bone.location, bone.rotation_quaternion = position, rotation
    update()


def blend(rig, a, b, weight):
    apply(rig, [(p.lerp(q, weight), r.slerp(s, weight)) for (p,r), (q,s) in zip(a,b)])


def peek(rig, amount):
    before = basis(rig)
    peeking(rig)
    anchor_pelvis(rig, (.185, -.032, -.068))
    blend(rig, before, basis(rig), amount)
    for side, suffix, _ in SIDES:
        ankle = rest(rig, "pe" + suffix)
        ankle.x += .076 * amount * (1 if side > 0 else 0.3)
        leg(rig, suffix, ankle)


def add_action(rig, clip):
    frames = round(clip.seconds * FPS)
    samples = []
    rig.animation_data.action = None
    for frame in range(frames + 1):
        reset(rig)
        clip.pose(rig, frame / frames)
        update()
        values = basis(rig)
        if samples:
            for (_, quaternion), (_, previous) in zip(values, samples[-1]):
                quaternion.make_compatible(previous)
        samples.append(values)
    action = bpy.data.actions.new(clip.name)
    action.use_fake_user = True
    slot = action.slots.new(rig.id_type, rig.name)
    strip = action.layers.new("Movimento").strips.new(type="KEYFRAME")
    curves = strip.channelbag(slot, ensure=True).fcurves
    for index, bone in enumerate(rig.pose.bones):
        for channel, part in (("location", 0), ("rotation_quaternion", 1)):
            values = [pose[index][part] for pose in samples]
            if max(abs(v - start) for value in values for v, start in zip(value, values[0])) < .00005:
                frameset = (0, frames)
            else:
                frameset = range(frames+1)
            for axis in range(len(values[0])):
                curve = curves.new(bone.path_from_id(channel), index=axis)
                curve.keyframe_points.add(len(frameset))
                curve.keyframe_points.foreach_set("co", [
                    coordinate for frame in frameset for coordinate in (frame, values[frame][axis])
                ])
                for key in curve.keyframe_points:
                    key.interpolation = "LINEAR"
                curve.update()
    track = rig.animation_data.nla_tracks.new()
    track.name = clip.name
    strip = track.strips.new(clip.name, 0, action)
    strip.action_slot = slot
    track.mute = True
    print(f"AÇÃO {clip.name} / {frames} intervalos", flush=True)


def glb_header(path):
    with path.open("rb") as archive:
        header = archive.read(20)
        size = struct.unpack_from("<I", header, 12)[0]
        return json.loads(archive.read(size))


def metadata(clips, glb):
    measured = {}
    for action in glb["animations"]:
        inputs = [glb["accessors"][sampler["input"]] for sampler in action["samplers"]]
        if int(action["name"][:2]) >= 8:
            assert all(sampler["interpolation"] == "LINEAR"
                       or (sampler["interpolation"] == "STEP"
                           and glb["accessors"][sampler["input"]]["count"] == 2)
                       for sampler in action["samplers"]), action["name"]
        measured[action["name"]] = max(curve["max"][0] for curve in inputs)
    assert len(measured) == 7 + len(clips)
    out = []
    for clip in clips:
        duration = measured[clip.name]
        assert abs(duration - round(clip.seconds * FPS)/FPS) < .0002, (clip.name, duration)
        out.append({
            "label": clip.name,
            "fps": FPS,
            "duration": round(duration, 6),
            "state": clip.state,
            "posture": clip.posture,
            "repeat": clip.repeat,
            "speed": clip.speed,
            "direction": clip.direction,
            "markers": [{"at": round(t,6), "kind": name} for t,name in sorted(clip.markers)],
            "stance": [{"start": round(a,6), "end": round(b,6), "limb": name, "plant": plant}
                       for a,b,name,plant in sorted(clip.stance)],
        })
    return out


def main():
    source, output = (Path(s).resolve() for s in sys.argv[sys.argv.index("--")+1:])
    output.mkdir(parents=True, exist_ok=True)
    bpy.ops.wm.open_mainfile(filepath=str(source), load_ui=True)
    rig, = (obj for obj in bpy.context.scene.objects if obj.type == "ARMATURE")
    scene = bpy.context.scene
    assert len(rig.pose.bones) == 52 and len(rig.animation_data.nla_tracks) == 7
    bpy.context.preferences.filepaths.save_version = 0
    scene.render.fps = FPS
    scene.frame_start, scene.frame_end = 0, 240
    clips = catalogue()
    for clip in clips:
        add_action(rig, clip)
    reset(rig)
    scene.frame_set(0)
    bpy.ops.wm.save_as_mainfile(filepath=str(output / "presenca.blend"), compress=True)
    path = output / "presenca.glb"
    bpy.ops.export_scene.gltf(
        filepath=str(path), export_format="GLB", export_texcoords=True, export_normals=True,
        export_tangents=True, export_materials="EXPORT", export_skins=True, export_animations=True,
        export_animation_mode="ACTIONS", export_all_influences=False, export_apply=False,
        export_extras=True, export_anim_slide_to_zero=False,
    )
    (output / "movimentos.json").write_text(
        json.dumps(metadata(clips, glb_header(path)), ensure_ascii=False, indent=2) + "\n",
        encoding="utf-8",
    )


if __name__ == "__main__":
    main()
