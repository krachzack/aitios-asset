
use scene::Entity;
use std::path::PathBuf;
use std::fs::{File, canonicalize};
use std::io::Write;
use err::{Error, Result};
use pathdiff::diff_paths;
use std::collections::HashSet;
use std::borrow::Borrow;

/// FIXME mtl output does only work when obj output also specified
pub fn save<I, E, P>(entities: I, obj_output_path: Option<P>, mtl_output_path: Option<P>) -> Result<()>
    where I : IntoIterator<Item = E>,
        E : Borrow<Entity>,
        P : Into<PathBuf>
{
    let obj_output_path = obj_output_path.map(|p| p.into());
    let mtl_output_path = mtl_output_path.map(|p| p.into());
    let mut mtl_file = None;
    let mut persisted_materials = HashSet::new();

    if let Some(ref mtl_output_path) = mtl_output_path {
        let mut mtl = File::create(&mtl_output_path)?;
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
            let relative_mtl_path = diff_paths(&mtl, &base);

            if let Some(relative_mtl_path) = relative_mtl_path {
                Some(String::from(relative_mtl_path.to_str().expect("Mtl path could not be converted to string")))
            } else {
                return Err(Error::Other(format!(
                    "mtl output path {:?} cannot be expressed relative to parent of OBJ file {:?}",
                    mtl, base
                )));
            }
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
            if let Some(ref mut mtl) = mtl_file {
                let entity = entity.borrow();
                let material_name = entity.material.name();
                let mtl_maps = entity.material.maps();

                if persisted_materials.insert(material_name) {
                    mtl.write(format!("\nnewmtl {}\n", material_name).as_bytes())?;
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
        }
    }

    Ok(())
}
