import argparse
import json
import sys
from contextlib import contextmanager
from dataclasses import dataclass
from itertools import chain
from math import radians
from pathlib import Path
from tempfile import TemporaryDirectory
from typing import Any, Dict, Iterator, Mapping, Optional, Sequence, Tuple, cast
from zipfile import ZipFile

import bmesh  # type: ignore
import bpy  # type: ignore
from mathutils import Euler, Quaternion, Vector  # type: ignore


@dataclass
class Node:
    location: Vector
    rotation: Quaternion
    mesh_index: int


CHICKEN_WALKERS: Mapping[str, int] = {
    "cauldron": 0,
    "daishi": 0,
    "puma": 30,
    "supernova": 65,
}


DISCARD_LOC = {
    "rarm",
    "larm",
    "rhand",
    "lhand",
    # sometimes, this is called torso_polys, but these don't require locking down
    "torso",
    "rtoe01",
    "rtoe02",
    "rtoe03",
    "ltoe01",
    "ltoe02",
    "ltoe03",
    "rcalf",
    "lcalf",
}


class MaterialFactory:
    @classmethod
    @contextmanager
    def with_tempdir(
        cls,
        textures: Optional[Path],
        materials: Sequence[Mapping[str, Any]],
    ) -> Iterator["MaterialFactory"]:
        with TemporaryDirectory() as tempdir:
            yield cls(textures, materials, Path(tempdir))

    def __init__(
        self,
        textures: Optional[Path],
        materials: Sequence[Mapping[str, Any]],
        tempdir: Path,
    ):
        self.tempdir = Path(tempdir)
        self.textures = ZipFile(textures) if textures else None
        self.materials = materials
        self.cache: Dict[int, bpy.types.Material] = {}
        self.default_image: Optional[bpy.types.Image] = None

    @staticmethod
    def _create_material(name: str) -> Tuple[bpy.types.Material, bpy.types.Node]:
        mat = bpy.data.materials.new(name)
        mat.use_nodes = True
        bsdf = mat.node_tree.nodes["Principled BSDF"]
        return mat, bsdf

    def _default_image(self) -> bpy.types.Image:
        if self.default_image:
            return self.default_image

        self.default_image = bpy.data.images.new(name="default", width=2, height=2)
        pixels = [
            [1.0, 0.0, 0.5, 1.0],
            [0.5, 0.0, 1.0, 1.0],
            [0.5, 0.0, 1.0, 1.0],
            [1.0, 0.0, 0.5, 1.0],
        ]
        self.default_image.pixels = list(chain.from_iterable(pixels))
        self.default_image.update()
        self.default_image.pack()
        return self.default_image

    def __call__(self, texture_index: int) -> bpy.types.Material:
        # cache hit
        try:
            return self.cache[texture_index]
        except KeyError:
            pass

        material_info = self.materials[texture_index]

        try:
            texture_info = material_info["Textured"]
        except KeyError:
            # untextured material
            mat, bsdf = self._create_material(f"material_{texture_index}")
            color_info = material_info["Colored"]

            color = color_info["color"]
            bsdf.inputs[0].default_value = (
                color["r"] / 255.0,
                color["g"] / 255.0,
                color["b"] / 255.0,
                1,
            )
        else:
            # textured material
            texture_name = texture_info["texture"]
            mat, bsdf = self._create_material(texture_name)

            if self.textures:
                # load image from archive
                filename = f"{texture_name}.png"
                try:
                    self.textures.extract(filename, path=str(self.tempdir))
                except KeyError:
                    print("WARNING: did not find", texture_name)
                    image = self._default_image()
                else:
                    image = bpy.data.images.load(str(self.tempdir / filename))
                    # we extracted the image to a temp dir, ensure it is packed into the blend file
                    image.pack()
                    # update the filepath to be sensible once the temp dir is deleted
                    image.filepath_raw = f"//{texture_name}.png"
            else:
                # otherwise, use a default image
                image = self._default_image()

            tex = mat.node_tree.nodes.new("ShaderNodeTexImage")
            tex.image = image
            mat.node_tree.links.new(bsdf.inputs["Base Color"], tex.outputs["Color"])

        self.cache[texture_index] = mat
        return mat


