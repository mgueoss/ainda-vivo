"""Codifica em MP4 a sequência PNG capturada pelo Bevy com --lab-record-motion."""

from pathlib import Path
import sys

import bpy


def main():
    directory, output = (Path(value).resolve() for value in sys.argv[sys.argv.index("--") + 1:])
    frames = sorted(directory.glob("*.png"))
    if len(frames) < 2 or any(frame.stem != f"{index:04d}" for index, frame in enumerate(frames)):
        raise ValueError("As capturas precisam formar uma sequência contínua a partir de 0000.png")
    scene = bpy.context.scene
    image = bpy.data.images.load(str(frames[0]))
    width, height = image.size
    bpy.data.images.remove(image)
    editor = scene.sequence_editor_create()
    strip = editor.strips.new_image("Captura Bevy", str(frames[0]), channel=1, frame_start=1)
    for frame in frames[1:]:
        strip.elements.append(frame.name)
    scene.frame_start, scene.frame_end = 1, len(frames)
    scene.render.resolution_x, scene.render.resolution_y = width, height
    scene.render.resolution_percentage = 100
    scene.render.fps = 30
    scene.render.use_sequencer = True
    scene.view_settings.view_transform = "Standard"
    scene.render.image_settings.file_format = "FFMPEG"
    scene.render.ffmpeg.format = "MPEG4"
    scene.render.ffmpeg.codec = "H264"
    scene.render.ffmpeg.constant_rate_factor = "HIGH"
    scene.render.filepath = str(output)
    bpy.ops.render.render(animation=True)
    print(f"MP4: {len(frames)} quadros a 30 fps em {width} × {height}")


if __name__ == "__main__":
    main()
