"""Malha orgânica em metros. Frente em -Y e chão em Z=0 no Blender."""

from collections import defaultdict
from math import ceil, cos, exp, pi, sin

import bpy
from mathutils import Vector
from mathutils.kdtree import KDTree


class Sculpt:
    def __init__(self):
        self.vertices = []
        self.faces = []
        self.weights = []
        self.bones = {}

    def bone(self, name, head, tail, parent=None):
        self.bones[name] = Vector(head), Vector(tail), parent

    def loft(self, rings, direction, shape=None, sides=32):
        # Um anel é centro, dois raios transversais e pesos de referência.
        # O voxel remesh une as interseções. Estes anéis nunca ficam no GLB.
        start = len(self.vertices)
        along = Vector(direction).normalized()
        across = (Vector((1, 0, 0)) - along * along.x).normalized()
        depth = along.cross(across)
        rings = smooth_sections(rings)
        for center, rx, ry, weights in rings:
            position = Vector(center)
            for sector in range(sides):
                angle = 2 * pi * sector / sides
                point = position + across * (rx * cos(angle)) + depth * (ry * sin(angle))
                if shape:
                    shape(point, angle)
                self.vertices.append(point)
                self.weights.append(weights)
        for index in range(len(rings) - 1):
            base = start + index * sides
            for s in range(sides):
                n = (s + 1) % sides
                self.faces.append((base + s, base + n, base + sides + n, base + sides + s))
        self.faces.append(tuple(start + s for s in reversed(range(sides))))
        end = start + (len(rings) - 1) * sides
        self.faces.append(tuple(end + s for s in range(sides)))

    def vertical(self, rings, shape=None, sides=40):
        self.loft(
            [((x, y, z), rx, ry, weights) for z, x, y, rx, ry, weights in rings],
            (0, 0, 1), shape, sides,
        )

    def limb(self, points, sections, shape=None, sides=28):
        rings = []
        for index, t, rx, ry, weights, shift in sections:
            center = Vector(points[index]).lerp(Vector(points[index + 1]), t) + Vector(shift)
            rings.append((center, rx, ry, weights))
        self.loft(rings, Vector(points[-1]) - Vector(points[0]), shape, sides)

    def build(self):
        mesh = bpy.data.meshes.new("Superfície contínua • Presença")
        mesh.from_pydata(self.vertices, [], self.faces)
        body = bpy.data.objects.new("Presença · corpo", mesh)
        bpy.context.collection.objects.link(body)
        bpy.context.view_layer.objects.active = body
        body.select_set(True)
        mesh.remesh_voxel_size = 0.0046
        bpy.ops.object.voxel_remesh()
        smooth = body.modifiers.new("União das transições", "SMOOTH")
        smooth.factor = 0.75
        smooth.iterations = 6
        bpy.ops.object.modifier_apply(modifier=smooth.name)
        decimate = body.modifiers.new("Topologia para tempo real", "DECIMATE")
        body.data.calc_loop_triangles()
        decimate.ratio = min(1, 18800 / len(body.data.loop_triangles))
        decimate.use_collapse_triangulate = True
        bpy.ops.object.modifier_apply(modifier=decimate.name)
        relax = body.modifiers.new("Relaxa o contorno", "SMOOTH")
        relax.factor = .48
        relax.iterations = 2
        bpy.ops.object.modifier_apply(modifier=relax.name)
        for face in body.data.polygons:
            face.use_smooth = True
        self.bind(body)
        return body

    def bind(self, body):
        tree = KDTree(len(self.vertices))
        for index, point in enumerate(self.vertices):
            tree.insert(point, index)
        tree.balance()
        groups = {bone: body.vertex_groups.new(name=bone) for bone in self.bones}
        weights = []
        for vertex in body.data.vertices:
            result = defaultdict(float)
            neighbours = tree.find_n(vertex.co, 7)
            nearest = neighbours[0][2]
            for _, index, distance in neighbours:
                if distance > nearest + 0.022:
                    break
                contribution = 1 / (0.003 + distance) ** 3
                for bone, weight in self.weights[index].items():
                    result[bone] += contribution * weight
            weights.append(normalize(result))
        # Relaxa a transferência na superfície real, sem misturar dedos vizinhos
        # através do ar. As bordas mais curtas recebem correção mais forte.
        neighbours = [[] for _ in weights]
        for a, b in (edge.vertices for edge in body.data.edges):
            distance = (body.data.vertices[a].co - body.data.vertices[b].co).length
            factor = 1 / max(distance, .0025)
            neighbours[a].append((b, factor))
            neighbours[b].append((a, factor))
        for _ in range(11):
            adjusted = []
            for index, result in enumerate(weights):
                vicinity = neighbours[index]
                blend_factor = .70
                smooth = defaultdict(float, {bone: weight * (1 - blend_factor) for bone, weight in result.items()})
                divisor = sum(amount for _, amount in vicinity)
                for neighbour, factor in vicinity:
                    for bone, weight in weights[neighbour].items():
                        smooth[bone] += blend_factor * weight * factor / divisor
                adjusted.append(normalize(smooth, 7))
            weights = adjusted
        for index, result in enumerate(weights):
            for bone, weight in normalize(result).items():
                groups[bone].add([index], weight, "REPLACE")


