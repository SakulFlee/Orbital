import bpy
import os

output_dir = os.getcwd()

filename_without_extension = os.path.splitext(os.path.basename(bpy.data.filepath))[0]
print(filename_without_extension)

export_gltf = False
if filename_without_extension.startswith("Test"):
    export_gltf = True

# Export
output_file = output_dir + "/" + filename_without_extension + ".glb"
print("Output File: " + output_file)
bpy.ops.export_scene.gltf(export_format="GLB", filepath=output_file)

if export_gltf:
    output_file = output_dir + "/" + filename_without_extension + ".gltf"
    print("Output File: " + output_file)
    bpy.ops.export_scene.gltf(export_format="GLTF", filepath=output_file)
