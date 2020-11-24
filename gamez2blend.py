import argparse
import json
import sys
from contextlib import contextmanager
from dataclasses import dataclass
from itertools import chain
from math import radians
from pathlib import Path
from tempfile import TemporaryDirectory
from typing import Any, Dict, Iterator, List, Mapping, Optional, Sequence, Tuple
from zipfile import ZipFile

import bmesh  # type: ignore
import bpy  # type: ignore
from mathutils import Euler, Vector  # type: ignore


@dataclass
class Node:
    node_type: str
    data: Mapping[str, Any]
    children: List["Node"]


class MaterialFactory:
    @classmethod
    @contextmanager
    def with_tempdir(
        cls,
        textures: Sequence[Path],
        materials: Sequence[Mapping[str, Any]],
    ) -> Iterator["MaterialFactory"]:
        with TemporaryDirectory() as tempdir:
            yield cls(textures, materials, Path(tempdir))

    def __init__(
        self,
        textures: Sequence[Path],
        materials: Sequence[Mapping[str, Any]],
        tempdir: Path,
    ):
        self.tempdir = Path(tempdir)
        self.textures = [(tex, ZipFile(tex)) for tex in textures]
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

    def _extract_texture(self, filename: str) -> Path:
        for zp, zf in self.textures:
            path = self.tempdir / zp.stem
            try:
                zf.extract(filename, path=str(path))
            except KeyError:
                pass
            else:
                return path / filename
        raise KeyError(filename)

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

            red, green, blue = color_info["color"]
            bsdf.inputs[0].default_value = (red / 255.0, green / 255.0, blue / 255.0, 1)
        else:
            # textured material
            texture_name, _, _extension = texture_info["texture"].partition(".")
            mat, bsdf = self._create_material(f"{texture_name} {texture_info['flag']}")
            # mat, bsdf = self._create_material(texture_name)

            if self.textures:
                # load image from archive
                filename = f"{texture_name}.png"
                try:
                    path = self._extract_texture(filename)
                except KeyError:
                    print("Image", texture_name, "not found in archive")
                    image = self._default_image()
                else:
                    image = bpy.data.images.load(str(path))
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
            mat.node_tree.links.new(bsdf.inputs["Alpha"], tex.outputs["Alpha"])
            mat.blend_method = "BLEND"
            mat.show_transparent_back = False

        self.cache[texture_index] = mat
        return mat


def _process_mesh(
    mesh_name: str,
    mesh_data: Mapping[str, Any],
    material_factory: MaterialFactory,
) -> Tuple[bpy.types.Mesh, Mapping[int, int]]:
    mesh = bpy.data.meshes.new(mesh_name)

    vertices = mesh_data["vertices"]
    # skipped: normals
    # Blender doesn't really deal with vertex normals, instead it's per face
    # normals = mesh_data["normals"]
    polygons = mesh_data["polygons"]

    textures = set()
    for poly in polygons:
        textures.add(poly["texture_index"])

    # faces have to be assigned a material index, not a material name
    tex_index_to_mat_index = {
        tex_index: mat_index for mat_index, tex_index in enumerate(textures)
    }

    bm = bmesh.new(use_operators=True)
    for vert in vertices:
        bm.verts.new(vert)

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
            for index_in_mesh, loop, color in zip(vertex_indices, face.loops, colors):
                assert loop.vert.index == index_in_mesh
                loop[color_layer] = (
                    color[0] / 255.0,
                    color[1] / 255.0,
                    color[2] / 255.0,
                    1.0,
                )

        if uvs:
            # because all our faces are created as loops, this should work
            for index_in_mesh, loop, uv in zip(vertex_indices, face.loops, uvs):
                assert loop.vert.index == index_in_mesh
                loop[uv_layer].uv = uv

    # since normals can't be loaded, and we've set smooth shading, calculate them
    bmesh.ops.recalc_face_normals(bm, faces=bm.faces)
    bm.to_mesh(mesh)
    bm.free()

    return mesh, tex_index_to_mat_index


