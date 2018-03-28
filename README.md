# aitios-asset
Provides input/output for 3D models and materials.

Currently, only OBJ is supported.

    use aitios_asset::obj;

    // Load entities from an OBJ as aitios_scene entities
    let entities = obj::load("tests/cube.obj")
        .unwrap();

    // Save them back to OBJ/MTL
    obj::save(
        entities.iter(),
        Some("tests/cube_with_mtl.obj"),
        Some("tests/cube_with_mtl.mtl")
    ).unwrap();