def _process_mesh(
    mech_name: str,
    mesh_name: str,
    mesh_data: Mapping[str, Any],
    material_factory: MaterialFactory,
    fix_chicken_walkers: bool,
) -> bpy.types.Object:
    mesh = bpy.data.meshes.new(mesh_name)
    obj = bpy.data.objects.new(mesh_name, mesh)

    if fix_chicken_walkers and mesh_name in ("mesh_rcalf", "mesh_lcalf"):
        rot_x = CHICKEN_WALKERS.get(mech_name, 180.0)
    else:
        rot_x = 180.0

    # i don't remember why this is needed?
    obj.rotation_euler = (radians(rot_x), 0.0, 0.0)
    bpy.context.scene.collection.objects.link(obj)

    vertices = mesh_data["vertices"]
    # skipped: normals
    # Blender doesn't really deal with vertex normals, instead it's per face
    # normals = mesh_data["normals"]
    polygons = mesh_data["polygons"]

    textures = set()
    for poly in polygons:
        textures.add(poly["texture_index"])

    # faces have to be assigned a material index, not a material name
    tex_index_to_mat_index = {}
    for material_index, texture_index in enumerate(textures):
        mat = material_factory(texture_index)
        obj.data.materials.append(mat)
        tex_index_to_mat_index[texture_index] = material_index

    bm = bmesh.new(use_operators=True)
    for vert in vertices:
        bm.verts.new((vert["x"], vert["y"], vert["z"]))

    bm.verts.ensure_lookup_table()
    bm.verts.index_update()

    uv_layer = bm.loops.layers.uv.new()
    color_layer = bm.loops.layers.color.new("color")

    for poly in polygons:
        vertex_indices = poly["vertex_indices"]
        colors = poly["vertex_colors"]
        # skipped: normal_indices
        # normal_indices = poly["normal_indices"]
        uvs = poly["uv_coords"]
        texture_index = poly["texture_index"]

        if poly.get("triangle_strip", False):
            print("Triangle strip")
            for i in range(len(vertex_indices) - 3 + 1):
                vertex_window = vertex_indices[i : i + 3]
                try:
                    face = bm.faces.new(bm.verts[i] for i in vertex_window)
                except ValueError as e:
                    # some models contain duplicate faces. they are identical, so
                    # skipping them seems harmless
                    print(
                        "Mesh",
                        mesh_name,
                        "error:",
                        str(e),
                        "ptr:",
                        poly["vertices_ptr"],
                    )
                    continue
                face.smooth = True
                face.material_index = tex_index_to_mat_index[texture_index]

                if colors:
                    colors_window = colors[i : i + 3]
                    # because all our faces are created as loops, this should work
                    for index_in_mesh, loop, color in zip(
                        vertex_window, face.loops, colors_window
                    ):
                        assert loop.vert.index == index_in_mesh
                        loop[color_layer] = (
                            color["r"] / 255.0,
                            color["g"] / 255.0,
                            color["b"] / 255.0,
                            1.0,
                        )

                if uvs:
                    uvs_window = uvs[i : i + 3]
                    # because all our faces are created as loops, this should work
                    for index_in_mesh, loop, uv in zip(
                        vertex_window, face.loops, uvs_window
                    ):
                        assert loop.vert.index == index_in_mesh
                        loop[uv_layer].uv = (uv["u"], 1.0 - uv["v"])

        else:
            try:
                face = bm.faces.new(bm.verts[i] for i in vertex_indices)
            except ValueError as e:
                # some models contain duplicate faces. they are identical, so
                # skipping them seems harmless
                print("Mesh", mesh_name, "error:", str(e), "ptr:", poly["vertices_ptr"])
                continue

            face.smooth = True
            face.material_index = tex_index_to_mat_index[texture_index]

            if colors:
                # because all our faces are created as loops, this should work
                for index_in_mesh, loop, color in zip(
                    vertex_indices, face.loops, colors
                ):
                    assert loop.vert.index == index_in_mesh
                    loop[color_layer] = (
                        color["r"] / 255.0,
                        color["g"] / 255.0,
                        color["b"] / 255.0,
                        1.0,
                    )

            if uvs:
                # because all our faces are created as loops, this should work
                for index_in_mesh, loop, uv in zip(vertex_indices, face.loops, uvs):
                    assert loop.vert.index == index_in_mesh
                    loop[uv_layer].uv = (uv["u"], uv["v"])

    # since normals can't be loaded, and we've set smooth shading, calculate them
    bmesh.ops.recalc_face_normals(bm, faces=bm.faces)
    bm.to_mesh(mesh)
    bm.free()

    if mesh_name == "mesh_head":
        obj.hide_viewport = True
        obj.hide_render = True

    return obj


