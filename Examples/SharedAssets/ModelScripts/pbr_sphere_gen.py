import os
import bpy
import bmesh

# Settings
entries = 11
spacing = 2.5

# Clear
bpy.ops.object.select_all(action="DESELECT")
bpy.ops.object.select_all(action="SELECT")
bpy.ops.object.delete()
bpy.data.batch_remove([o for o in bpy.data.objects if not o.users_scene])

# Generate
for z in range(0, entries):
    for x in range(0, entries):
        idx_name = "x" + str(x) + "_z" + str(z)

        # Create an empty mesh and the object.
        mesh = bpy.data.meshes.new("Sphere_" + idx_name)
        basic_sphere = bpy.data.objects.new("Sphere_" + idx_name, mesh)

        mat = bpy.data.materials.new(name="Material_" + idx_name)
        mat.roughness = 0.0 if z == 0.0 else 1.0 if z == entries - 1 else z / 10
        mat.metallic = 1.0 if x == 0.0 else 0.0 if x == entries - 1 else 1.0 - x / 10
        basic_sphere.data.materials.append(mat)

        # Add the object into the scene.
        bpy.context.collection.objects.link(basic_sphere)

        # Select the newly created object
        bpy.context.view_layer.objects.active = basic_sphere
        basic_sphere.select_set(True)
        basic_sphere.location.x = (x - entries / 2) * spacing + spacing / 2
        basic_sphere.location.z = (z - entries / 2) * spacing + spacing / 2

        # Construct the bmesh sphere and assign it to the blender mesh.
        bm = bmesh.new()
        bmesh.ops.create_uvsphere(bm, u_segments=16, v_segments=16, radius=1)
        bm.to_mesh(mesh)
        bm.free()

        bpy.ops.object.modifier_add(type="SUBSURF")
        bpy.ops.object.shade_smooth()

# Export
cwd = os.getcwd()
export_path = cwd + "/PBR_Spheres.glb"
bpy.ops.export_scene.gltf(export_format="GLB", filepath=export_path)