def normalize(weights, limit=4):
    strongest = sorted(weights.items(), key=lambda pair: -pair[1])[:limit]
    total = sum(weight for _, weight in strongest)
    return {bone: weight / total for bone, weight in strongest}


def smooth_sections(rings):
    """Interpola os perfis sem degraus entre juntas ou mudanças de diâmetro."""
    values = [(*center, rx, ry) for center, rx, ry, _ in rings]
    length = [max((Vector(b[:3]) - Vector(a[:3])).length, .001) for a, b in zip(values, values[1:])]
    slopes = [[(b - a) / d for a, b in zip(current, following)]
              for current, following, d in zip(values, values[1:], length)]
    slopes = [slopes[0]] + [[2 * a * b / (a + b) if a * b > 0 else 0 for a, b in zip(first, second)]
                           for first, second in zip(slopes, slopes[1:])] + [slopes[-1]]
    result = []
    regions = [ring[3] for ring in rings]
    for i, distance in enumerate(length):
        count = max(1, ceil(distance / .010))
        for part in range(count):
            u = part / count
            u2 = u * u
            u3 = u2 * u
            shape = [(2 * u3 - 3 * u2 + 1) * a + (u3 - 2 * u2 + u) * d0 * distance
                     + (-2 * u3 + 3 * u2) * b + (u3 - u2) * d1 * distance
                     for a, b, d0, d1 in zip(values[i], values[i + 1], slopes[i], slopes[i + 1])]
            weights = {bone: weight * (1 - u) for bone, weight in regions[i].items()}
            for bone, weight in regions[i + 1].items():
                weights[bone] = weights.get(bone, 0) + weight * u
            result.append((shape[:3], shape[3], shape[4], weights))
    return result + rings[-1:]


def g(value, center, width):
    return exp(-((value - center) / width) ** 2)


def single(bone):
    return {bone: 1.0}


def blend(bone, other, amount):
    return {bone: 1 - amount, other: amount}


