"""Texturas PBR originais, pintadas em coordenadas do corpo e gravadas em UV."""

from math import radians
from pathlib import Path
import struct
import zlib

import bpy
import numpy as np


def gaussian(value, center, radius):
    return np.exp(-((value - center) / radius) ** 2)


def cloud(position):
    # Ondas em direções não coincidentes com a malha evitam padrão de grade e emendas UV.
    axes = np.array([[.36, -.41, .84], [-.67, .59, .45], [.74, .57, -.36],
                     [.22, -.91, -.35], [-.48, -.69, .55], [.89, -.20, .44]], dtype=np.float32)
    amplitude = np.array([.29, .21, .16, .13, .12, .09], dtype=np.float32)
    phase = np.array([.33, 1.71, -2.24, 1.34, -1.24, .67], dtype=np.float32)
    projected = position @ axes.T
    slow = np.sin(phase + projected * np.array([11., 15., 22., 31., 43., 59.])) @ amplitude
    medium = np.sin(phase * 2 + projected * np.array([60., 83., 109., 143., 167., 191.])) @ amplitude
    pore = np.sin(phase * 3 + projected * np.array([188., 251., 315., 409., 511., 628.])) @ amplitude
    return slow, medium, pore


def pigment(position, normal):
    x, y, z = position.T
    cheeks = np.abs(x + .022)
    slow, medium, pores = cloud(position)
    diffuse = (np.array([.224, .226, .222]) + slow[:, None] * np.array([.059, .050, .042])
               + medium[:, None] * .046 + pores[:, None] * .019)
    rough = .80 + .11 * slow - .04 * medium + .035 * pores
    # Pele deteriorada na testa, nariz, arcos zigomáticos e nós das mãos.
    frontal = np.clip(-normal[:, 1] * 1.65, 0, 1)
    cheeks_brow = frontal * (gaussian(z, 2.106 + (x + .02) * .11, .041) * .20
                             + gaussian(z, 2.041, .018) * gaussian(cheeks, .052, .023) * .77
                             + gaussian(z, 2.038, .043) * gaussian(x, -.013, .016) * .58)
    diffuse += cheeks_brow[:, None] * np.array([.135, .119, .105])
    sockets = frontal * gaussian(z + x * .15, 2.071, .011) * gaussian(cheeks, .035, .019)
    diffuse -= sockets[:, None] * np.array([.13, .123, .115])
    eyelid = frontal * gaussian(z + x * .15, 2.067, .003) * gaussian(cheeks, .035, .012)
    diffuse += eyelid[:, None] * np.array([.064, .051, .040])
    slit = frontal * gaussian(z, 1.994 - (x + .022) * .42 + 6 * (x + .022) ** 2, .0035) * gaussian(cheeks, .004, .030)
    diffuse -= slit[:, None] * np.array([.12, .102, .091])
    rough -= .12 * slit
    abrasion = gaussian(cheeks + .014 * np.sin(60 * z), .050, .008) * gaussian(z, 2.007, .038) * frontal
    diffuse += abrasion[:, None] * np.array([.039, .023, .018])
    knuckles = gaussian(np.abs(x), .325, .065) * (
        gaussian(z, .891, .014) + .60 * gaussian(z, .831, .012)) * (.6 + .4 * frontal)
    diffuse += knuckles[:, None] * np.array([.073, .061, .050])
    # Os pés e o vestígio da veste têm pouca reflexão, com fibras mais escuras
    # no centro. A faixa curta do colar é gasta, sem branco puro.
    cloth = np.clip((1.841 - z) / .018, 0, 1)
    cloth *= 1 - (gaussian(np.abs(x), .32, .074) * np.clip((1.115 - z) / .14, 0, 1))
    diffuse *= 1 - .25 * cloth[:, None]
    thread = np.sin(780 * z + 1.8 * np.sin(133 * x)) * np.sin(548 * (x + .34 * y))
    diffuse += (cloth * thread * .010)[:, None]
    rough += cloth * .070
    fastening = gaussian(x, -.016 + .014 * np.sin(4.2 * z), .005) * gaussian(z, 1.625, .155) * frontal
    diffuse -= fastening[:, None] * np.array([.047, .044, .036])
    topstitch = gaussian(x, -.007 + .014 * np.sin(4.2 * z), .005) * gaussian(z, 1.625, .16) * frontal
    diffuse += (topstitch * (0.71 + .29 * np.sin(365 * z)))[:, None] * np.array([.033, .029, .022])
    collar = gaussian(z, 1.849 - (x + .018) * .26 + .04 * (y + .016), .017)
    insert = collar * frontal * gaussian(x, -.024, .022) * np.clip((-.039 - y) / .022, 0, 1)
    diffuse -= collar[:, None] * np.array([.018, .016, .013])
    diffuse += insert[:, None] * np.array([.162, .145, .116])

    grime = (.52 + .26 * slow) * np.clip((.31 - z) / .27, 0, 1)
    diffuse *= 1 - grime[:, None] * .32
    return np.clip(diffuse, .065, .47), np.clip(rough, .59, .96)


