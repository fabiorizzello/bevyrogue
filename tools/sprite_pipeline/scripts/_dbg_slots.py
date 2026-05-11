import sys, bpy
argv = sys.argv[sys.argv.index("--")+1:]
path = argv[0]
bpy.ops.wm.read_factory_settings(use_empty=True)
ext = path.lower().split(".")[-1]
if ext in ("glb","gltf"):
    bpy.ops.import_scene.gltf(filepath=path)
else:
    bpy.ops.import_scene.fbx(filepath=path)

print("=== OBJECTS ===")
for o in bpy.data.objects:
    print(f"  {o.type} '{o.name}' loc={tuple(round(x,3) for x in o.location)} dim={tuple(round(x,3) for x in o.dimensions)}")
    if o.type == "MESH":
        print(f"    materials={[m.name if m else None for m in o.data.materials]}")
        print(f"    vert_count={len(o.data.vertices)} parent={o.parent.name if o.parent else None}")
print("=== ACTIONS ===")
for a in bpy.data.actions:
    fr = list(a.frame_range)
    print(f"  '{a.name}' frames={fr[0]:.0f}-{fr[1]:.0f}")
    slots = getattr(a,"slots",None)
    if slots:
        for s in slots:
            print(f"    slot id={s.identifier} type={s.target_id_type}")
