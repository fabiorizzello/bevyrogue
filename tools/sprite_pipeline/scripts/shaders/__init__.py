"""Shader variant registry for cel-shading experiments.

Each shader module exports `apply(texture_path, body_meshes, outline_meshes, **opts)`.
Modules:
- basic_cel:      Posterize emission + inverted-hull outline (default)
- toon_bsdf:      Cycles ToonBSDF + lighting
- anime_eevee:    Eevee Diffuse + ColorRamp 2-3 levels + freestyle outline
- flat_emission:  Pure unshaded flat color
- threelevel:     3-level cel-shaded with rim light
- astropulse:     Use Astropulse Blender-to-Pixels compositor template
"""