def anatomy():
    s = Sculpt()
    s.bone("pelvis", (0, .047, 1.065), (-.006, .051, 1.228))
    s.bone("lombar", (-.006, .051, 1.228), (.010, .060, 1.433), "pelvis")
    s.bone("torax", (.010, .060, 1.433), (.012, .026, 1.657), "lombar")
    s.bone("peito", (.012, .026, 1.657), (-.007, -.012, 1.810), "torax")
    s.bone("pescoco", (-.007, -.012, 1.810), (-.024, -.071, 1.932), "peito")
    s.bone("cabeca", (-.024, -.071, 1.932), (-.019, -.085, 2.169), "pescoco")

    p, l, t, c, n, h = map(single, ("pelvis", "lombar", "torax", "peito", "pescoco", "cabeca"))

    def torso(point, _):
        x, y, z = point
        if y > .045:
            # Crista vertebral, escápulas desiguais e tendões entre pescoço e escápula.
            ridge = sum(g(z, level, .023) for level in (1.27, 1.36, 1.45, 1.54, 1.62, 1.70))
            point.y += .014 * g(x, .004, .036) * ridge
            point.y += .020 * g(abs(x), .116, .052) * g(z, 1.611 + x * .19, .12)
            point.y -= .015 * g(abs(x), .105, .040) * g(z, 1.41, .10)
        elif y < -.005:
            ribs = sum(g(z, level + abs(x) * .22, .024) for level in (1.44, 1.51, 1.58))
            point.y -= .013 * ribs * g(abs(x), .106, .059)
            point.y -= .010 * g(x, -.009, .024) * g(z, 1.63, .11)
            point.y += .012 * g(abs(x), .104, .035) * g(z, 1.36, .10)
        point.z += x * .13 * g(z, 1.75, .10)

    s.vertical([
        (1.013, 0, .035, .076, .053, p), (1.06, 0, .041, .120, .083, p),
        (1.126, .002, .045, .157, .098, p), (1.204, -.008, .049, .150, .091, blend("pelvis", "lombar", .32)),
        (1.263, -.011, .053, .128, .079, l), (1.337, -.006, .061, .120, .073, l),
        (1.403, .002, .057, .136, .080, blend("lombar", "torax", .44)),
        (1.464, .006, .049, .151, .087, t), (1.521, .012, .046, .158, .096, t),
        (1.570, .014, .045, .162, .101, t), (1.625, .012, .033, .185, .105, blend("torax", "peito", .38)),
        (1.684, .008, .025, .206, .096, c), (1.736, .010, .022, .211, .084, c),
        (1.772, .005, .009, .165, .077, c), (1.815, -.009, -.005, .083, .062, blend("peito", "pescoco", .23)),
        (1.852, -.015, -.018, .054, .050, n),
    ], torso, 56)

    def neck(point, _):
        x, y, z = point
        point.y -= .007 * g(abs(x + .014), .028, .009) * g(z, 1.875, .054) if y < -.023 else 0

    s.vertical([
        (1.771, -.006, .004, .066, .060, c), (1.811, -.010, -.008, .062, .055, blend("peito", "pescoco", .50)),
        (1.842, -.014, -.022, .053, .051, n), (1.867, -.019, -.036, .045, .045, n),
        (1.895, -.024, -.045, .045, .044, n), (1.922, -.026, -.049, .049, .046, blend("pescoco", "cabeca", .42)),
        (1.953, -.027, -.058, .052, .044, h),
    ], neck)

    def face(point, angle):
        x, _, z = point
        x += .022
        weight = max(0, -sin(angle)) ** 2
        z += x * .15
        if weight == 0:
            return
        for side, height, depth in [(-1, 2.074, .023), (1, 2.067, .018)]:
            center = side * .035
            socket = g(x, center, .023) * g(z, height, .014)
            brow = g(x, center, .028) * g(z, height + .018 + x * .12, .008)
            lid = g(x, center, .015) * g(z, height - .004, .0032)
            cheek = g(x, center * 1.42, .021) * g(z, 2.041 + side * .005, .019)
            hollow = g(x, center * 1.33, .023) * g(z, 2.004, .018)
            point.y += weight * (depth * socket - .011 * brow - .010 * lid - .016 * cheek + .010 * hollow)
        bridge = .020 * g(x, .005, .010) * g(z, 2.057, .029)
        tip = .022 * g(x, .008, .015) * g(z, 2.029, .011)
        nostrils = .009 * g(abs(x - .006), .012, .006) * g(z, 2.018, .004)
        line = z - (1.994 - x * .27 + 6 * x * x)
        lips = .006 * g(x, -.003, .034) * (g(line, -.006, .006) + .66 * g(line, .007, .005))
        slit = .008 * g(x, -.004, .033) * g(line, 0, .0037)
        chin = .012 * g(x, -.014, .027) * g(z, 1.956, .013)
        point.y += weight * (-bridge - tip + nostrils - lips + slit - chin)

    sections = [
        (1.916, -.026, -.082, .016, .025), (1.932, -.029, -.091, .032, .045),
        (1.944, -.032, -.094, .046, .060),
        (1.962, -.031, -.094, .059, .072), (1.989, -.029, -.086, .070, .080),
        (2.015, -.023, -.081, .077, .085), (2.044, -.020, -.077, .085, .092),
        (2.081, -.017, -.075, .087, .095), (2.103, -.016, -.069, .083, .096),
        (2.129, -.018, -.063, .078, .089), (2.154, -.019, -.060, .067, .076),
        (2.178, -.020, -.058, .049, .059), (2.192, -.021, -.057, .025, .034),
        (2.197, -.020, -.057, .010, .014), (2.199, -.020, -.057, .003, .004),
    ]
    head = []
    for a, b in zip(sections, sections[1:]):
        count = max(1, round((b[0] - a[0]) / .006))
        for i in range(count):
            u = i / count
            head.append(tuple((1 - u) * v + u * w for v, w in zip(a, b)) + (h,))
    s.vertical(head + [sections[-1] + (h,)], face, 88)
    for side in (-1, 1):
        x = -.022 + side * .080
        z = 2.036 + (side * .009)
        s.vertical([
            (z - .036, x, -.050, .007, .010, h), (z - .027, x + side * .007, -.048, .012, .012, h),
            (z - .008, x + side * .008, -.045, .014, .016, h),
            (z + .013, x + side * .004, -.041, .014, .019, h),
            (z + .030, x - side * .001, -.047, .010, .013, h),
            (z + .034, x - side * .004, -.047, .004, .007, h),
        ], lambda point, _: ear(point, x, z, side), 22)
        arm(s, side)
        leg(s, side)
    return s


