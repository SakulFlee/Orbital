import os
import bpy
import bmesh

# Settings
entries_per_axis = 11
spacing = 2.5

# Clear
def clear():
    bpy.ops.object.select_all(action="DESELECT")
    bpy.ops.object.select_all(action="SELECT")
    bpy.ops.object.delete()
    bpy.data.batch_remove([o for o in bpy.data.objects if not o.users_scene])

def axis(offset_y=0, spheres=True):
    # Generate
    for z in range(0, entries_per_axis):
        for x in range(0, entries_per_axis):
            idx_name = "x" + str(x) + "_z" + str(z)

            # Create an empty mesh and the object.
            mesh = bpy.data.meshes.new("Sphere_" + idx_name)
            object = bpy.data.objects.new("Sphere_" + idx_name, mesh)

            mat = bpy.data.materials.new(name="Material_" + idx_name)
            mat.roughness = 0.0 if z == 0.0 else 1.0 if z == entries_per_axis - 1 else z / 10
            mat.metallic = 1.0 if x == 0.0 else 0.0 if x == entries_per_axis - 1 else 1.0 - x / 10
            object.data.materials.append(mat)

            # Add the object into the scene.
            bpy.context.collection.objects.link(object)

            # Select the newly created object
            bpy.context.view_layer.objects.active = object
            object.select_set(True)
            object.location.x = (x - entries_per_axis / 2) * spacing + spacing / 2
            object.location.z = (z - entries_per_axis / 2) * spacing + spacing / 2
            object.location.y = offset_y

            # Construct the bmesh sphere and assign it to the blender mesh.
            bm = bmesh.new()
            if spheres:
                bmesh.ops.create_uvsphere(bm, u_segments=16, v_segments=16, radius=1)
            else:
                bmesh.ops.create_cube(bm)
            bm.to_mesh(mesh)
            bm.free()

            bpy.ops.object.modifier_add(type="SUBSURF")
            bpy.ops.object.shade_smooth()

# Export
def export():
    cwd = os.getcwd()
    export_path = cwd + "/PBR_Grid.glb"
    bpy.ops.export_scene.gltf(export_format="GLB", filepath=export_path)

def main():
    print("### START ###")

    clear()
    axis(offset_y=5, spheres=True)
    axis(offset_y=-5, spheres=False)
    export()

    print("### FINISHED ###")

main()