def bump(position):
    x, y, z = position.T
    _, medium, pore = cloud(position)
    fiber = np.sin(729 * z + 1.6 * np.sin(134 * x)) * np.sin(540 * (x + y * .29))
    return .00085 * medium + .00053 * pore + .00012 * fiber * np.clip((1.87 - z) / .15, 0, 1)


def rasterize(mesh, size):
    mesh.calc_loop_triangles()
    uv = mesh.uv_layers.active.data
    vertices = mesh.vertices
    position = np.zeros((size, size, 3), np.float32)
    normal = np.zeros_like(position)
    tangent = np.zeros_like(position)
    handed = np.ones((size, size), np.float32)
    covered = np.zeros((size, size), np.bool_)
    for tri in mesh.loop_triangles:
        a, b, c = np.array([uv[i].uv for i in tri.loops], dtype=np.float32) * size
        points = np.array([vertices[i].co for i in tri.vertices], dtype=np.float32)
        normals = np.array([vertices[i].normal for i in tri.vertices], dtype=np.float32)
        uv1, uv2 = b - a, c - a
        area = uv1[0] * uv2[1] - uv1[1] * uv2[0]
        if abs(area) < 1e-4:
            continue
        minimum = np.floor(np.min([a, b, c], axis=0)).astype(int).clip(0, size - 1)
        maximum = np.ceil(np.max([a, b, c], axis=0)).astype(int).clip(0, size - 1)
        xs, ys = np.meshgrid(np.arange(minimum[0], maximum[0] + 1),
                             np.arange(minimum[1], maximum[1] + 1))
        x, y = xs + .5 - a[0], ys + .5 - a[1]
        u, v = (uv2[1] * x - uv2[0] * y) / area, (uv1[0] * y - uv1[1] * x) / area
        valid = (u >= 0) & (v >= 0) & (u + v <= 1)
        if not valid.any():
            continue
        where = (ys[valid], xs[valid])
        weights = np.array([1 - (u + v)[valid], u[valid], v[valid]]).T.astype(np.float32)
        edge1, edge2 = points[1] - points[0], points[2] - points[0]
        along = (edge1 * uv2[1] - edge2 * uv1[1]) / area
        bitangent = (edge2 * uv1[0] - edge1 * uv2[0]) / area
        normal[where] = weights @ normals
        position[where] = weights @ points
        tangent[where] = along / max(np.linalg.norm(along), 1e-7)
        handed[where] = 1 if np.dot(np.cross(tri.normal, along), bitangent) >= 0 else -1
        covered[where] = True
    return covered, position[covered], normal[covered], tangent[covered], handed[covered]


def unit(array):
    return array / np.maximum(1e-7, np.linalg.norm(array, axis=1, keepdims=True))


def bleeding(coverage, maps):
    covered = coverage.copy()
    for _ in range(12):
        before = covered.copy()
        for axis, offset in [(0, 1), (1, 1), (0, -1), (1, -1)]:
            source = np.roll(before, offset, axis=axis)
            writing = source & ~covered
            if axis == 0:
                writing[0 if offset > 0 else -1, :] = False
            else:
                writing[:, 0 if offset > 0 else -1] = False
            for image in maps:
                image[writing] = np.roll(image, offset, axis=axis)[writing]
            covered |= writing


