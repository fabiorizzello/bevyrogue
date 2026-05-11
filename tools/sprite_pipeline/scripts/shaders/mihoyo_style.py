"""MiHoYo/HSR-inspired multi-ramp cel-shading.

Clean, generic implementation:
- 3-band configurable shading (shadow, mid, highlight)
- Configurable Fresnel rim light
- No character-specific heuristics
- Standard Cycles compatibility
"""

import bpy
import os

def make_body_material(texture_path, hue=0.5, saturation=1.0, value=1.0,
                       shadow_tint=(0.7, 0.7, 0.7), mid_tint=(1.0, 1.0, 1.0),
                       lit_tint=(1.3, 1.3, 1.3), rim_tint=(0.8, 0.8, 0.8),
                       posterize_levels=0):
    mat = bpy.data.materials.new(name="mihoyo_body")
    mat.use_nodes = True
    nodes = mat.node_tree.nodes
    links = mat.node_tree.links
    nodes.clear()

    # Inputs
    tex = nodes.new('ShaderNodeTexImage')
    if os.path.exists(texture_path):
        tex.image = bpy.data.images.load(str(texture_path))
    tex.interpolation = 'Closest'

    hsv = nodes.new('ShaderNodeHueSaturation')
    hsv.inputs['Hue'].default_value = hue
    hsv.inputs['Saturation'].default_value = saturation
    hsv.inputs['Value'].default_value = value
    links.new(tex.outputs['Color'], hsv.inputs['Color'])

    # Shading logic
    geom = nodes.new('ShaderNodeNewGeometry')
    
    # Lambert dot
    light_dot = nodes.new('ShaderNodeVectorMath')
    light_dot.operation = 'DOT_PRODUCT'
    light_dot.inputs[1].default_value = (0.3, -0.4, 1.0)
    links.new(geom.outputs['Normal'], light_dot.inputs[0])

    # 3-band cel ramp
    ramp = nodes.new('ShaderNodeValToRGB')
    ramp.color_ramp.interpolation = 'CONSTANT'
    ramp.color_ramp.elements[0].position = 0.0
    ramp.color_ramp.elements[0].color = (*shadow_tint, 1.0)
    
    e1 = ramp.color_ramp.elements.new(0.15)
    e1.color = (*mid_tint, 1.0)

    e2 = ramp.color_ramp.elements.new(0.5)
    e2.color = (*lit_tint, 1.0)
    
    links.new(light_dot.outputs['Value'], ramp.inputs['Fac'])

    # Multiply
    mul = nodes.new('ShaderNodeMixRGB')
    mul.blend_type = 'MULTIPLY'
    mul.inputs['Fac'].default_value = 1.0
    links.new(hsv.outputs['Color'], mul.inputs[1])
    links.new(ramp.outputs['Color'], mul.inputs[2])

    # Rim light
    fresnel = nodes.new('ShaderNodeFresnel')
    fresnel.inputs['IOR'].default_value = 1.15
    
    rim_ramp = nodes.new('ShaderNodeValToRGB')
    rim_ramp.color_ramp.interpolation = 'CONSTANT'
    rim_ramp.color_ramp.elements[0].position = 0.0
    rim_ramp.color_ramp.elements[0].color = (0, 0, 0, 1)
    rim_ramp.color_ramp.elements[1].position = 0.95
    rim_ramp.color_ramp.elements[1].color = (*rim_tint, 1.0)
    links.new(fresnel.outputs['Fac'], rim_ramp.inputs['Fac'])

    add = nodes.new('ShaderNodeMixRGB')
    add.blend_type = 'ADD'
    add.inputs['Fac'].default_value = 0.1
    links.new(mul.outputs['Color'], add.inputs[1])
    links.new(rim_ramp.outputs['Color'], add.inputs[2])

    emit = nodes.new('ShaderNodeEmission')
    links.new(add.outputs['Color'], emit.inputs['Color'])
    
    transp = nodes.new('ShaderNodeBsdfTransparent')
    mix = nodes.new('ShaderNodeMixShader')
    links.new(tex.outputs['Alpha'], mix.inputs['Fac'])
    links.new(transp.outputs['BSDF'], mix.inputs[1])
    links.new(emit.outputs['Emission'], mix.inputs[2])
    
    output = nodes.new('ShaderNodeOutputMaterial')
    links.new(mix.outputs['Shader'], output.inputs['Surface'])
    
    mat.blend_method = 'CLIP'
    return mat


def make_outline_material(claw_white_height=0.0):
    mat = bpy.data.materials.new(name="mihoyo_outline")
    mat.use_nodes = True
    nodes = mat.node_tree.nodes
    links = mat.node_tree.links
    nodes.clear()

    geom = nodes.new('ShaderNodeNewGeometry')
    sep = nodes.new('ShaderNodeSeparateXYZ')
    links.new(geom.outputs['Position'], sep.inputs['Vector'])
    
    ramp = nodes.new('ShaderNodeValToRGB')
    ramp.color_ramp.interpolation = 'CONSTANT'
    ramp.color_ramp.elements[0].position = 0.0
    
    if claw_white_height > 0:
        ramp.color_ramp.elements[0].color = (1, 1, 1, 1)
        e1 = ramp.color_ramp.elements.new(claw_white_height)
        e1.color = (0, 0, 0, 1)
    else:
        ramp.color_ramp.elements[0].color = (0, 0, 0, 1)
        
    links.new(sep.outputs['Z'], ramp.inputs['Fac'])
    
    emit = nodes.new('ShaderNodeEmission')
    links.new(ramp.outputs['Color'], emit.inputs['Color'])
    
    transp = nodes.new('ShaderNodeBsdfTransparent')
    back = nodes.new('ShaderNodeNewGeometry')
    mix = nodes.new('ShaderNodeMixShader')
    links.new(back.outputs['Backfacing'], mix.inputs['Fac'])
    links.new(transp.outputs['BSDF'], mix.inputs[1])
    links.new(emit.outputs['Emission'], mix.inputs[2])
    
    output = nodes.new('ShaderNodeOutputMaterial')
    links.new(mix.outputs['Shader'], output.inputs['Surface'])
    
    mat.blend_method = 'CLIP'
    return mat


def apply(texture_path, body_meshes=None, outline_meshes=None, **opts):
    hue = opts.get('hue', 0.5)
    saturation = opts.get('saturation', 1.0)
    value = opts.get('value', 1.0)
    
    def _opt(k, default):
        val = opts.get(f'mihoyo_{k}', opts.get(k, default))
        return tuple(val) if isinstance(val, (list, tuple)) else val

    shadow_tint = _opt('shadow_tint', (0.7, 0.7, 0.7))
    mid_tint    = _opt('mid_tint',    (1.0, 1.0, 1.0))
    lit_tint    = _opt('lit_tint',    (1.3, 1.3, 1.3))
    rim_tint    = _opt('rim_tint',    (0.8, 0.8, 0.8))
    posterize_levels = opts.get('posterize_levels', 0)
    claw_white_height = _opt('claw_white_height', 0.0)

    body_mat = make_body_material(texture_path, hue, saturation, value,
                                  shadow_tint, mid_tint, lit_tint, rim_tint, posterize_levels)
    outline_mat = make_outline_material(claw_white_height=claw_white_height)
    
    outline_meshes = outline_meshes or []
    
    for obj in bpy.data.objects:
        if obj.type != 'MESH':
            continue
        obj.data.materials.clear()
        if obj.name in outline_meshes:
            obj.data.materials.append(outline_mat)
            obj.scale = (1.05, 1.05, 1.05)
            # Flip normals for inverted hull
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
    scn.render.film_transparent = True