def _process_node(
    nodes: Sequence[Any],
    node_index: int,
    parent: Optional[bpy.types.EditBone],
    obj: bpy.types.Object,
    positions: Dict[str, Node],
) -> None:
    object3d = nodes[node_index]
    object3d = object3d["Object3d"]
    part_name = object3d["name"]
    mesh_index = object3d["mesh_index"]

    # create a bone to represent an object3d node, so we can animate it later
    # (if motions were requested)
    bone = cast(bpy.types.EditBone, obj.data.edit_bones.new(part_name))
    # our bones aren't connected... yet? i don't know how difficult this would be
    bone.use_connect = False  # default
    # do not deform meshes, only apply location and rotation
    bone.use_deform = False  # default

    if parent:
        bone.parent = parent

    bone.use_relative_parent = True

    transformation = object3d["transformation"]
    if transformation:
        trans = transformation["translation"]
        location = (trans["x"], trans["y"], trans["z"])
        rot = transformation["rotation"]
        rotation = Euler((rot["x"], rot["y"], rot["z"])).to_quaternion()
    else:
        location = (0.0, 0.0, 0.0)
        rotation = Quaternion()

    positions[part_name] = Node(
        location=Vector(location),
        # converting to quaternion plays nice with the motion data
        rotation=rotation,
        mesh_index=mesh_index,
    )

    # point straight up; this could probably be improved
    bone.tail = (0.0, 0.0, 1.0)

    children = object3d["children"]
    for child in children:
        _process_node(nodes, child, bone, obj, positions)


def _add_camera() -> None:
    camera = bpy.data.cameras.new("camera")
    camera.lens = 18

    obj = bpy.data.objects.new("camera", camera)
    obj.location = (0.0, 15.0, 10.0)
    obj.rotation_euler = (radians(80.0), 0.0, radians(180.0))

    copy_loc = obj.constraints.new(type="COPY_LOCATION")
    copy_loc.use_offset = True
    try:
        copy_loc.target = bpy.data.objects["mesh_hip_polys"]
    except KeyError:
        # blackhawk has no hip
        copy_loc.target = bpy.data.objects["mesh_torso_polys"]
    copy_loc.use_x = False
    copy_loc.use_y = True
    copy_loc.use_z = False

    bpy.context.scene.collection.objects.link(obj)
    bpy.context.scene.camera = obj


def _set_shading() -> None:
    for area in bpy.context.workspace.screens[0].areas:
        for space in area.spaces:
            if space.type == "VIEW_3D":
                space.shading.type = "MATERIAL"


def _add_lighting() -> None:
    light = bpy.data.lights.new("sun", type="SUN")
    light.distance = 80.0
    obj = bpy.data.objects.new("sun", light)
    obj.location = (0.0, 60.0, 40.0)
    bpy.context.scene.collection.objects.link(obj)

    world = bpy.data.worlds.new("world")
    world.use_nodes = True
    background = world.node_tree.nodes["Background"]
    background.inputs["Color"].default_value = (0.8, 0.8, 0.8, 1.0)
    background.inputs["Strength"].default_value = 0.1
    bpy.context.scene.world = world


