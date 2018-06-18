use scene::{Entity, MaterialBuilder};
use std::path::PathBuf;
use std::fs::{File, canonicalize};
use std::io::Write;
use err::{AssetError, Result};
use pathdiff::diff_paths;
use std::borrow::Borrow;

/// Exports the given iterator over entities (or references, boxes, etc.) to the given OBJ/MTL files.
/// If one of the files should not be exported, leave it as None.
///
/// FIXME mtl output does only work when obj output also specified
pub fn save<I, E, P>(entities: I, obj_output_path: Option<P>, mtl_output_path: Option<P>) -> Result<()>
    where I : IntoIterator<Item = E>,
        E : Borrow<Entity>,
        P : Into<PathBuf>
{
    let obj_output_path = obj_output_path.map(|p| p.into());
    let mtl_output_path = mtl_output_path.map(|p| p.into());
    let mut mtl_file = None;
    let mut persisted_materials = Vec::new();

    if let Some(ref mtl_output_path) = mtl_output_path {
        let mut mtl = File::create(&mtl_output_path)
            .map_err(AssetError::from)?;

        // Write header
        mtl.write("# aitios procedurally weathered MTL file\n".as_bytes())?;
        mtl_file = Some(mtl);

        // TODO give materials unique names if properties are different but name is the same
    }

    if let Some(obj_output_path) = obj_output_path {
        let mut obj = File::create(&obj_output_path)?;
        let mut base = canonicalize(&obj_output_path)?;
        base.pop();

        // Make it a relative path
        let mtl_lib = if let Some(ref mtl) = mtl_output_path {
            let mtl = canonicalize(mtl)?;
            let relative_mtl_path = diff_paths(&mtl, &base)
                .ok_or_else(|| AssetError::InvalidData(
                    format!(
                        "Output path for MTL \"{mtl_path}\" cannot be expressed relative to directory that contains the OBJ \"{obj_path}\".",
                        mtl_path = mtl_output_path.as_ref().unwrap().to_str().unwrap(),
                        obj_path = obj_output_path.to_str().unwrap()
                    )
                ))?;

            let relative_mtl_path = relative_mtl_path.to_str()
                .ok_or(AssetError::InvalidData("Mtl path could not be converted to UTF-8 string.".to_string()))?
                .to_string();

            Some(relative_mtl_path)
        } else {
            None
        };

        // Write header
        obj.write("# aitios procedurally weathered OBJ file\n".as_bytes())?;
        if let Some(ref mtl_lib) = mtl_lib {
            obj.write("mtllib ".as_bytes())?;
            obj.write(mtl_lib.as_bytes())?;
            obj.write("\n".as_bytes())?;
        }
        obj.write("\n".as_bytes())?;

        let mut position_idx_base = 1_usize;
        let mut texcoord_idx_base = 1_usize;
        let mut normals_idx_base = 1_usize;

        for entity in entities.into_iter() {
            let entity = entity.borrow();

            let material = if persisted_materials.contains(&*entity.material) {
                // An exact same material with same maps can be shared,
                // no need for duplication
                (*entity.material).clone()
            } else if persisted_materials.iter().any(|m| m.name() == entity.material.name()) {
                // On a collision, where the name is the same but the maps are different,
                // make the name unique by appending the entity name
                // If that is not enough for uniqueness, try adding a numeric suffix until
                // the name is finally unique.
                // e.g. iron => iron-bunny => iron-bunny-2 => iron-bunny-3
                let unique_name_base = format!("{}-{}", entity.material.name(), entity.name);
                let mut unique_name = unique_name_base.clone();
                let mut suffix = 1;
                while persisted_materials.iter().any(|m| m.name() == &unique_name) {
                    suffix += 1; // start at two, since 1 is the one without suffix
                    unique_name = format!("{}-{}", unique_name_base, suffix);
                }
                MaterialBuilder::from(&*entity.material)
                    .name(unique_name)
                    .build()
            } else {
                (*entity.material).clone()
            };

            obj.write("o ".as_bytes())?;
            obj.write(entity.name.as_bytes())?;
            obj.write("\n".as_bytes())?;

            let position_lines = entity.mesh.positions.chunks(3)
                .map(|p| format!("v {} {} {}\n", p[0], p[1], p[2]));

            for position_line in position_lines {
                obj.write(position_line.as_bytes())?;
            }

            let texcoord_lines = entity.mesh.texcoords.chunks(2)
                .map(|t| format!("vt {} {}\n", t[0], t[1]));

            for texcoord_line in texcoord_lines {
                obj.write(texcoord_line.as_bytes())?;
            }

            let normal_lines = entity.mesh.normals.chunks(3)
                .map(|n| format!("vn {} {} {}\n", n[0], n[1], n[2]));

            for normal_line in normal_lines {
                obj.write(normal_line.as_bytes())?;
            }

            if mtl_lib.is_some() {
                obj.write(format!("usemtl {}\n", material.name()).as_bytes())?;
            }

            {
                let face_lines = entity.mesh.indices.chunks(3)
                    .map(|tri_indices| {
                        assert!(entity.mesh.texcoords.len() > 0);
                        match (!entity.mesh.positions.is_empty(), !entity.mesh.texcoords.is_empty(), !entity.mesh.normals.is_empty()) {
                            (true, true, true) => format!(
                                "f {}/{}/{} {}/{}/{} {}/{}/{}\n",
                                position_idx_base + (tri_indices[0] as usize), texcoord_idx_base + (tri_indices[0] as usize), normals_idx_base + (tri_indices[0] as usize),
                                position_idx_base + (tri_indices[1] as usize), texcoord_idx_base + (tri_indices[1] as usize), normals_idx_base + (tri_indices[1] as usize),
                                position_idx_base + (tri_indices[2] as usize), texcoord_idx_base + (tri_indices[2] as usize), normals_idx_base + (tri_indices[2] as usize)
                            ),
                            (true, true, false) => format!(
                                "f {}/{} {}/{} {}/{}\n",
                                position_idx_base + (tri_indices[0] as usize), texcoord_idx_base + (tri_indices[0] as usize),
                                position_idx_base + (tri_indices[1] as usize), texcoord_idx_base + (tri_indices[1] as usize),
                                position_idx_base + (tri_indices[2] as usize), texcoord_idx_base + (tri_indices[2] as usize)
                            ),
                            (true, false, true) => format!(
                                "f {}//{} {}//{} {}//{}\n",
                                position_idx_base + (tri_indices[0] as usize), normals_idx_base + (tri_indices[0] as usize),
                                position_idx_base + (tri_indices[1] as usize), normals_idx_base + (tri_indices[1] as usize),
                                position_idx_base + (tri_indices[2] as usize), normals_idx_base + (tri_indices[2] as usize)
                            ),
                            (true, false, false) => format!(
                                "f {} {} {}\n",
                                position_idx_base + (tri_indices[0] as usize),
                                position_idx_base + (tri_indices[1] as usize),
                                position_idx_base + (tri_indices[2] as usize)
                            ),
                            (false, _, _) => unimplemented!("OBJ cannot contain mesh that does not define positions")
                        }
                    });

                for face_line in face_lines {
                    obj.write(face_line.as_bytes())?;
                }
            }

            obj.write("\n".as_bytes())?;

            position_idx_base += entity.mesh.positions.len() / 3;
            texcoord_idx_base += entity.mesh.texcoords.len() / 2;
            normals_idx_base += entity.mesh.normals.len() / 3;

            if let Some(ref mut mtl) = mtl_file {
                if !persisted_materials.contains(&material) {
                    let mtl_maps = material.maps();
                    mtl.write(format!("\nnewmtl {}\n", material.name()).as_bytes())?;
                    //mtl.write(format!("Ns {}\n", material.shininess).as_bytes())?;
                    //mtl.write(format!("Ka {} {} {}\n", material.ambient[0], material.ambient[1], material.ambient[2]).as_bytes())?;
                    //mtl.write(format!("Kd {} {} {}\n", material.diffuse[0], material.diffuse[1], material.diffuse[2]).as_bytes())?;
                    //mtl.write(format!("Ks {} {} {}\n", material.specular[0], material.specular[1], material.specular[2]).as_bytes())?;
                    //mtl.write("Ke 0.000000 0.000000 0.000000\n".as_bytes())?;
                    //mtl.write("Ni 1.000000\n".as_bytes())?;
                    //mtl.write("d 1.000000\n".as_bytes())?;
                    mtl.write("illum 1\n".as_bytes())?;

                    for (map_mtl_key, map_path) in mtl_maps.iter() {
                        let map_path = canonicalize(map_path)?;
                        let map_path = diff_paths(&map_path, &base)
                            .expect(&format!("Path {:?} could not be expressed relative to OBJ parent directory {:?}", map_path, base));
                        let map_path = map_path.to_str()
                            .expect("Could not make UTF-8 string out of texture filename");
                        let map_line = format!("{key} {value}\n", key=map_mtl_key, value=map_path);
                        mtl.write(map_line.as_bytes())?;
                    }
                }
            }

            persisted_materials.push(material);
        }
    }

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    use obj::load;
    use std::rc::Rc;
    use std::fs::remove_file;

    #[test]
    fn test_material_name_collision_resolution() {
        let scene = load("tests/cube.obj")
            .unwrap();

        let cube = &scene[0];

        // Exact same material, name will not be duplicated and material shared
        let cube_clone = cube.clone();

        // Different roughness map, will receive entity name suffix
        let cube_roughness = Entity {
            material: Rc::new(MaterialBuilder::from(&*cube.material)
                // Using current directory as image file, otherwise saving would file since it
                // cannot find the map and thus cannot build a relative path
                .roughness_map(".")
                .build()),
            ..cube.clone()
        };

        // Different roughness map, will receive entity name suffix
        let cube_normal = Entity {
            material: Rc::new(MaterialBuilder::from(&*cube.material)
                .normal_map(".")
                .build()),
            ..cube.clone()
        };

        let obj_path = "aitios-test-obj-export.obj";
        let mtl_path = "aitios-test-obj-export.mtl";

        save(
            vec![cube, &cube_clone, &cube_roughness, &cube_normal],
            Some(obj_path), Some(mtl_path)
        ).unwrap();

        let loaded = load(obj_path)
            .unwrap();

        assert_eq!(
            2,
            loaded.iter().filter(|e| e.material.name() == "Material").count(),
            "Expecting two entities with material Material"
        );

        assert_eq!(
            1,
            loaded.iter().filter(|e| e.material.name() == "Material-Cube").count(),
            "Expecting two entities with material Material"
        );

        assert_eq!(
            1,
            loaded.iter().filter(|e| e.material.name() == "Material-Cube-2").count(),
            "Expecting two entities with material Material"
        );

        assert_eq!(
            0,
            loaded.iter().filter(|e| e.material.name() == "Material-Cube-3").count(),
            "Expecting two entities with material Material"
        );

        remove_file(obj_path).expect("Could not remove obj file created for test");
        remove_file(mtl_path).expect("Could not remove obj file created for test");
    }
}
