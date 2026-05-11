import bpy
import sys
import argparse

def parse_args():
    parser = argparse.ArgumentParser()
    # Find the '--' separator
    if '--' in sys.argv:
        argv = sys.argv[sys.argv.index('--') + 1:]
    else:
        argv = []
    parser.add_argument("--model", required=True)
    return parser.parse_args(argv)

def main():
    args = parse_args()
    
    # Clear scene
    bpy.ops.wm.read_factory_settings(use_empty=True)
    
    # Import model
    bpy.ops.import_scene.fbx(filepath=args.model)
    
    print("\n=== MESH & MATERIALS ===")
    for obj in bpy.data.objects:
        if obj.type == 'MESH':
            mats = [slot.material.name if slot.material else "NONE" for slot in obj.material_slots]
            print(f"  [MESH] {obj.name} (mats: {', '.join(mats)})")
    
    print("\n=== DONE ===\n")

if __name__ == "__main__":
    main()