def _setup_render() -> None:
    # set up render properties
    render = bpy.context.scene.render
    render.ffmpeg.format = "MPEG4"
    render.ffmpeg.codec = "H264"
    render.ffmpeg.constant_rate_factor = "MEDIUM"
    render.use_compositing = True
    render.resolution_x = 1080
    render.film_transparent = True


def _setup_compositing(use_alpha: bool) -> None:
    # set up background, otherwise it's black
    bpy.context.scene.use_nodes = True
    tree = bpy.context.scene.node_tree
    for node in tree.nodes:
        tree.nodes.remove(node)

    # create nodes
    background = tree.nodes.new(type="CompositorNodeRGB")
    background.outputs[0].default_value = (0.01, 0.00, 0.03, 1.00)
    background.location = (-250.0, 200.0)
    render_layers = tree.nodes.new(type="CompositorNodeRLayers")
    render_layers.mute = False
    render_layers.location = (-300.0, -200.0)
    alpha_over = tree.nodes.new(type="CompositorNodeAlphaOver")
    alpha_over.location = (0.0, 0.0)
    composite = tree.nodes.new(type="CompositorNodeComposite")
    composite.use_alpha = use_alpha
    composite.location = (250.0, 0.0)

    # link nodes
    tree.links.new(background.outputs["RGBA"], alpha_over.inputs[1])
    tree.links.new(render_layers.outputs["Image"], alpha_over.inputs[2])
    tree.links.new(render_layers.outputs["Alpha"], alpha_over.inputs["Fac"])
    tree.links.new(alpha_over.outputs["Image"], composite.inputs["Image"])
    tree.links.new(render_layers.outputs["Alpha"], composite.inputs["Alpha"])


def _create_motion(
    armature: bpy.types.Armature,
    motion_name: str,
    motion_data: Mapping[str, Any],
    positions: Dict[str, Node],
) -> None:
    bpy.ops.poselib.new()
    armature.pose_library.name = motion_name
    # stop the poselibs from being deleted if they are unused
    armature.pose_library.use_fake_user = True

    frame_count = motion_data["frame_count"]

    # reset the default pose based on the saved positions (since we discard some locs)
    for bone in armature.pose.bones:
        node = positions[bone.name]
        bone.location = node.location
        bone.rotation_quaternion = node.rotation

    def _insert_frame(frame_index: int) -> None:
        for part in motion_data["parts"]:
            part_name = part["name"]
            # since the bones "belong" to the pose, they have to be fetched for every frame
            bone = armature.pose.bones.get(part_name)
            if not bone:
                continue

            frames = part["frames"]
            frame_data = frames[frame_index]
            if part_name not in DISCARD_LOC:
                trans = frame_data["translation"]
                bone.location = (trans["x"], trans["y"], trans["z"])
            rot = frame_data["rotation"]
            bone.rotation_quaternion = (rot["x"], rot["y"], rot["z"], rot["w"])

        bpy.ops.poselib.pose_add(frame=frame_index + 1, name=f"{frame_index:02d}")

    for frame_index in range(frame_count):
        _insert_frame(frame_index)


