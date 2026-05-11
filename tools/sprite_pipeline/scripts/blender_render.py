"""Blender headless render: 3D model → frame PNG hi-res orthographic.

Usage:
    blender -b -P blender_render.py -- --config configs/agumon.json

Config JSON (auto-camera mode):
{
    "model_path": "/abs/path/model.fbx",
    "output_root": "/abs/path/raw_renders/agumon",
    "render_size": 256,
    "ortho_scale": 3.0,
    "cam_distance": 5.0,        // distance from char along selected axis
    "cam_axis": "x",            // "x" = side view (+X), "y" = front (+Y), "-x", "-y"
    "light_energy": 5.0,
    "world_strength": 1.0,
    "animations": [
        {
            "name": "idle",
            "action_name": "...",   // exact action name (run inspect_model.py first)
            "frame_start": 1,
            "frame_end": 81,
            "frame_step": 10
        }
    ]
}
"""

import bpy
import sys
import json
import importlib.util
from pathlib import Path
from math import radians
from mathutils import Vector


def load_shader_module(name):
    """Dynamically load a shader variant from scripts/shaders/{name}.py."""
    base = Path(__file__).parent / "shaders" / f"{name}.py"
    if not base.exists():
        raise SystemExit(f"Shader module not found: {base}")
    spec = importlib.util.spec_from_file_location(f"shaders.{name}", base)
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    return mod


def parse_args():
    if "--" not in sys.argv:
        raise SystemExit("Pass args after '--': blender -b -P script.py -- --config X.json")
    argv = sys.argv[sys.argv.index("--") + 1:]
    if "--config" not in argv:
        raise SystemExit("Missing --config")
    return argv[argv.index("--config") + 1]


def clear_scene():
    bpy.ops.wm.read_factory_settings(use_empty=True)


def import_model(path, fbx_opts=None):
    ext = Path(path).suffix.lower()
    if ext == ".fbx":
        # Game-exported FBX (mobile/Unity) typically need automatic bone orientation,
        # otherwise primary_bone_axis defaults distort mesh deformation on import.
        # Per-char override via cfg["fbx_import_opts"] (e.g. Digimon Links Renamon
        # needs automatic_bone_orientation=False + primary_bone_axis explicit because
        # action keyframes target original axis, not blender-reoriented axis).
        opts = {
            "automatic_bone_orientation": True,
            "ignore_leaf_bones": True,
            "use_anim": True,
        }
        if fbx_opts:
            opts.update(fbx_opts)
        bpy.ops.import_scene.fbx(filepath=path, **opts)
    elif ext in (".glb", ".gltf"):
        bpy.ops.import_scene.gltf(filepath=path)
    elif ext == ".obj":
        bpy.ops.wm.obj_import(filepath=path)
    elif ext == ".dae":
        bpy.ops.wm.collada_import(filepath=path)
    elif ext == ".blend":
        with bpy.data.libraries.load(path) as (data_from, data_to):
            data_to.objects = data_from.objects
        for obj in data_to.objects:
            if obj is not None:
                bpy.context.collection.objects.link(obj)
    else:
        raise SystemExit(f"Unsupported model format: {ext}")


def get_combined_bbox():
    """Compute combined bounding box of all VISIBLE mesh objects in world space (current frame)."""
    meshes = [o for o in bpy.data.objects if o.type == 'MESH' and not o.hide_render]
    if not meshes:
        return Vector((-1, -1, -1)), Vector((1, 1, 1))

    all_corners = []
    for m in meshes:
        # Use evaluated mesh (with armature deformation applied)
        depsgraph = bpy.context.evaluated_depsgraph_get()
        m_eval = m.evaluated_get(depsgraph)
        mat = m_eval.matrix_world
        if m_eval.data and hasattr(m_eval.data, 'vertices'):
            for v in m_eval.data.vertices:
                world_v = mat @ v.co
                all_corners.append(world_v)

    if not all_corners:
        return Vector((-1, -1, -1)), Vector((1, 1, 1))

    bb_min = Vector((
        min(c.x for c in all_corners),
        min(c.y for c in all_corners),
        min(c.z for c in all_corners),
    ))
    bb_max = Vector((
        max(c.x for c in all_corners),
        max(c.y for c in all_corners),
        max(c.z for c in all_corners),
    ))
    return bb_min, bb_max


