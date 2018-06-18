use scene::{Entity, Material, MaterialBuilder, DeinterleavedIndexedMeshBuf};
use tobj;
use std::path::PathBuf;
use std::rc::Rc;
use std::iter::repeat;
use err::Result;

/// Loads the entities stored in the OBJ file at the given path, also loading
/// associated materials from the MTL file referenced in the OBJ.
pub fn load<P : Into<PathBuf>>(from : P) -> Result<Vec<Entity>> {
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

fn tobj_mesh_to_aitios_mesh(mesh: tobj::Mesh) -> Rc<DeinterleavedIndexedMeshBuf> {
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

    Rc::new(
        DeinterleavedIndexedMeshBuf {
            positions, normals, texcoords, indices
        }
    )
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

    if !source_mat.ambient_texture.is_empty() {
        mat = mat.ambient_color_map(source_mat.ambient_texture);
    }

    if !source_mat.specular_texture.is_empty() {
        mat = mat.specular_color_map(source_mat.specular_texture);
    }

    let other = &source_mat.unknown_param;
    let bump = other.get("bump") // official name
        .or_else(|| other.get("map_bump")) // also seen this
        .or_else(|| other.get("bump_map")); // this one is just silly

    if let Some(bump) = bump {
        mat = mat.bump_map(bump);
    }

    let displacement = other.get("disp") // official name
        .or_else(|| other.get("map_disp")) // maybe some people also use this?
        .or_else(|| other.get("disp_map"));

    // While bump and displacement are standardized,
    // what follows isnt

    if let Some(displacement) = displacement {
        mat = mat.displacement_map(displacement);
    }

    // There is a built-in source_math.normal_texture in tobj.
    // However, it falsely uses map_Ns
    // Hence, use the unknown "norm", which is recommended for normals
    let normal = other.get("norm") // official, inofficial name
        .or_else(|| other.get("map_norm")) // maybe some people also use this?
        .or_else(|| other.get("map_normal"))
        .or_else(|| other.get("normal"))
        .or_else(|| other.get("normal_map"));

    if let Some(normal) = normal {
        mat = mat.normal_map(normal);
    }

    let roughness = other.get("map_Pr") // official, inofficial name
        .or_else(|| other.get("map_PR")) // maybe some people also use this?
        .or_else(|| other.get("map_pr"))
        .or_else(|| other.get("map_pR"))
        .or_else(|| other.get("Pr_map"));

    if let Some(roughness) = roughness {
        mat = mat.roughness_map(roughness);
    }

    let metallic = other.get("map_Pm") // official, inofficial name
        .or_else(|| other.get("map_PM")) // maybe some people also use this?
        .or_else(|| other.get("map_pm"))
        .or_else(|| other.get("map_pM"))
        .or_else(|| other.get("Pm_map"));

    if let Some(metallic) = metallic {
        mat = mat.metallic_map(metallic);
    }

    let sheen = other.get("map_Ps") // official, inofficial name
        .or_else(|| other.get("map_PS")) // maybe some people also use this?
        .or_else(|| other.get("map_ps"))
        .or_else(|| other.get("map_pS"))
        .or_else(|| other.get("Ps_map"));

    if let Some(sheen) = sheen {
        mat = mat.sheen_map(sheen);
    }

    let emissive = other.get("map_Ke") // official, inofficial name
        .or_else(|| other.get("map_KE")) // maybe some people also use this?
        .or_else(|| other.get("map_ke"))
        .or_else(|| other.get("map_kE"))
        .or_else(|| other.get("Ke_map"));

    if let Some(emissive) = emissive {
        mat = mat.emissive_map(emissive);
    }

    Rc::new(mat.build())
}
