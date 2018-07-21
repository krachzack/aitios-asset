use err::{AssetError::*, Result};
use scene::{DeinterleavedIndexedMeshBuf, Entity, Material, MaterialBuilder};
use std::iter::repeat;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use tobj;

/// Loads the entities stored in the OBJ file at the given path, also loading
/// associated materials from the MTL file referenced in the OBJ.
pub fn load<P: Into<PathBuf>>(from: P) -> Result<Vec<Entity>> {
    let from = from.into();
    let (models, materials) = tobj::load_obj(&from)?;

    let materials = convert_materials(materials, &from)?;
    let models = convert_models(models, &materials);

    Ok(models)
}

fn convert_models<I>(models: I, materials: &Vec<Rc<Material>>) -> Vec<Entity>
where
    I: IntoIterator<Item = tobj::Model>,
{
    // Default material if object or group does not have a material
    let no_material = Rc::new(MaterialBuilder::new().name("NoMaterial").build());

    models
        .into_iter()
        .map(|m| {
            Entity {
                name: m.name,
                // Reference same material for each with same index,
                // If no index, add a synthetic no_material with default properties.
                material: m
                    .mesh
                    .material_id
                    .map(|id| Rc::clone(&materials[id]))
                    .unwrap_or_else(|| Rc::clone(&no_material)),
                // DeinterleavedIndexedMeshBuf has format compatible to tobj,
                // just move the vectors and we are done
                mesh: tobj_mesh_to_aitios_mesh(m.mesh),
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
        let zero_texcoords = repeat(0.0).take((positions.len() / 3) * 2);

        texcoords.extend(zero_texcoords);
    }

    Rc::new(DeinterleavedIndexedMeshBuf {
        positions,
        normals,
        texcoords,
        indices,
    })
}

fn convert_materials<I>(materials: I, obj_file: &Path) -> Result<Vec<Rc<Material>>>
where
    I: IntoIterator<Item = tobj::Material>,
{
    let obj_parent = obj_file.parent().unwrap_or_else(|| &Path::new("."));

    materials
        .into_iter()
        .map(|m| tobj_to_aitios_mat(m, obj_parent))
        .collect()
}

fn resolve(path: &str, base: &Path) -> Result<PathBuf> {
    let mut path: &Path = path.as_ref();

    if path.as_os_str().is_empty() {
        return Err(InvalidData(
            "OBJ/MTL reference an empty string where a path to an MTL or texture file shold be"
                .to_string(),
        ));
    }

    match path.canonicalize() {
        // If could be canonicalized, it must exist, return it
        Ok(path) => Ok(path),
        Err(_) => {
            // Try stripping first path component and interpreting as relative
            // instead of absolute
            if path.is_absolute() {
                path =
                    path.strip_prefix(
                        path.iter().next().unwrap(), // unwrap safe since is_empty() returned false
                    ).unwrap();
            }

            let mut relative_to_base = PathBuf::from(base);
            relative_to_base.push(path);

            match relative_to_base.canonicalize() {
                Ok(path) => Ok(path),
                Err(_) => Err(InvalidData(format!(
                    "OBJ/MTL referenced non-existing file: {:?}",
                    path
                ))),
            }
        }
    }
}

fn tobj_to_aitios_mat(source_mat: tobj::Material, base_dir: &Path) -> Result<Rc<Material>> {
    let mut mat = MaterialBuilder::new().name(source_mat.name);

    if !source_mat.diffuse_texture.is_empty() {
        mat = mat.diffuse_color_map(resolve(&source_mat.diffuse_texture, base_dir)?);
    }

    if !source_mat.ambient_texture.is_empty() {
        mat = mat.ambient_color_map(resolve(&source_mat.ambient_texture, base_dir)?);
    }

    if !source_mat.specular_texture.is_empty() {
        mat = mat.specular_color_map(resolve(&source_mat.specular_texture, base_dir)?);
    }

    let other = &source_mat.unknown_param;
    let bump = other.get("bump") // official name
        .or_else(|| other.get("map_bump")) // also seen this
        .or_else(|| other.get("bump_map")); // this one is just silly

    if let Some(bump) = bump {
        mat = mat.bump_map(resolve(&bump, base_dir)?);
    }

    let displacement = other.get("disp") // official name
        .or_else(|| other.get("map_disp")) // maybe some people also use this?
        .or_else(|| other.get("disp_map"));

    // While bump and displacement are standardized,
    // what follows isnt

    if let Some(displacement) = displacement {
        mat = mat.displacement_map(resolve(&displacement, base_dir)?);
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
        mat = mat.normal_map(resolve(&normal, base_dir)?);
    }

    let roughness = other.get("map_Pr") // official, inofficial name
        .or_else(|| other.get("map_PR")) // maybe some people also use this?
        .or_else(|| other.get("map_pr"))
        .or_else(|| other.get("map_pR"))
        .or_else(|| other.get("Pr_map"));

    if let Some(roughness) = roughness {
        mat = mat.roughness_map(resolve(&roughness, base_dir)?);
    }

    let metallic = other.get("map_Pm") // official, inofficial name
        .or_else(|| other.get("map_PM")) // maybe some people also use this?
        .or_else(|| other.get("map_pm"))
        .or_else(|| other.get("map_pM"))
        .or_else(|| other.get("Pm_map"));

    if let Some(metallic) = metallic {
        mat = mat.metallic_map(resolve(&metallic, base_dir)?);
    }

    let sheen = other.get("map_Ps") // official, inofficial name
        .or_else(|| other.get("map_PS")) // maybe some people also use this?
        .or_else(|| other.get("map_ps"))
        .or_else(|| other.get("map_pS"))
        .or_else(|| other.get("Ps_map"));

    if let Some(sheen) = sheen {
        mat = mat.sheen_map(resolve(&sheen, base_dir)?);
    }

    let emissive = other.get("map_Ke") // official, inofficial name
        .or_else(|| other.get("map_KE")) // maybe some people also use this?
        .or_else(|| other.get("map_ke"))
        .or_else(|| other.get("map_kE"))
        .or_else(|| other.get("Ke_map"));

    if let Some(emissive) = emissive {
        mat = mat.emissive_map(resolve(&emissive, base_dir)?);
    }

    Ok(Rc::new(mat.build()))
}