def png(path, rgb):
    image = np.uint8(np.round(rgb[::-1] * 255))
    h, w, _ = image.shape

    def chunk(key, data):
        return struct.pack(">I", len(data)) + key + data + struct.pack(">I", zlib.crc32(key + data))

    scanlines = b"".join(b"\x00" + row.tobytes() for row in image)
    path.write_bytes(b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", struct.pack(">IIBBBBB", w, h, 8, 2, 0, 0, 0))
                     + chunk(b"IDAT", zlib.compress(scanlines, 9)) + chunk(b"IEND", b""))


def material_for(body, directory: Path):
    bpy.ops.object.select_all(action="DESELECT")
    bpy.context.view_layer.objects.active = body
    body.select_set(True)
    bpy.ops.object.mode_set(mode="EDIT")
    bpy.ops.mesh.select_all(action="SELECT")
    bpy.ops.uv.smart_project(angle_limit=radians(66), island_margin=.018, area_weight=.24)
    bpy.ops.object.mode_set(mode="OBJECT")
    size = 1024
    coverage, positions, normals, tangents, sign = rasterize(body.data, size)
    print(f"UV_PIXELS {np.count_nonzero(coverage)}/{size * size}", flush=True)

    normals = unit(normals)
    tangent = unit(tangents - normals * np.sum(tangents * normals, axis=1)[:, None])
    bitangent = np.cross(normals, tangent) * sign[:, None]
    width = .001
    center = bump(positions)
    du = (bump(positions + tangent * width) - center) / width
    dv = (bump(positions + bitangent * width) - center) / width
    relief = .5 + .5 * unit(np.array([-du, -dv, np.ones(len(du))]).T)
    color, roughness = pigment(positions, normals)
    albedo = np.full((size, size, 3), [.18, .18, .18], np.float32)
    encoded_normal = np.full_like(albedo, [.5, .5, 1.0])
    packed = np.full_like(albedo, [1.0, .81, 0.0])
    albedo[coverage] = color
    encoded_normal[coverage] = relief
    packed[coverage, 1] = roughness
    bleeding(coverage, (albedo, encoded_normal, packed))
    directory.mkdir(parents=True, exist_ok=True)
    for name, data in (("cor", albedo), ("relevo", encoded_normal), ("rugosidade", packed)):
        if name == "rugosidade":
            data = data.reshape(512, 2, 512, 2, 3).mean(axis=(1, 3))
        png(directory / f"presenca-{name}.png", data)

    material = bpy.data.materials.new("Presença • cinza, tecido e pele")
    material.diffuse_color = (.071, .075, .074, 1)
    material.use_nodes = True
    nodes, links = material.node_tree.nodes, material.node_tree.links
    shader = nodes.get("Principled BSDF")
    shader.inputs["Metallic"].default_value = 0
    shader.inputs["Specular IOR Level"].default_value = .26

    def image_node(part, x, y, color_space):
        image = bpy.data.images.load(str(directory / f"presenca-{part}.png"))
        image.colorspace_settings.name = color_space
        image.pack()
        image.filepath = f"//texturas/presenca-{part}.png"
        node = nodes.new("ShaderNodeTexImage")
        node.image, node.location = image, (x, y)
        return node

    source = image_node("cor", -580, 220, "sRGB")
    rough = image_node("rugosidade", -580, -70, "Non-Color")
    normal = image_node("relevo", -580, -370, "Non-Color")
    channels = nodes.new("ShaderNodeSeparateColor")
    channels.location = -280, -50
    normal_map = nodes.new("ShaderNodeNormalMap")
    normal_map.location = -285, -300
    normal_map.inputs["Strength"].default_value = .42
    links.new(source.outputs["Color"], shader.inputs["Base Color"])
    links.new(rough.outputs["Color"], channels.inputs["Color"])
    links.new(channels.outputs["Green"], shader.inputs["Roughness"])
    links.new(channels.outputs["Blue"], shader.inputs["Metallic"])
    links.new(normal.outputs["Color"], normal_map.inputs["Color"])
    links.new(normal_map.outputs["Normal"], shader.inputs["Normal"])
    body.data.materials.append(material)
