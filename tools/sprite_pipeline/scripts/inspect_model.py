"""Inspect 3D model: list objects, armatures, actions, frame ranges.

Usage:
    blender -b -P inspect_model.py -- --model /path/to/model.fbx
"""

import bpy
import sys
from pathlib import Path


def parse_args():
    if "--" not in sys.argv:
        raise SystemExit("Pass args after '--': blender -b -P script.py -- --model X.fbx")
    argv = sys.argv[sys.argv.index("--") + 1:]
    if "--model" not in argv:
        raise SystemExit("Missing --model")
    return argv[argv.index("--model") + 1]


def import_model(path):
    ext = Path(path).suffix.lower()
    if ext == ".fbx":
        bpy.ops.import_scene.fbx(filepath=path)
    elif ext in (".glb", ".gltf"):
        bpy.ops.import_scene.gltf(filepath=path)
    elif ext == ".obj":
        bpy.ops.wm.obj_import(filepath=path)
    elif ext == ".blend":
        with bpy.data.libraries.load(path) as (data_from, data_to):
            data_to.objects = data_from.objects
        for obj in data_to.objects:
            if obj is not None:
                bpy.context.collection.objects.link(obj)
    else:
        raise SystemExit(f"Unsupported format: {ext}")


def main():
    bpy.ops.wm.read_factory_settings(use_empty=True)
    model_path = parse_args()
    print(f"\n=== INSPECTING: {model_path} ===\n")
    import_model(model_path)

    print("--- OBJECTS ---")
    for obj in bpy.data.objects:
        print(f"  [{obj.type}] {obj.name}  (loc={tuple(round(c, 2) for c in obj.location)})")

    print("\n--- ARMATURES ---")
    armatures = [o for o in bpy.data.objects if o.type == 'ARMATURE']
    if not armatures:
        print("  (none)")
    for arm in armatures:
        print(f"  {arm.name}  bones={len(arm.data.bones)}")
        for bone in arm.data.bones[:10]:
            print(f"    - {bone.name}")
        if len(arm.data.bones) > 10:
            print(f"    ... ({len(arm.data.bones) - 10} more)")

    print("\n--- ACTIONS ---")
    if not bpy.data.actions:
        print("  (none — no animations baked)")
    for action in bpy.data.actions:
        try:
            frame_range = action.frame_range
            n_curves = len(action.fcurves) if hasattr(action, 'fcurves') else sum(
                len(layer.strips) for layer in getattr(action, 'layers', [])
            )
            print(f"  {action.name}  frames={int(frame_range[0])}-{int(frame_range[1])}  curves/strips={n_curves}")
        except Exception as e:
            print(f"  {action.name}  (error reading: {e})")

    print("\n--- MESHES ---")
    meshes = [o for o in bpy.data.objects if o.type == 'MESH']
    for mesh in meshes:
        print(f"  {mesh.name}  verts={len(mesh.data.vertices)}  polys={len(mesh.data.polygons)}")

    print("\n--- BOUNDING BOX (combined) ---")
    if meshes:
        all_verts = []
        for m in meshes:
            for v in m.bound_box:
                world_v = m.matrix_world @ type(m.location)(v)
                all_verts.append(world_v)
        if all_verts:
            xs = [v.x for v in all_verts]
            ys = [v.y for v in all_verts]
            zs = [v.z for v in all_verts]
            print(f"  min=({min(xs):.2f}, {min(ys):.2f}, {min(zs):.2f})")
            print(f"  max=({max(xs):.2f}, {max(ys):.2f}, {max(zs):.2f})")
            print(f"  size=({max(xs)-min(xs):.2f}, {max(ys)-min(ys):.2f}, {max(zs)-min(zs):.2f})")

    print("\n=== DONE ===\n")


if __name__ == "__main__":
    main()
