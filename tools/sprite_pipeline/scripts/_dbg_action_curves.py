"""Inspect action fcurves to detect bone-vs-root-translation."""
import sys, bpy
argv = sys.argv[sys.argv.index("--")+1:]
glb = argv[0]
target = argv[1]  # action name
bpy.ops.wm.read_factory_settings(use_empty=True)
bpy.ops.import_scene.gltf(filepath=glb)

a = bpy.data.actions.get(target)
if a is None:
    print(f"action '{target}' not found. available:")
    for x in bpy.data.actions: print(f"  {x.name}")
    sys.exit(1)

print(f"=== action '{target}' frames {a.frame_range[0]}-{a.frame_range[1]} ===")
slots = getattr(a, "slots", None)
layers = getattr(a, "layers", None)
total_fcurves = 0
bone_paths = set()
non_bone_paths = set()
if layers:
    for layer in layers:
        for strip in getattr(layer, "strips", []):
            for s in (slots or []):
                cb = strip.channelbag(s) if hasattr(strip, "channelbag") else None
                if cb is None: continue
                for fc in getattr(cb, "fcurves", []):
                    total_fcurves += 1
                    if "pose.bones" in fc.data_path:
                        bone_paths.add(fc.data_path.split('"')[1])
                    else:
                        non_bone_paths.add(fc.data_path)
print(f"total fcurves: {total_fcurves}")
print(f"bones with curves ({len(bone_paths)}): {sorted(bone_paths)[:20]}")
print(f"non-bone paths ({len(non_bone_paths)}): {sorted(non_bone_paths)[:10]}")
