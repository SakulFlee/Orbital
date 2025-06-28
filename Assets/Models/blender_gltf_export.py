import bpy

# Export
export_path = bpy.data.filepath.replace(".blend", ".glb")
bpy.ops.export_scene.gltf(export_format="GLB", filepath=export_path)
