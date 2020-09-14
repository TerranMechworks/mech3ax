import argparse
import json
import sys
from math import radians
from pathlib import Path
from typing import Any, Dict, Mapping, Optional, Sequence, cast
from zipfile import ZipFile

import bmesh  # type: ignore
import bpy  # type: ignore

Mesh = Mapping[str, Any]
Meshes = Sequence[Mesh]


def _process_mesh(
    name: str, mesh_data: Mesh, material_factory: "MaterialFactory"
) -> Any:  # pylint: disable=too-many-locals
    mesh = bpy.data.meshes.new(name=name)
    obj = bpy.data.objects.new(name, mesh)

    vertices = mesh_data["vertices"]
    # skipped: normals
    polygons = mesh_data["polygons"]

    textures = set()
    for poly in polygons:
        textures.add(poly["texture_index"])

    tex_index_to_mat_index = {}
    for material_index, texture_index in enumerate(textures):
        mat = material_factory(texture_index)
        obj.data.materials.append(mat)
        tex_index_to_mat_index[texture_index] = material_index

    # it may be possible to do this with a normal Mesh object, similar
    # to Blender's import_obj.py add-on. But this works.
    bm = bmesh.new()
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
        uvs = poly["uv_coords"]
        texture_index = poly["texture_index"]

        try:
            face = bm.faces.new(bm.verts[i] for i in vertex_indices)
        except ValueError as e:
            # some models contain duplicate faces. they are identical, so
            # skipping them seems harmless
            print("Mesh", name, "error:", str(e), "ptr:", poly["vertices_ptr"])
            continue

        face.material_index = tex_index_to_mat_index[texture_index]

        if colors:
            for index_in_mesh, loop, color in zip(vertex_indices, face.loops, colors):
                assert loop.vert.index == index_in_mesh
                loop[color_layer] = (color[0] / 255.0, color[1] / 255.0, color[2] / 255.0, 1.0)

        if uvs:
            # because all our faces are created as loops, this should work
            for index_in_mesh, loop, uv in zip(vertex_indices, face.loops, uvs):
                assert loop.vert.index == index_in_mesh
                loop[uv_layer].uv = uv

    bm.to_mesh(mesh)
    bm.free()

    return obj


def _process_node(
    node: Mapping[str, Any],
    parent: Any,
    material_factory: "MaterialFactory",
    meshes: Meshes,
) -> Any:
    object3d = node["Object3d"]
    name = object3d["name"]
    mesh_index = object3d["mesh_index"]

    if mesh_index < 0:
        obj = bpy.data.objects.new(name, None)  # empty
    else:
        mesh_data = meshes[mesh_index]
        obj = _process_mesh(name, mesh_data, material_factory)

    transformation = object3d["transformation"]
    if transformation:
        obj.location = transformation["translation"]
        obj.rotation_euler = transformation["rotation"]
    else:
        obj.location = (0.0, 0.0, 0.0)
        obj.rotation_euler = (0.0, 0.0, 0.0)

    # move the "head" object to a different layer
    if name == "head":
        head = bpy.data.collections.new("head")
        bpy.context.scene.collection.children.link(head)
        head.hide_viewport = True
        head.hide_render = True
        head.objects.link(obj)
    else:
        bpy.context.collection.objects.link(obj)

    if parent:
        obj.parent = parent

    children = object3d["children"]
    for child in children:
        _process_node(child, obj, material_factory, meshes)

    return obj


def _add_camera() -> None:
    camera = bpy.data.cameras.new("Camera")
    camera.lens = 18

    camera_obj = bpy.data.objects.new("Camera", camera)
    camera_obj.location = (0.0, -60.0, 8.0)
    camera_obj.rotation_euler = (radians(80), 0.0, 0.0)

    bpy.context.scene.collection.objects.link(camera_obj)
    bpy.context.scene.camera = camera_obj


def _set_shading() -> None:
    for area in bpy.context.workspace.screens[0].areas:
        for space in area.spaces:
            if space.type == "VIEW_3D":
                space.shading.type = "MATERIAL"


def model_to_blend(
    model: Mapping[str, Any], material_factory: "MaterialFactory", name: str
) -> None:
    # empty scene, this prints some errors
    bpy.ops.wm.read_factory_settings(use_empty=True)

    root_node = model["root"]
    meshes = cast(Meshes, model["meshes"])

    root_obj = _process_node(root_node, None, material_factory, meshes)
    # hack to convert Y-up model to Blender's coordinate system
    root_obj.rotation_euler = (radians(90), radians(0), radians(180))

    _add_camera()
    _set_shading()
    bpy.context.scene.render.filepath = f"//{name}_##"

    bpy.ops.wm.save_mainfile(filepath=f"{name}.blend")


class MaterialFactory:
    def __init__(
        self,
        textures: Optional[str],
        materials: Sequence[Mapping[str, Any]],
        extract: bool,
    ):
        self.textures = ZipFile(textures) if textures else None
        self.materials = materials
        self.cache: Dict[int, Any] = {}
        self.extract = extract

    def __call__(self, texture_index: int) -> Any:
        try:
            return self.cache[texture_index]
        except KeyError:
            pass

        material_info = self.materials[texture_index]

        try:
            texture_info = material_info["Textured"]
        except KeyError:
            color_info = material_info["Colored"]
            # untextured
            mat = bpy.data.materials.new(f"material_{texture_index}")
            mat.use_nodes = True

            red, green, blue = color_info["color"]
            bsdf = mat.node_tree.nodes["Principled BSDF"]
            bsdf.inputs[0].default_value = (red / 255.0, green / 255.0, blue / 255.0, 1)
        else:
            texture_name = texture_info["texture"]
            mat = bpy.data.materials.new(texture_name)
            mat.use_nodes = True
            bsdf = mat.node_tree.nodes["Principled BSDF"]

            if self.textures:
                if self.extract:
                    self.textures.extract(f"{texture_name}.png")
                image = bpy.data.images.load(f"//{texture_name}.png")

                tex = mat.node_tree.nodes.new("ShaderNodeTexImage")
                tex.image = image

                mat.node_tree.links.new(bsdf.inputs["Base Color"], tex.outputs["Color"])

        self.cache[texture_index] = mat
        return mat


def main() -> None:
    parser = argparse.ArgumentParser(
        prog="blender --background --factory-startup --python mechlib2blend.py --",
        description="Convert dumped MechWarrior 3 'mech model data to Blender files.",
    )
    parser.add_argument(
        "mechlib",
        type=lambda value: Path(value).resolve(strict=True),
        help="A path to 'mechlib.zip'",
    )
    parser.add_argument("model_name", help="The model to convert")
    parser.add_argument(
        "--mechtex",
        type=lambda value: Path(value).resolve(strict=True),
        help=(
            "If specified, textures are loaded from this archive. "
            "(Use 'rmechtex.zip' for the highest quality.)"
        ),
    )
    parser.add_argument(
        "--no-extract",
        action="store_true",
        help="If specified, textures are not extracted.",
    )

    # split our arguments from Blender arguments
    argv = sys.argv
    argv = argv[argv.index("--") + 1 :]

    args = parser.parse_args(argv)

    name = f"{args.model_name}"

    print(f"Converting '{args.model_name}' to '{name}.blend'")

    with ZipFile(args.mechlib) as zipfile:
        with zipfile.open("materials.json", "r") as f:
            materials = json.load(f)

        with zipfile.open(f"mech_{args.model_name}.json", "r") as f:
            model = json.load(f)

    material_factory = MaterialFactory(args.mechtex, materials, not args.no_extract)
    model_to_blend(model, material_factory, name)


if __name__ == "__main__":
    main()
