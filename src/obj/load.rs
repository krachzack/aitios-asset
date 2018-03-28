use scene::{Entity, Material, MaterialBuilder, DeinterleavedIndexedMeshBuf};
use tobj;
use std::path::PathBuf;
use std::rc::Rc;
use std::iter::repeat;

/// Loads the entities stored in the OBJ file at the given path, also loading
/// associated materials from the MTL file referenced in the OBJ.
pub fn load<P : Into<PathBuf>>(from : P) -> Result<Vec<Entity>, tobj::LoadError> {
    let (models, materials) = tobj::load_obj(&from.into())?;

    let materials = convert_materials(materials);
    let models = convert_models(models, &materials);

    Ok(models)
}

fn convert_models<I>(models: I, materials: &Vec<Rc<Material>>) -> Vec<Entity>
    where I : IntoIterator<Item = tobj::Model>
{
    // Default material if object or group does not have a material
    let no_material = Rc::new(
        MaterialBuilder::new()
            .name("NoMaterial")
            .build()
    );

    models.into_iter()
        .map(|m| {
            Entity {
                name: m.name,
                // Reference same material for each with same index,
                // If no index, add a synthetic no_material with default properties.
                material: m.mesh.material_id
                    .map(|id| Rc::clone(&materials[id]))
                    .unwrap_or_else(|| Rc::clone(&no_material)),
                // DeinterleavedIndexedMeshBuf has format compatible to tobj,
                // just move the vectors and we are done
                mesh: tobj_mesh_to_aitios_mesh(m.mesh)
            }
        })
        .collect()
}

fn tobj_mesh_to_aitios_mesh(mesh: tobj::Mesh) -> DeinterleavedIndexedMeshBuf {
    let tobj::Mesh {
        positions,
        normals,
        mut texcoords,
        indices,
        ..
    } = mesh;

    if normals.len() == 0 {
        // If mesh does not define any normals, panic
        panic!("Tried to load OBJ file without normals");

        // TODO instead of panicking, calculate the normals
    }

    if texcoords.len() == 0 {
        // If no texcoords defined, assume them as (0.0, 0.0)
        let zero_texcoords = repeat(0.0)
            .take((positions.len() / 3) * 2);

        texcoords.extend(zero_texcoords);
    }

    DeinterleavedIndexedMeshBuf {
        positions, normals, texcoords, indices
    }
}

fn convert_materials<I>(materials: I) -> Vec<Rc<Material>>
    where I: IntoIterator<Item = tobj::Material>
{
    materials.into_iter()
        .map(tobj_to_aitios_mat)
        .collect()
}

fn tobj_to_aitios_mat(source_mat: tobj::Material) -> Rc<Material> {
    let mut mat = MaterialBuilder::new()
        .name(source_mat.name);

    if !source_mat.diffuse_texture.is_empty() {
        mat = mat.diffuse_color_map(source_mat.diffuse_texture);
    }

    if !source_mat.specular_texture.is_empty() {
        mat = mat.specular_color_map(source_mat.specular_texture);
    }

    if !source_mat.ambient_texture.is_empty() {
        mat = mat.ambient_color_map(source_mat.ambient_texture);
    }

    Rc::new(mat.build())
}
