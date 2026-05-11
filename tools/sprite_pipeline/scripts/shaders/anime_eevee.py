"""Anime cel-shading via standard Diffuse + ColorRamp (Cycles fallback).

Approximates classic anime style:
- Discrete bands via ColorRamp
- Hard shadow boundary (ColorRamp Constant interpolation)
- Slightly higher saturation than raw texture
- "Claw whitening" override to keep white features pure white
- "Inverted-hull" based outline
"""

import bpy
import os

def make_body_material(texture_path, shadow_levels=3, hue=0.5, saturation=1.15, value=1.05,
                       shadow_tint=(0.65, 0.65, 0.65), mid_tint=(1.05, 1.05, 1.05),
                       lit_tint=(1.40, 1.40, 1.40), posterize_levels=4):
    mat = bpy.data.materials.new(name="AnimeBody")
    mat.use_nodes = True
    nodes = mat.node_tree.nodes
    links = mat.node_tree.links
    nodes.clear()

    # Input: Texture
    tex = nodes.new('ShaderNodeTexImage')
    if os.path.exists(texture_path):
        try:
            tex.image = bpy.data.images.load(texture_path)
        except Exception:
            pass

    # HSV adjustment
    hsv = nodes.new('ShaderNodeHueSaturation')
    hsv.inputs['Hue'].default_value = hue
    hsv.inputs['Saturation'].default_value = saturation
    hsv.inputs['Value'].default_value = value
    links.new(tex.outputs['Color'], hsv.inputs['Color'])

    # Shading: Standard Diffuse (Cycles compatible)
    # Use texture intensity as a proxy for banding Fac
    ramp = nodes.new('ShaderNodeValToRGB')
    ramp.color_ramp.interpolation = 'CONSTANT'
    ramp.color_ramp.elements[0].position = 0.0
    ramp.color_ramp.elements[0].color = (*shadow_tint, 1.0)
    
    if shadow_levels > 2:
        if len(ramp.color_ramp.elements) < 3:
            ramp.color_ramp.elements.new(0.4)
        ramp.color_ramp.elements[1].position = 0.4
        ramp.color_ramp.elements[1].color = (*mid_tint, 1.0)
        ramp.color_ramp.elements[2].position = 0.7
        ramp.color_ramp.elements[2].color = (*lit_tint, 1.0)
    else:
        ramp.color_ramp.elements[1].position = 0.5
        ramp.color_ramp.elements[1].color = (*lit_tint, 1.0)

    rgb2bw = nodes.new('ShaderNodeRGBToBW')
    links.new(hsv.outputs['Color'], rgb2bw.inputs['Color'])
    links.new(rgb2bw.outputs['Val'], ramp.inputs['Fac'])

    mix = nodes.new('ShaderNodeMixRGB')
    mix.blend_type = 'MULTIPLY'
    mix.inputs['Fac'].default_value = 1.0
    links.new(hsv.outputs['Color'], mix.inputs['Color1'])
    links.new(ramp.outputs['Color'], mix.inputs['Color2'])

    # CLAW OVERRIDE LOGIC
    sep = nodes.new('ShaderNodeSeparateColor')
    sep.mode = 'RGB'
    links.new(tex.outputs['Color'], sep.inputs['Color'])
    
    min1 = nodes.new('ShaderNodeMath')
    min1.operation = 'MINIMUM'
    links.new(sep.outputs['Red'], min1.inputs[0])
    links.new(sep.outputs['Green'], min1.inputs[1])
    
    min2 = nodes.new('ShaderNodeMath')
    min2.operation = 'MINIMUM'
    links.new(min1.outputs[0], min2.inputs[0])
    links.new(sep.outputs['Blue'], min2.inputs[1])
    
    is_white = nodes.new('ShaderNodeMath')
    is_white.operation = 'GREATER_THAN'
    is_white.inputs[1].default_value = 0.85
    links.new(min2.outputs[0], is_white.inputs[0])

    final_color_mix = nodes.new('ShaderNodeMixRGB')
    links.new(is_white.outputs[0], final_color_mix.inputs['Fac'])
    links.new(mix.outputs['Color'], final_color_mix.inputs['Color1'])
    links.new(tex.outputs['Color'], final_color_mix.inputs['Color2'])

    # Output: Correct Alpha handling for Cycles Emission
    emit = nodes.new('ShaderNodeEmission')
    links.new(final_color_mix.outputs['Color'], emit.inputs['Color'])

    transp = nodes.new('ShaderNodeBsdfTransparent')

    mix_shader = nodes.new('ShaderNodeMixShader')
    links.new(tex.outputs['Alpha'], mix_shader.inputs['Fac'])
    links.new(transp.outputs['BSDF'], mix_shader.inputs[1])
    links.new(emit.outputs['Emission'], mix_shader.inputs[2])

    output = nodes.new('ShaderNodeOutputMaterial')
    links.new(mix_shader.outputs['Shader'], output.inputs['Surface'])

    try:
        mat.blend_method = 'HASHED'
    except Exception:
        pass

    return mat

def make_outline_material():
    mat = bpy.data.materials.new(name="AnimeOutline")
    mat.use_nodes = True
    nodes = mat.node_tree.nodes
    nodes.clear()

    emit = nodes.new('ShaderNodeEmission')
    emit.inputs['Color'].default_value = (0, 0, 0, 1)
    output = nodes.new('ShaderNodeOutputMaterial')
    mat.node_tree.links.new(emit.outputs['Emission'], output.inputs['Surface'])

    try:
        mat.blend_method = 'OPAQUE'
        mat.use_backface_culling = True
    except Exception:
        pass
    return mat
def apply(texture_path, body_meshes=None, outline_meshes=None, **opts):
    shadow_levels = opts.get('shadow_levels', 3)
    hue = opts.get('hue', 0.5)
    saturation = opts.get('saturation', 1.15)
    value = opts.get('value', 1.05)

    def _opt(k, default):
        val = opts.get(f'eevee_{k}', opts.get(k, default))
        return tuple(val) if isinstance(val, list) else val

    shadow_tint = _opt('shadow_tint', (0.65, 0.65, 0.65))
    mid_tint    = _opt('mid_tint',    (1.05, 1.05, 1.05))
    lit_tint    = _opt('lit_tint',    (1.40, 1.40, 1.40))
    posterize_levels = opts.get('posterize_levels', 4)

    body_mat = make_body_material(texture_path, shadow_levels, hue, saturation, value,
                                  shadow_tint, mid_tint, lit_tint, posterize_levels)
    outline_mat = make_outline_material()
    outline_meshes = outline_meshes or []

    for obj in bpy.data.objects:
        if obj.type != 'MESH':
            continue
        obj.data.materials.clear()
        if obj.name in outline_meshes:
            obj.data.materials.append(outline_mat)
            outline_thickness = opts.get('outline_thickness', 1.03)
            obj.scale = (outline_thickness, outline_thickness, outline_thickness)
            bpy.context.view_layer.objects.active = obj
            obj.select_set(True)
            bpy.ops.object.mode_set(mode='EDIT')
            bpy.ops.mesh.select_all(action='SELECT')
            bpy.ops.mesh.flip_normals()
            bpy.ops.object.mode_set(mode='OBJECT')
            obj.select_set(False)
        else:
            obj.data.materials.append(body_mat)

def render_engine_settings(scn):
    scn.render.engine = 'CYCLES'
    scn.cycles.samples = 1
    scn.cycles.use_denoising = False
    scn.cycles.pixel_filter_type = 'BOX'
    # Force transparent background
    scn.render.film_transparent = True