def model_to_blend(
    model: Mapping[str, Any],
    name: str,
    material_factory: MaterialFactory,
    motions: Mapping[str, Mapping[str, Any]],
) -> bpy.types.Object:
    # empty scene, this prints some errors
    bpy.ops.wm.read_factory_settings(use_empty=True)

    nodes = model["nodes"]
    meshes = model["meshes"]

    # unpack the root object, which is always uninteresting (mech_foo.flt)
    root = nodes[0]["Object3d"]
    children = root["children"]
    assert len(children) == 1, len(children)

    # create a new armature
    arma = cast(bpy.types.Armature, bpy.data.armatures.new(name))
    armature = cast(bpy.types.Object, bpy.data.objects.new(name, arma))
    bpy.context.collection.objects.link(armature)

    # to create the bones, must be in edit mode
    bpy.context.view_layer.objects.active = armature
    bpy.ops.object.mode_set(mode="EDIT", toggle=False)
    # the positions can only be applied in pose mode, so save them
    positions: Dict[str, Node] = {}
    _process_node(nodes, children[0], None, armature, positions)

    # creating the meshes is easiest in object mode
    bpy.ops.object.mode_set(mode="OBJECT", toggle=False)
    for bone in armature.data.bones:
        node = positions[bone.name]
        if node.mesh_index < 0:
            continue

        obj = _process_mesh(
            name,
            f"mesh_{bone.name}",
            meshes[node.mesh_index],
            material_factory,
            fix_chicken_walkers=not motions,
        )

        # parent the obj to the correct bone in the armature
        obj.parent = armature
        obj.parent_bone = bone.name
        obj.parent_type = "BONE"
        # preserve translation
        obj.matrix_parent_inverse = bone.matrix_local.inverted()

    # this is changed during mesh creation to set smooth shading
    bpy.context.view_layer.objects.active = armature

    if motions:
        # if there are motions, apply them first to new poses/poselibs
        # toggling the mode somehow helps keep things clean???
        for motion_name, motion_data in motions.items():
            bpy.ops.object.mode_set(mode="POSE", toggle=False)
            _create_motion(armature, motion_name, motion_data, positions)
            bpy.ops.object.mode_set(mode="OBJECT", toggle=False)

        # create animation data to later set the action
        anim_data = armature.animation_data_create()
        scene = bpy.context.scene
        for motion_name in motions.keys():
            # set action
            action = bpy.data.actions[motion_name]
            # armature.animation_data.action = action

            # create an NLA track, otherwise action isn't played when rendering anim
            track = anim_data.nla_tracks.new()
            track.name = action.name
            strip = track.strips.new(action.name, 1, action)
    else:
        # create the default pose based on the saved positions
        bpy.ops.object.mode_set(mode="POSE", toggle=False)
        for bone in armature.pose.bones:
            node = positions[bone.name]
            bone.location = node.location
            bone.rotation_quaternion = node.rotation

        bpy.ops.poselib.new()
        armature.pose_library.name = "default"
        armature.pose_library.use_fake_user = True
        bpy.ops.poselib.pose_add(frame=1, name="default")

    # finally, set up the scene some more
    bpy.ops.object.mode_set(mode="OBJECT", toggle=False)
    if material_factory.textures:
        _set_shading()
    _add_camera()
    _add_lighting()
    _setup_render()
    # only use alpha composing for still renders
    _setup_compositing(use_alpha=not motions)

    # we're done!
    path = Path.cwd() / f"{name}.blend"
    bpy.ops.wm.save_mainfile(filepath=str(path))
    return armature


def render_model(mech_name: str, obj: bpy.types.Object) -> None:
    # we are free to modify the the values here, since the file has already been saved
    render = bpy.context.scene.render
    camera = bpy.data.objects["camera"]
    render.image_settings.file_format = "PNG"

    render.filepath = f"//{mech_name}_persp.png"
    print(f"Rendering '{render.filepath}'")
    bpy.ops.render.render(write_still=True)

    # set values
    render.use_file_extension = False
    camera.data.type = "ORTHO"
    camera.data.ortho_scale = 15.0
    camera.data.shift_y = -0.1

    render.filepath = f"//{mech_name}_front.png"
    print(f"Rendering '{render.filepath}'")
    bpy.ops.render.render(write_still=True)

    camera.location[1] = -15.0
    camera.rotation_euler[2] = 0.0
    render.filepath = f"//{mech_name}_back.png"
    print(f"Rendering '{render.filepath}'")
    bpy.ops.render.render(write_still=True)

    camera.location[1] = 0.0
    camera.location[0] = 15.0
    camera.rotation_euler[2] = radians(90.0)
    render.filepath = f"//{mech_name}_right.png"
    print(f"Rendering '{render.filepath}'")
    bpy.ops.render.render(write_still=True)

    camera.location[0] = -15.0
    camera.rotation_euler[2] = radians(-90.0)
    render.filepath = f"//{mech_name}_left.png"
    print(f"Rendering '{render.filepath}'")
    bpy.ops.render.render(write_still=True)


