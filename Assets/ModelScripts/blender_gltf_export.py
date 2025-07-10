import bpy
import os

output_dir = os.getcwd()

filename_without_extension = os.path.splitext(os.path.basename(bpy.data.filepath))[0]
print(filename_without_extension)

export_gltf = False
if filename_without_extension.startswith("Test"):
    export_gltf = True

# Export
def export(gltf_format, output_file_path):
    bpy.ops.export_scene.gltf(
        filepath=output_file_path,
        export_format=gltf_format,
        check_existing=True,
        export_cameras=True
    )

output_file = output_dir + "/" + filename_without_extension + ".glb"
print("Output File: " + output_file)
export("GLB", output_file)

if export_gltf:
    export("GLTF_SEPARATE", output_file)

print("### FINISHED ###")
