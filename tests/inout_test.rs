extern crate aitios_asset;

use aitios_asset::obj;

#[test]
fn inout_test() {
    let entities = obj::load("tests/cube.obj")
        .unwrap();

    obj::save(
        entities.iter(),
        Some("tests/cube_with_mtl.obj"),
        Some("tests/cube_with_mtl.mtl")
    ).unwrap();

    obj::save(
        entities.iter(),
        Some("tests/cube_without_mtl.obj"),
        None
    ).unwrap();

    obj::save(
        entities.iter(),
        None,
        Some("tests/cube_without_obj.mtl")
    ).unwrap();
}
