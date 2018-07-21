use std::io;
use std::result;
use tobj;

pub type Result<T> = result::Result<T, AssetError>;

#[derive(Debug, Fail)]
pub enum AssetError {
    #[fail(display = "Asset import encountered error")]
    Load(#[cause] tobj::LoadError),
    #[fail(display = "Asset export encountered IO error")]
    Save(#[cause] io::Error),
    #[fail(display = "Invalid data during asset import/export: ")]
    InvalidData(String),
}

impl From<tobj::LoadError> for AssetError {
    fn from(err: tobj::LoadError) -> AssetError {
        AssetError::Load(err)
    }
}

impl From<io::Error> for AssetError {
    fn from(err: io::Error) -> AssetError {
        AssetError::Save(err)
    }
}