def get_animation_bbox(frame_start, frame_end, frame_step):
    """Sample bbox across animation frames, return union extents."""
    scn = bpy.context.scene
    all_min = Vector((float('inf'),) * 3)
    all_max = Vector((float('-inf'),) * 3)

    for frame in range(frame_start, frame_end + 1, max(1, frame_step // 2)):
        scn.frame_set(frame)
        bb_min, bb_max = get_combined_bbox()
        all_min = Vector((min(all_min[i], bb_min[i]) for i in range(3)))
        all_max = Vector((max(all_max[i], bb_max[i]) for i in range(3)))

    return all_min, all_max


def setup_camera_auto(cfg, bb_min=None, bb_max=None):
    """Position camera based on bbox + cam_axis + cam_distance.

    Auto-frames char: ortho_scale = max(visible dim) * padding unless explicitly set.
    If bb_min/bb_max provided, use them (e.g. animation-aware bbox).
    """
    if bb_min is None or bb_max is None:
        bb_min, bb_max = get_combined_bbox()
    center = (bb_min + bb_max) / 2
    size = bb_max - bb_min
    print(f"[INFO] Bbox center: {center}")
    print(f"[INFO] Bbox size:   {size}")

    distance = cfg.get("cam_distance", 5.0)
    axis = cfg.get("cam_axis", "x")
    world_up = cfg.get("world_up", "z")

    # Iso preset: yaw (around vertical axis) + pitch (above horizon)
    # All presets render top-down looking (camera ABOVE char looking down).
    iso_presets = {
        # iso_45 = elevated side view; yaw=270 mirrors yaw=90 to face char right.
        "iso_45":      {"yaw": 270, "pitch": 35},
        "iso_30":      {"yaw": 270, "pitch": 50},
        "iso_threequarter": {"yaw": 270, "pitch": 25},
        # Alt diagonals:
        "iso_45_diag":  {"yaw": 315, "pitch": 35},
        "iso_135":      {"yaw": 135, "pitch": 35},
        "iso_225":      {"yaw": 225, "pitch": 35},
    }

    if axis in iso_presets:
        from math import cos, sin
        preset = iso_presets[axis]
        yaw = radians(preset["yaw"])
        pitch = radians(preset["pitch"])
        # Spherical coords offset (world_up = Z assumed for iso presets)
        offset_iso = Vector((
            distance * cos(pitch) * sin(yaw),
            -distance * cos(pitch) * cos(yaw),
            distance * sin(pitch),
        ))
        # Visible dims approximate as max of all axes
        vis_w = max(size.x, size.y)
        vis_h = size.z + max(size.x, size.y) * 0.5  # height + iso depth bonus
    else:
        # Compute visible dimensions for axis (perpendicular to view direction)
        # Map axis → visible (width, height) bbox dimensions
        visible_dims_map = {
            ("z", "x"):   (size.y, size.z),
            ("z", "-x"):  (size.y, size.z),
            ("z", "y"):   (size.x, size.z),
            ("z", "-y"):  (size.x, size.z),
            ("z", "z"):   (size.x, size.y),
            ("z", "-z"):  (size.x, size.y),
            ("y", "x"):   (size.z, size.y),
            ("y", "-x"):  (size.z, size.y),
            ("y", "z"):   (size.x, size.y),
            ("y", "-z"):  (size.x, size.y),
            ("y", "y"):   (size.x, size.z),
            ("y", "-y"):  (size.x, size.z),
        }
        vis_w, vis_h = visible_dims_map[(world_up, axis)]
        offset_iso = None
    visible_max = max(vis_w, vis_h)

    # Auto ortho_scale unless explicit
    if cfg.get("ortho_scale_auto", False) or "ortho_scale" not in cfg:
        ortho_scale = visible_max * cfg.get("ortho_padding", 1.15)
    else:
        ortho_scale = cfg["ortho_scale"]

    cam_data = bpy.data.cameras.new("ortho_cam")
    cam_data.type = 'ORTHO'
    cam_data.ortho_scale = ortho_scale
    cam_obj = bpy.data.objects.new("ortho_cam", cam_data)
    bpy.context.collection.objects.link(cam_obj)

    # Position (iso uses pre-computed offset_iso)
    if offset_iso is not None:
        cam_obj.location = center + offset_iso
    else:
        offset_map = {
            "x": Vector((distance, 0, 0)),
            "-x": Vector((-distance, 0, 0)),
            "y": Vector((0, distance, 0)),
            "-y": Vector((0, -distance, 0)),
            "z": Vector((0, 0, distance)),
            "-z": Vector((0, 0, -distance)),
        }
        cam_obj.location = center + offset_map[axis]

    # Camera look-at via matrix construction (avoids to_track_quat ambiguity).
    # World up = Z (or Y). Camera local axes: forward=-Z, up=+Y, right=+X.
    forward = (center - cam_obj.location).normalized()
    world_up_vec = Vector((0, 0, 1)) if world_up == 'z' else Vector((0, 1, 0))
    # If forward is collinear with world_up, fallback to Y axis
    if abs(forward.dot(world_up_vec)) > 0.999:
        world_up_vec = Vector((0, 1, 0)) if world_up == 'z' else Vector((0, 0, 1))
    right = forward.cross(world_up_vec).normalized()
    cam_up = right.cross(forward).normalized()
    # Build 3x3 rotation matrix: camera +X = right, +Y = up, -Z = forward
    from mathutils import Matrix
    rot_mat = Matrix((
        (right.x,   cam_up.x,  -forward.x),
        (right.y,   cam_up.y,  -forward.y),
        (right.z,   cam_up.z,  -forward.z),
    ))
    cam_obj.rotation_mode = 'QUATERNION'
    cam_obj.rotation_quaternion = rot_mat.to_quaternion()

    bpy.context.scene.camera = cam_obj
    print(f"[INFO] Camera at {cam_obj.location}, target {center}, world_up={world_up}, axis={axis}")
    print(f"[INFO] Visible dims: {vis_w:.2f}×{vis_h:.2f}, ortho_scale={ortho_scale:.2f}")
    return cam_obj


def make_textured_material(texture_path, posterize_levels=4):
    """Create cel-shaded textured material with posterized emission for pixel art look."""
    img = bpy.data.images.load(texture_path)

    mat = bpy.data.materials.new(name="cel_textured")
    mat.use_nodes = True
    nodes = mat.node_tree.nodes
    links = mat.node_tree.links
    nodes.clear()

    uv_node = nodes.new('ShaderNodeUVMap')
    uv_node.uv_map = "UVMap"

    tex_node = nodes.new('ShaderNodeTexImage')
    tex_node.image = img
    tex_node.interpolation = 'Closest'

    # Posterize color via ColorRamp set to constant interpolation with N stops
    # → discretize colors into N levels per channel
    sep = nodes.new('ShaderNodeSeparateColor')
    sep.mode = 'RGB'
    combine = nodes.new('ShaderNodeCombineColor')
    combine.mode = 'RGB'

    # For each RGB channel, use Math floor(x*levels)/levels to posterize
    posterize_nodes = []
    for i, ch in enumerate(['Red', 'Green', 'Blue']):
        mul = nodes.new('ShaderNodeMath')
        mul.operation = 'MULTIPLY'
        mul.inputs[1].default_value = float(posterize_levels)

        floor = nodes.new('ShaderNodeMath')
        floor.operation = 'FLOOR'

        div = nodes.new('ShaderNodeMath')
        div.operation = 'DIVIDE'
        div.inputs[1].default_value = float(posterize_levels - 1)

        links.new(sep.outputs[ch], mul.inputs[0])
        links.new(mul.outputs[0], floor.inputs[0])
        links.new(floor.outputs[0], div.inputs[0])
        links.new(div.outputs[0], combine.inputs[ch])

    emission = nodes.new('ShaderNodeEmission')
    transparent = nodes.new('ShaderNodeBsdfTransparent')
    mix = nodes.new('ShaderNodeMixShader')
    output = nodes.new('ShaderNodeOutputMaterial')

    links.new(uv_node.outputs['UV'], tex_node.inputs['Vector'])
    links.new(tex_node.outputs['Color'], sep.inputs['Color'])
    links.new(combine.outputs['Color'], emission.inputs['Color'])
    links.new(tex_node.outputs['Alpha'], mix.inputs['Fac'])
    links.new(transparent.outputs['BSDF'], mix.inputs[1])
    links.new(emission.outputs['Emission'], mix.inputs[2])
    links.new(mix.outputs['Shader'], output.inputs['Surface'])
    mat.blend_method = 'CLIP'
    return mat


def make_outline_material(color=(0.0, 0.0, 0.0)):
    """Black material for inverted-hull cel-shading outline.

    Outline mesh has flipped normals (backfaces facing camera).
    Show only backfaces (Geometry > Backfacing == 1), discard frontfaces.
    """
    mat = bpy.data.materials.new(name="cel_outline")
    mat.use_nodes = True
    nodes = mat.node_tree.nodes
    links = mat.node_tree.links
    nodes.clear()

    emit = nodes.new('ShaderNodeEmission')
    emit.inputs['Color'].default_value = (*color, 1.0)
    emit.inputs['Strength'].default_value = 1.0

    transparent = nodes.new('ShaderNodeBsdfTransparent')

    geom = nodes.new('ShaderNodeNewGeometry')

    mix = nodes.new('ShaderNodeMixShader')

    output = nodes.new('ShaderNodeOutputMaterial')

    # Backfacing output: 1 if backface, 0 if front. Mix factor to switch shaders.
    # Fac=0 → first input (transparent for frontfaces)
    # Fac=1 → second input (black emission for backfaces)
    links.new(geom.outputs['Backfacing'], mix.inputs['Fac'])
    links.new(transparent.outputs['BSDF'], mix.inputs[1])
    links.new(emit.outputs['Emission'], mix.inputs[2])
    links.new(mix.outputs['Shader'], output.inputs['Surface'])
    mat.blend_method = 'CLIP'
    return mat


def bind_texture(texture_path, body_meshes=None, outline_meshes=None, posterize_levels=4, outline_flip_normals=True):
    """Apply textured cel material to body meshes, black material to outline meshes."""
    if not Path(texture_path).exists():
        print(f"[WARN] Texture file not found: {texture_path}")
        return

    print(f"[INFO] Loading texture: {texture_path}")
    body_mat = make_textured_material(texture_path, posterize_levels)
    outline_mat = make_outline_material()

    body_meshes = body_meshes or []
    outline_meshes = outline_meshes or []

    for obj in bpy.data.objects:
        if obj.type != 'MESH':
            continue
        obj.data.materials.clear()
        if obj.name in outline_meshes:
            obj.data.materials.append(outline_mat)
            # Flip normals so backfaces render as outer surface (cel-shading trick)
            if outline_flip_normals:
                bpy.context.view_layer.objects.active = obj
                obj.select_set(True)
                bpy.ops.object.mode_set(mode='EDIT')
                bpy.ops.mesh.select_all(action='SELECT')
                bpy.ops.mesh.flip_normals()
                bpy.ops.object.mode_set(mode='OBJECT')
                obj.select_set(False)
                print(f"[INFO] Flipped normals on {obj.name}")
            print(f"[INFO] Applied OUTLINE material to: {obj.name}")
        else:
            obj.data.materials.append(body_mat)
            print(f"[INFO] Applied BODY material to: {obj.name}")


def setup_lighting(cfg):
    world = bpy.context.scene.world
    if world is None:
        world = bpy.data.worlds.new("World")
        bpy.context.scene.world = world
    world.use_nodes = True
    bg_node = world.node_tree.nodes.get("Background")
    if bg_node:
        bg_node.inputs[1].default_value = cfg.get("world_strength", 1.0)

    light_data = bpy.data.lights.new("sun", type='SUN')
    light_data.energy = cfg.get("light_energy", 5.0)
    light_obj = bpy.data.objects.new("sun_light", light_data)
    bpy.context.collection.objects.link(light_obj)
    light_obj.location = (5, 5, 10)
    light_obj.rotation_euler = (radians(45), radians(0), radians(45))


def _enable_cycles_gpu():
    """Try OPTIX → CUDA → HIP → ONEAPI → METAL. Return backend name on success, None if no GPU."""
    try:
        prefs = bpy.context.preferences.addons['cycles'].preferences
    except KeyError:
        return None
    for backend in ('OPTIX', 'CUDA', 'HIP', 'ONEAPI', 'METAL'):
        try:
            prefs.compute_device_type = backend
        except TypeError:
            continue
        prefs.get_devices()
        gpu_devices = [d for d in prefs.devices if d.type == backend]
        if not gpu_devices:
            continue
        for d in prefs.devices:
            d.use = (d.type == backend)
        names = ", ".join(d.name for d in gpu_devices)
        print(f"[INFO] Cycles GPU enabled: {backend} ({names})")
        return backend
    print("[WARN] No Cycles GPU backend available — falling back to CPU.")
    return None


def setup_render(cfg):
    scn = bpy.context.scene
    scn.render.resolution_x = cfg["render_size"]
    scn.render.resolution_y = cfg["render_size"]
    scn.render.resolution_percentage = 100
    scn.render.film_transparent = True
    scn.render.image_settings.file_format = 'PNG'
    scn.render.image_settings.color_mode = 'RGBA'

    # Disable anti-aliasing for crisp pixel-art look
    if cfg.get("disable_aa", True):
        scn.render.filter_size = 0.0

    engine = cfg.get("render_engine", "CYCLES")
    if engine == "CYCLES":
        scn.render.engine = 'CYCLES'
        scn.cycles.samples = cfg.get("cycles_samples", 1)
        # AUTO: GPU only above sample threshold (GPU init overhead beats render gain at samples=1).
        cycles_device = cfg.get("cycles_device", "AUTO").upper()
        gpu_sample_threshold = cfg.get("gpu_sample_threshold", 16)
        force_gpu = cycles_device == "GPU"
        try_gpu = force_gpu or (
            cycles_device == "AUTO" and scn.cycles.samples >= gpu_sample_threshold
        )
        if try_gpu:
            backend = _enable_cycles_gpu()
            scn.cycles.device = 'GPU' if backend else 'CPU'
        else:
            scn.cycles.device = 'CPU'
            if cycles_device == "AUTO":
                print(f"[INFO] Cycles AUTO: samples={scn.cycles.samples} < {gpu_sample_threshold}, using CPU.")
        scn.cycles.use_denoising = False
        scn.cycles.pixel_filter_type = 'BOX'
        if hasattr(scn.cycles, 'filter_width'):
            scn.cycles.filter_width = 0.01
    elif engine == "EEVEE_NEXT":
        try:
            scn.render.engine = 'BLENDER_EEVEE_NEXT'
        except TypeError:
            scn.render.engine = 'BLENDER_EEVEE'
    else:
        scn.render.engine = 'BLENDER_EEVEE'

    print(f"[INFO] Render engine: {scn.render.engine}, AA disabled: {cfg.get('disable_aa', True)}")


def setup_freestyle_outline(cfg):
    """Enable Freestyle silhouette/border outline (post-render line engine).

    Use when mesh has no inverted-hull outline mesh, or we want geometry-aware
    contour lines that DO NOT cover thin features (e.g. claws). Lines drawn only
    at silhouette/border edges, not on every front-facing polygon.
    """
    fs_cfg = cfg.get("freestyle_outline")
    if not fs_cfg:
        return
    scn = bpy.context.scene
    scn.render.use_freestyle = True
    vl = scn.view_layers[0]
    vl.use_freestyle = True
    fs = vl.freestyle_settings
    for ls in list(fs.linesets):
        fs.linesets.remove(ls)
    lineset = fs.linesets.new("outline")
    lineset.select_silhouette = fs_cfg.get("silhouette", True)
    lineset.select_border = fs_cfg.get("border", True)
    lineset.select_contour = fs_cfg.get("contour", False)
    lineset.select_crease = fs_cfg.get("crease", False)
    lineset.select_edge_mark = False
    linestyle = lineset.linestyle
    if linestyle is None:
        linestyle = bpy.data.linestyles.new("outline_style")
        lineset.linestyle = linestyle
    linestyle.color = tuple(fs_cfg.get("color", [0.0, 0.0, 0.0]))
    linestyle.thickness = fs_cfg.get("thickness", 2.0)
    print(f"[INFO] Freestyle outline enabled (thickness={linestyle.thickness}, silhouette={lineset.select_silhouette})")


def apply_solidify_outline(cfg):
    """Add Solidify modifier with backface-only black material to body meshes.

    Runtime equivalent of inverted-hull outline mesh. Use for single-mesh sources
    (ReArise, NewCentury) which lack a separate chr050_2-style outline mesh.
    """
    sol_cfg = cfg.get("auto_solidify")
    if not sol_cfg:
        return
    thickness = sol_cfg.get("thickness", 0.02) if isinstance(sol_cfg, dict) else 0.02
    skip = set(sol_cfg.get("skip_meshes", [])) if isinstance(sol_cfg, dict) else set()
    skip |= set(cfg.get("hide_meshes", []))
    skip |= set(cfg.get("outline_meshes", []))

    # Inverted-hull outline material: black emission, but ONLY on backfaces.
    # Front faces are transparent so the shell doesn't cover the body.
    # Combined with Solidify (offset=+1, flip_normals=True) this makes the
    # outset shell visible only along silhouette where the camera sees its
    # back (= flipped front, after flip_normals).
    black = bpy.data.materials.new("solidify_black")
    black.use_nodes = True
    bn = black.node_tree.nodes
    bl = black.node_tree.links
    bn.clear()
    geom = bn.new('ShaderNodeNewGeometry')
    emit = bn.new('ShaderNodeEmission')
    emit.inputs['Color'].default_value = (0, 0, 0, 1)
    transp = bn.new('ShaderNodeBsdfTransparent')
    mix_sh = bn.new('ShaderNodeMixShader')
    out = bn.new('ShaderNodeOutputMaterial')
    bl.new(geom.outputs['Backfacing'], mix_sh.inputs['Fac'])
    bl.new(transp.outputs['BSDF'], mix_sh.inputs[1])
    bl.new(emit.outputs['Emission'], mix_sh.inputs[2])
    bl.new(mix_sh.outputs['Shader'], out.inputs['Surface'])
    black.blend_method = 'CLIP'

    for obj in bpy.data.objects:
        if obj.type != 'MESH' or obj.name in skip or obj.hide_render:
            continue
        obj.data.materials.append(black)
        slot_idx = len(obj.data.materials) - 1
        mod = obj.modifiers.new("outline_solidify", 'SOLIDIFY')
        mod.thickness = thickness
        mod.offset = 1.0          # extrude outward (shell larger than body)
        mod.use_flip_normals = True
        mod.use_rim = False
        mod.material_offset = slot_idx
        mod.material_offset_rim = slot_idx
        print(f"[INFO] Solidify outline added to {obj.name} (thickness={thickness})")


def find_armature():
    for obj in bpy.data.objects:
        if obj.type == 'ARMATURE':
            return obj
    return None


def apply_action(armature, action_name):
    if armature is None:
        print("[WARN] No armature found.")
        return False
    action = bpy.data.actions.get(action_name)
    if action is None:
        print(f"[ERROR] Action '{action_name}' not found.")
        print(f"[INFO] Available actions:")
        for a in bpy.data.actions:
            print(f"  - {a.name}")
        return False
    if armature.animation_data is None:
        armature.animation_data_create()
    armature.animation_data.action = action
    # Blender 4.4+ layered actions: must bind a slot, otherwise armature stays in rest pose.
    # FBX2glTF emits multiple slots per action (e.g. OBJ_root for translation+scale,
    # OBGRP_mesh / OBGRP_joint for bone pose curves). Picking the first OBJECT slot
    # often grabs OBJ_root → only root translation animates, body stays static.
    # Fix: prefer slot whose channelbag actually contains pose.bones fcurves.
    slots = getattr(action, "slots", None)
    if slots and len(slots) > 0:
        def _slot_has_bone_curves(slot):
            try:
                for layer in getattr(action, "layers", []) or []:
                    for strip in getattr(layer, "strips", []):
                        cb = strip.channelbag(slot) if hasattr(strip, "channelbag") else None
                        if cb is None:
                            continue
                        for fc in getattr(cb, "fcurves", []):
                            if "pose.bones" in fc.data_path:
                                return True
            except Exception:
                pass
            return False

        # Prefer bone-curve slot, fall back to first OBJECT slot, then slots[0].
        slot = next((s for s in slots if s.target_id_type == 'OBJECT' and _slot_has_bone_curves(s)), None)
        if slot is None:
            slot = next((s for s in slots if s.target_id_type == 'OBJECT'), slots[0])
        try:
            armature.animation_data.action_slot = slot
            print(f"[INFO] Bound action slot: {slot.name_display if hasattr(slot,'name_display') else slot.identifier}")
        except Exception as e:
            print(f"[WARN] action_slot assignment failed: {e}")
    print(f"[INFO] Applied action: {action_name}")
    return True


def render_animation(anim_cfg, output_root, armature):
    output_dir = Path(output_root) / anim_cfg["name"]
    output_dir.mkdir(parents=True, exist_ok=True)

    if anim_cfg.get("action_name"):
        apply_action(armature, anim_cfg["action_name"])
    else:
        print("[INFO] No action specified — rendering bind/rest pose")
        if armature and armature.animation_data:
            armature.animation_data.action = None

    scn = bpy.context.scene
    start = anim_cfg["frame_start"]
    end = anim_cfg["frame_end"]
    step = anim_cfg.get("frame_step", 1)

    rendered_frames = []
    for i, frame in enumerate(range(start, end + 1, step)):
        scn.frame_set(frame)
        out_path = output_dir / f"frame_{i:02d}.png"
        scn.render.filepath = str(out_path)
        bpy.ops.render.render(write_still=True)
        rendered_frames.append(out_path)
        print(f"[INFO] Rendered frame {i} (blender frame {frame}) -> {out_path.name}")

    # Cyclic-anim dedup: Detect if frame_end is identical to frame_start
    # and drop the redundant last frame to ensure a clean loop.
    if len(rendered_frames) >= 3:
        import numpy as _np
        img_a = bpy.data.images.load(str(rendered_frames[0]))
        img_b = bpy.data.images.load(str(rendered_frames[-1]))
        try:
            n_a = len(img_a.pixels)
            n_b = len(img_b.pixels)
            if n_a == n_b:
                pa = _np.empty(n_a, dtype=_np.float32)
                pb = _np.empty(n_b, dtype=_np.float32)
                img_a.pixels.foreach_get(pa)
                img_b.pixels.foreach_get(pb)
                # flat RGBA[0..1]; compare RGB only
                pa_rgb = pa.reshape(-1, 4)[:, :3]
                pb_rgb = pb.reshape(-1, 4)[:, :3]
                mean_0_255 = float(_np.abs(pa_rgb - pb_rgb).mean()) * 255.0
                if mean_0_255 <= 1.0:
                    dropped = rendered_frames.pop()
                    dropped.unlink(missing_ok=True)
                    print(f"[INFO] Dropped trailing duplicate frame: {dropped.name} (mean diff {mean_0_255:.2f})")
        finally:
            bpy.data.images.remove(img_a)
            bpy.data.images.remove(img_b)

    return rendered_frames


def main():
    config_path = parse_args()
    print(f"[INFO] Loading config: {config_path}")
    with open(config_path) as f:
        cfg = json.load(f)

    clear_scene()
    print(f"[INFO] Importing model: {cfg['model_path']}")
    import_model(cfg["model_path"], fbx_opts=cfg.get("fbx_import_opts"))

    # Hide specified meshes (e.g., outline meshes that block view)
    hide_meshes = cfg.get("hide_meshes", [])
    for mesh_name in hide_meshes:
        if mesh_name in bpy.data.objects:
            bpy.data.objects[mesh_name].hide_render = True
            print(f"[INFO] Hidden from render: {mesh_name}")

    # Optional cleanup: drop placeholder meshes by name (e.g. FBX2glTF inserts an
    # "Icosphere" stub when the source has no skinned mesh root, which inflates
    # bbox + renders as gray blob).
    for placeholder in cfg.get("drop_objects", []):
        if placeholder in bpy.data.objects:
            obj = bpy.data.objects[placeholder]
            bpy.data.objects.remove(obj, do_unlink=True)
            print(f"[INFO] Dropped placeholder object: {placeholder}")

    # Optional uniform model scale (some glTF imports come at metric scale e.g.
    # FBX2glTF outputs centimeter-scale meshes — apply 100x via cfg).
    model_scale = cfg.get("model_scale_factor", 1.0)
    if model_scale != 1.0:
        for obj in bpy.data.objects:
            if obj.parent is None and obj.type in {'ARMATURE', 'MESH', 'EMPTY'}:
                obj.scale = (model_scale, model_scale, model_scale)
        bpy.context.view_layer.update()
        print(f"[INFO] Applied model scale factor: {model_scale}")

    # Apply optional model rotation (some FBX have weird default orientation)
    # Wrap roots under an Empty pivot so rotation survives armature animation keyframes.
    model_rotation = cfg.get("model_rotation_deg", [0, 0, 0])
    if model_rotation != [0, 0, 0]:
        from math import radians as rad
        pivot = bpy.data.objects.new("model_pivot", None)
        bpy.context.collection.objects.link(pivot)
        pivot.rotation_euler = (rad(model_rotation[0]), rad(model_rotation[1]), rad(model_rotation[2]))
        for obj in list(bpy.data.objects):
            if obj is pivot:
                continue
            if obj.parent is None and obj.type in {'ARMATURE', 'MESH', 'EMPTY'}:
                obj.parent = pivot
                obj.matrix_parent_inverse.identity()
        bpy.context.view_layer.update()
        print(f"[INFO] Applied model rotation: {model_rotation}° (via pivot Empty)")

    # Optional: rotate char around vertical (Z if world_up=z) to face desired direction.
    # Default: char faces -Y (which renders as facing +X when viewed from +X side cam).
    # Use this to flip char orientation if rendered side view shows wrong facing.
    facing_rot_deg = cfg.get("model_facing_z_rotation_deg", 0)
    if facing_rot_deg != 0:
        from math import radians as rad
        # Apply additional Z rotation to root objects on top of existing rotation
        for obj in bpy.data.objects:
            if obj.parent is None:
                # Compose with existing rotation
                cur = list(obj.rotation_euler)
                cur[2] += rad(facing_rot_deg)
                obj.rotation_euler = tuple(cur)
        bpy.context.view_layer.update()
        print(f"[INFO] Applied facing Z rotation: {facing_rot_deg}°")

    print(f"[INFO] Available actions ({len(bpy.data.actions)}):")
    for a in bpy.data.actions:
        print(f"  - {a.name}")

    # Apply shader variant (or fallback to legacy bind_texture)
    texture_path = cfg.get("texture_path")
    shader_variant = cfg.get("shader_variant")
    if texture_path and shader_variant:
        print(f"[INFO] Loading shader variant: {shader_variant}")
        shader_mod = load_shader_module(shader_variant)
        shader_opts = cfg.get("shader_opts", {})
        shader_mod.apply(
            texture_path,
            outline_meshes=cfg.get("outline_meshes", []),
            **shader_opts,
        )
    elif texture_path:
        # Legacy path
        bind_texture(
            texture_path,
            outline_meshes=cfg.get("outline_meshes", []),
            posterize_levels=cfg.get("posterize_levels", 4),
        )

    setup_lighting(cfg)
    setup_render(cfg)
    setup_freestyle_outline(cfg)
    apply_solidify_outline(cfg)
    # Allow shader variant to override engine settings
    if shader_variant and 'shader_mod' in dir():
        if hasattr(shader_mod, 'render_engine_settings'):
            shader_mod.render_engine_settings(bpy.context.scene)
            print(f"[INFO] Shader-specific render engine: {bpy.context.scene.render.engine}")

    output_root = cfg["output_root"]
    Path(output_root).mkdir(parents=True, exist_ok=True)
    armature = find_armature()

    # PRE-PASS: compute UNION bbox across all animations, so a single camera
    # frames every clip identically. Without this, each clip would get its own
    # ortho_scale + center → idle/skill/attack render at different scales and
    # at different feet_y on the atlas, breaking pixel-level alignment when the
    # game cross-fades or snaps between clips.
    union_bb_min = Vector((float('inf'),) * 3)
    union_bb_max = Vector((float('-inf'),) * 3)
    for anim in cfg["animations"]:
        if anim.get("action_name") and armature:
            apply_action(armature, anim["action_name"])
        anim_bb_min, anim_bb_max = get_animation_bbox(
            anim["frame_start"], anim["frame_end"], anim.get("frame_step", 1)
        )
        union_bb_min = Vector((min(union_bb_min[i], anim_bb_min[i]) for i in range(3)))
        union_bb_max = Vector((max(union_bb_max[i], anim_bb_max[i]) for i in range(3)))
        print(f"[INFO] Pre-pass bbox {anim['name']}: min={anim_bb_min}, max={anim_bb_max}")
    print(f"[INFO] UNION bbox (shared camera): min={union_bb_min}, max={union_bb_max}")

    # Setup camera ONCE with the union bbox — all anims now share framing.
    for obj in list(bpy.data.objects):
        if obj.type == 'CAMERA' or obj.name == 'cam_target':
            bpy.data.objects.remove(obj, do_unlink=True)
    setup_camera_auto(cfg, union_bb_min, union_bb_max)

    for anim in cfg["animations"]:
        print(f"\n[INFO] Rendering animation: {anim['name']}")
        if anim.get("action_name") and armature:
            apply_action(armature, anim["action_name"])
        render_animation(anim, output_root, armature)

    print("\n[INFO] Render complete.")


if __name__ == "__main__":
    main()