def ear(point, x, z, side):
    if side * (point.x - x) > .010:
        point.x -= side * .012 * g(point.y, -.051, .011) * g(point.z, z, .017)


def arm(s, side):
    suffix = ".L" if side > 0 else ".R"
    inner = (.016, .020, 1.757)
    shoulder = (side * .210 + .010, .024 + side * .008, 1.718 + side * .028)
    elbow = (side * .283 + .005, .059 - side * .014, 1.365 + side * .014)
    wrist = (side * .315, -.003 + side * .018, 1.014 - side * .019)
    knuckles = (side * .322, -.020 + side * .015, .890 - side * .021)
    clavicle, upper, lower, hand = (name + suffix for name in ("clavicula", "braco", "antebraco", "mao"))
    s.bone(clavicle, inner, shoulder, "peito")
    s.bone(upper, shoulder, elbow, clavicle)
    s.bone(lower, elbow, wrist, upper)
    s.bone(hand, wrist, knuckles, lower)
    zero = (0, 0, 0)
    s.limb([shoulder, elbow, wrist, knuckles], [
        (0, -.25, .018, .033, single("peito"), (-side * .111, -.003, -.042)),
        (0, -.16, .039, .065, blend("peito", clavicle, .45), (-side * .065, .001, -.015)),
        (0, -.065, .057, .069, blend(clavicle, upper, .45), (-side * .026, .004, -.001)),
        (0, .075, .069, .070, blend(clavicle, upper, .80), (-side * .008, .003, 0)),
        (0, .20, .066, .061, single(upper), zero),
        (0, .40, .057, .056, single(upper), (side * .004, -.010, 0)),
        (0, .71, .048, .044, single(upper), (side * .003, -.005, 0)),
        (0, .92, .042, .042, blend(upper, lower, .16), zero),
        (1, .02, .042, .050, blend(upper, lower, .52), (0, .007, 0)),
        (1, .22, .050, .045, single(lower), zero),
        (1, .43, .044, .039, single(lower), (0, -.007, 0)),
        (1, .67, .033, .030, single(lower), (side * .001, -.008, 0)),
        (1, .91, .029, .023, blend(lower, hand, .17), zero),
        (2, .03, .033, .026, blend(lower, hand, .63), zero),
        (2, .28, .041, .025, single(hand), zero),
        (2, .59, .045, .023, single(hand), (0, -.002, 0)),
        (2, .86, .041, .018, single(hand), zero),
        (2, .99, .029, .015, single(hand), zero),
    ], sides=36)

    for i, (name, dx, dy, length, radius) in enumerate([
        ("indicador", -.031, -.010, .134, .0125), ("medio", -.009, -.014, .153, .013),
        ("anelar", .014, -.012, .142, .012), ("minimo", .035, -.009, .112, .010),
    ]):
        base = Vector(knuckles) + Vector((side * dx, dy, .022 + .006 * abs(i - 1)))
        joint = base + Vector((side * dx * .36, -.007 + i * .005, -length * .43))
        next_joint = joint + Vector((side * dx * .13, .008 + i * .003, -length * .32))
        tip = next_joint + Vector((side * dx * .02, .012 + i * .002, -length * .25))
        finger(s, suffix, name, hand, [base, joint, next_joint, tip], radius)

    base = Vector(wrist).lerp(Vector(knuckles), .31) + Vector((-side * .027, -.017, -.005))
    joint = base + Vector((-side * .035, -.012, -.038))
    tip = joint + Vector((-side * .004, -.008, -.051))
    finger(s, suffix, "polegar", hand, [base, joint, tip.lerp(joint, .34), tip], .015)