def _process_node(
    node: Node,
    parent: Optional[bpy.types.Object],
    meshes: Sequence[Tuple[bpy.types.Mesh, Mapping[int, int]]],
    material_factory: MaterialFactory,
    collection: bpy.types.Collection,
) -> None:
    name = node.data["name"]
    if node.node_type == "Object3d":
        mesh_index = node.data["mesh_index"]

        if mesh_index < 0:
            obj = bpy.data.objects.new(name, None)  # empty
        else:
            mesh, tex_index_to_mat_index = meshes[mesh_index]
            obj = bpy.data.objects.new(name, mesh)
            for tex_index, _mat_index in tex_index_to_mat_index.items():
                mat = material_factory(tex_index)
                obj.data.materials.append(mat)
    else:
        name = f"{node.node_type} {name}"
        obj = bpy.data.objects.new(name, None)  # empty

    transformation = node.data.get("transformation")
    if transformation:
        obj.location = Vector(transformation["translation"])
        obj.rotation_euler = Euler(transformation["rotation"])
    else:
        obj.location = Vector((0.0, 0.0, 0.0))
        obj.rotation_euler = Euler((0.0, 0.0, 0.0))

    collection.objects.link(obj)

    if parent:
        obj.parent = parent
    else:
        # hack to convert Y-up model to Blender's coordinate system
        obj.rotation_euler = (radians(90.0), radians(0.0), radians(180.0))
        # scale it down, since the entire map is huge
        obj.scale = (0.05, 0.05, 0.05)

    for child in node.children:
        _process_node(child, obj, meshes, material_factory, collection)


def _setup_render() -> None:
    # set up render properties
    render = bpy.context.scene.render
    render.ffmpeg.format = "MPEG4"
    render.ffmpeg.codec = "H264"
    render.ffmpeg.constant_rate_factor = "MEDIUM"
    render.use_compositing = True
    render.resolution_x = 1080
    render.film_transparent = True


def _set_relationship_lines() -> None:
    for area in bpy.context.workspace.screens[0].areas:
        for space in area.spaces:
            if space.type == "VIEW_3D":
                space.overlay.show_relationship_lines = False


def world_to_blend(
    nodes: Sequence[Node],
    material_factory: MaterialFactory,
    meshes: Sequence[Mapping[str, Any]],
    name: str,
) -> None:
    # empty scene, this prints some errors
    bpy.ops.wm.read_factory_settings(use_empty=True)

    mesh_objs = [
        _process_mesh(f"{i:04d}", mesh_data, material_factory)
        for i, mesh_data in enumerate(meshes)
    ]

    # move non-world children to a different layer
    collection = bpy.context.scene.collection
    world_collection = bpy.data.collections.new("world")
    collection.children.link(world_collection)
    other_collection = bpy.data.collections.new("other")
    collection.children.link(other_collection)
    other_collection.hide_viewport = True
    other_collection.hide_render = True

    for node in nodes:
        if node.node_type == "World":
            collection = world_collection
        else:
            collection = other_collection
        _process_node(node, None, mesh_objs, material_factory, collection)

    # finally, set up the scene some more
    _setup_render()
    # too many empties...
    _set_relationship_lines()

    # we're done!
    bpy.ops.wm.save_mainfile(filepath=f"{name}.blend")


def _node_array_to_tree(nodes: Sequence[Any]) -> Sequence[Node]:
    def flatten_node(node: Mapping[str, Any]) -> Node:
        items = node.items()
        # get the first key and value
        assert len(items) == 1, items
        node_type, node_data = next(iter(items))
        return Node(node_type, node_data, [])

    flattened = [flatten_node(node) for node in nodes]

    unparented = []
    for node in flattened:
        if node.node_type == "Empty":
            continue

        parent_index = node.data.get("parent")
        if parent_index is None:
            unparented.append(node)
        else:
            flattened[parent_index].children.append(node)

    return unparented


def main() -> None:
    parser = argparse.ArgumentParser(
        prog="blender --background --factory-startup --python gamez2blend.py --",
        description="Convert dumped MechWarrior 3 world data to a Blender file.",
    )
    parser.add_argument(
        "gamez",
        type=lambda value: Path(value).resolve(strict=True),
        help="Path to 'gamez.zip'",
    )
    parser.add_argument(
        "--rtexture",
        type=lambda value: Path(value).resolve(strict=True),
        help=(
            "If specified, textures are loaded from this archive. "
            "(Use 'rtexture.zip' for the highest quality.)"
        ),
    )
    parser.add_argument(
        "--rmechtex",
        type=lambda value: Path(value).resolve(strict=True),
        help=(
            "If specified, textures are loaded from this archive. "
            "(Use 'rmechtex.zip' for the highest quality.)"
        ),
    )

    # split our arguments from Blender arguments
    argv = sys.argv
    argv = argv[argv.index("--") + 1 :]

    args = parser.parse_args(argv)

    with ZipFile(args.gamez) as zipfile:
        with zipfile.open("materials.json", "r") as f:
            materials = json.load(f)

        with zipfile.open(f"nodes.json", "r") as f:
            nodes = json.load(f)

        with zipfile.open(f"meshes.json", "r") as f:
            meshes = json.load(f)

    unparented = _node_array_to_tree(nodes)

    textures = [tex for tex in (args.rtexture, args.rmechtex) if tex is not None]
    with MaterialFactory.with_tempdir(textures, materials) as material_factory:
        world_to_blend(unparented, material_factory, meshes, args.gamez.stem)


if __name__ == "__main__":
    main()
