"""Headless inspection: import FBX, run shader.apply, dump material state.

Usage:
  blender -b -P _inspect_after_shader.py -- <fbx_path> <texture_path> <shader_name>
"""
import bpy, sys, importlib.util
from pathlib import Path

argv = sys.argv[sys.argv.index("--") + 1:]
fbx, tex, shader_name = argv[0], argv[1], argv[2]

bpy.ops.wm.read_factory_settings(use_empty=True)
bpy.ops.import_scene.fbx(filepath=fbx, automatic_bone_orientation=True, ignore_leaf_bones=True, use_anim=True)

print("=== AFTER FBX IMPORT ===")
for obj in bpy.data.objects:
    if obj.type == 'MESH':
        slots = [s.material.name if s.material else "<None>" for s in obj.material_slots]
        print(f"MESH {obj.name}: slots={slots}")
print(f"Total bpy.data.materials: {[m.name for m in bpy.data.materials]}")

# Load shader
base = Path(__file__).parent / "shaders" / f"{shader_name}.py"
spec = importlib.util.spec_from_file_location(f"shaders.{shader_name}", base)
mod = importlib.util.module_from_spec(spec)
spec.loader.exec_module(mod)

print(f"\n=== APPLYING SHADER {shader_name} ===")
mod.apply(tex, outline_meshes=[])

print("\n=== AFTER SHADER APPLY ===")
for obj in bpy.data.objects:
    if obj.type == 'MESH':
        slots = [s.material.name if s.material else "<None>" for s in obj.material_slots]
        print(f"MESH {obj.name}: slots={slots}")
        # Inspect first slot's material output wiring
        if obj.material_slots and obj.material_slots[0].material:
            mat = obj.material_slots[0].material
            print(f"  slot[0] mat='{mat.name}' use_nodes={mat.use_nodes}")
            if mat.use_nodes:
                out_node = next((n for n in mat.node_tree.nodes if n.type == 'OUTPUT_MATERIAL'), None)
                if out_node:
                    surf = out_node.inputs.get('Surface')
                    if surf and surf.is_linked:
                        link = surf.links[0]
                        print(f"  Output.Surface <- {link.from_node.bl_idname}/{link.from_node.name}.{link.from_socket.name}")
                    else:
                        print(f"  Output.Surface NOT LINKED")
                else:
                    print(f"  No OUTPUT_MATERIAL node!")
print(f"\nTotal bpy.data.materials AFTER: {[m.name for m in bpy.data.materials]}")