def finger(s, suffix, label, hand, points, radius):
    a, b, c = [f"{label}.{i}{suffix}" for i in range(1, 4)]
    s.bone(a, points[0], points[1], hand)
    s.bone(b, points[1], points[2], a)
    s.bone(c, points[2], points[3], b)
    zero = (0, 0, 0)
    s.limb(points, [
        (0, -.24, radius * .76, radius * .79, single(hand), zero),
        (0, .12, radius * 1.06, radius, blend(hand, a, .56), zero),
        (0, .51, radius * .82, radius * .86, single(a), zero),
        (0, .87, radius * .95, radius * .89, blend(a, b, .23), zero),
        (1, .13, radius * .95, radius * .87, blend(a, b, .55), zero),
        (1, .49, radius * .74, radius * .78, single(b), zero),
        (1, .87, radius * .80, radius * .78, blend(b, c, .26), zero),
        (2, .25, radius * .75, radius * .66, single(c), zero),
        (2, .65, radius * .70, radius * .64, single(c), zero),
        (2, .90, radius * .54, radius * .49, single(c), zero),
        (2, 1, radius * .27, radius * .24, single(c), zero),
    ], sides=20)


def leg(s, side):
    suffix = ".L" if side > 0 else ".R"
    thigh, shin, foot, toes = (name + suffix for name in ("coxa", "canela", "pe", "dedos_pe"))
    hip = (side * .098, .044, 1.129)
    knee = (side * .131, -.004 + side * .008, .641 + side * .010)
    ankle = (side * .142, .045 + side * .029, .114)
    ball = (side * .152, -.105 + side * .029, .041)
    s.bone(thigh, hip, knee, "pelvis")
    s.bone(shin, knee, ankle, thigh)
    s.bone(foot, ankle, ball, shin)
    s.bone(toes, ball, (side * .163, -.181 + side * .029, .039), foot)
    zero = (0, 0, 0)
    s.limb([hip, knee, ankle], [
        (0, -.13, .063, .076, single("pelvis"), (-side * .032, .013, 0)),
        (0, .06, .086, .095, blend("pelvis", thigh, .63), (-side * .014, .006, 0)),
        (0, .24, .084, .082, single(thigh), (side * .004, -.010, 0)),
        (0, .44, .078, .071, single(thigh), (side * .007, -.009, 0)),
        (0, .73, .060, .059, single(thigh), zero),
        (0, .91, .048, .056, blend(thigh, shin, .22), (0, -.006, 0)),
        (1, .04, .052, .061, blend(thigh, shin, .61), (0, -.008, 0)),
        (1, .19, .054, .056, single(shin), (0, .024, 0)),
        (1, .35, .056, .063, single(shin), (0, .031, 0)),
        (1, .52, .047, .052, single(shin), (0, .021, 0)),
        (1, .73, .037, .035, single(shin), (0, .007, 0)),
        (1, .93, .032, .032, blend(shin, foot, .28), zero),
        (1, 1.07, .034, .033, single(foot), zero),
    ], sides=34)
    x, y, _ = ankle
    foot_start = len(s.vertices)
    s.vertical([
        (.001, x, y - .041, .033, .090, single(foot)),
        (.011, x + side * .008, y - .054, .052, .128, single(foot)),
        (.027, x + side * .008, y - .055, .055, .133, single(foot)),
        (.043, x + side * .004, y - .053, .053, .125, single(foot)),
        (.060, x + side * .001, y - .040, .049, .105, single(foot)),
        (.078, x, y - .013, .037, .068, single(foot)),
        (.102, x, y + .004, .032, .043, single(foot)),
        (.129, x, y + .003, .030, .034, blend(foot, shin, .34)),
    ], lambda point, _: foot_shape(point, side, x, y), 44)
    # Na malha final a porção frontal do antepé segue a articulação dos dedos.
    for index in range(foot_start, len(s.vertices)):
        point = s.vertices[index]
        amount = min(1, max(0, (-point.y + y - .105) / .063))
        if amount > .005:
            s.weights[index] = blend(foot, toes, amount)


def foot_shape(point, side, x, y):
    forward = max(0, min(1, (-point.y + y) / .17))
    taper = (point.x - x) * side
    point.y += forward ** 3 * (.040 * max(0, taper + .021) / .075)
    point.z += .013 * g(point.y - y, -.034, .043) * g(taper, -.036, .022) if point.z < .035 else 0