def render_motions(
    name: str, obj: bpy.types.Object, motions: Mapping[str, Mapping[str, Any]]
) -> None:
    # we are free to modify the the values here, since the file has already been saved
    scene = bpy.context.scene
    scene.render.use_file_extension = False
    for motion in motions.keys():
        # set action
        action = bpy.data.actions[motion]
        obj.animation_data.action = action

        # adjust frame range
        start, end = action.frame_range
        scene.frame_start = int(start)
        scene.frame_end = int(end)
        scene.frame_set(1)

        # render first frame of animation as a still
        scene.render.filepath = f"//{name}_{motion}.png"
        print(f"Rendering '{scene.render.filepath}'")

        scene.render.image_settings.file_format = "PNG"
        bpy.ops.render.render(write_still=True)

        # render entire animation
        scene.render.filepath = f"//{name}_{motion}.mp4"
        print(f"Rendering '{scene.render.filepath}'")

        scene.render.image_settings.file_format = "FFMPEG"
        bpy.ops.render.render(animation=True)


def main() -> None:
    parser = argparse.ArgumentParser(
        prog="blender --background --factory-startup --python mechlib2blend.py --",
        description="Convert dumped MechWarrior 3 'mech model data to a Blender file.",
    )
    parser.add_argument(
        "mechlib",
        type=lambda value: Path(value).resolve(strict=True),
        help="Path to 'mechlib.zip'",
    )
    parser.add_argument("mech_name", help="The model to convert")
    parser.add_argument(
        "--motion",
        type=lambda value: Path(value).resolve(strict=True),
        help="If specified, animation are loaded from this archive.",
    )
    parser.add_argument(
        "--mechtex",
        type=lambda value: Path(value).resolve(strict=True),
        help=(
            "If specified, textures are loaded from this archive. "
            "(Use 'rmechtex.zip' for the highest quality.)"
        ),
    )
    parser.add_argument(
        "--render",
        action="store_true",
        help="If specified, motions are rendered.",
    )

    # split our arguments from Blender arguments
    argv = sys.argv
    argv = argv[argv.index("--") + 1 :]

    args = parser.parse_args(argv)

    print(f"Converting '{args.mech_name}.blend'")

    with ZipFile(args.mechlib) as zipfile:
        with zipfile.open("materials.json", "r") as f:
            materials = json.load(f)

        with zipfile.open(f"mech_{args.mech_name}.json", "r") as f:
            model = json.load(f)

    motions = {}
    if args.motion:
        # normalize mech_1 low poly models
        name = args.mech_name.strip("_1")
        # in reader.zbd, the vulture uses the madcat animations
        if name == "vulture":
            name = "madcat"
        with ZipFile(args.motion) as zipfile:
            for fileinfo in zipfile.filelist:
                if not fileinfo.filename.startswith(name):
                    continue

                _, _, motion_name_json = fileinfo.filename.partition("_")
                motion_name, _, _ = motion_name_json.partition(".")
                with zipfile.open(fileinfo, "r") as f:
                    motions[motion_name.lower()] = json.load(f)

    with MaterialFactory.with_tempdir(args.mechtex, materials) as material_factory:
        obj = model_to_blend(model, args.mech_name, material_factory, motions)
        if args.render:
            if motions:
                render_motions(args.mech_name, obj, motions)
            else:
                render_model(args.mech_name, obj)


if __name__ == "__main__":
    main()
