mod error;

use std::path::Path;

use error::{Error, Result};

fn exists<P>(path: P) -> Result<()>
where
    P: AsRef<Path>,
{
    if !path.as_ref().exists() {
        Err(Error::NotFound)
    } else {
        Ok(())
    }
}

pub fn load_module_from_path<P>(path: P) -> Result<String>
where
    P: AsRef<Path>,
{
    exists(&path)?;
    let content = std::fs::read_to_string(&path)?;
    Ok(content)
}
