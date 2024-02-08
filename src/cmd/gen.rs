use crate::{render, ProjectTemplate, RenderedTemplate};
use std::{
    collections::HashMap,
    fs::{create_dir_all, write, File},
    path::Path,
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum GenError {
    #[error("Invalid rendered template.")]
    InvalidRenderedTemplateError,

    #[error("WriteError")]
    WriteError,

    #[error("IoError")]
    IoError(std::io::Error),

    #[error("SerializationError")]
    SerializationError(serde_yaml::Error),
}

fn read_defs(path: &Path) -> Result<HashMap<String, String>, GenError> {
    let def_file = match File::open(path) {
        Ok(f) => f,
        Err(e) => return Err(GenError::IoError(e)),
    };

    let defs: HashMap<String, String> = match serde_yaml::from_reader(def_file) {
        Ok(d) => d,
        Err(e) => return Err(GenError::SerializationError(e)),
    };

    return Ok(defs);
}

fn read_template(path: &Path) -> Result<ProjectTemplate, GenError> {
    let templ_file = match File::open(path) {
        Ok(f) => f,
        Err(e) => return Err(GenError::IoError(e)),
    };

    let templ: ProjectTemplate = match serde_yaml::from_reader(templ_file) {
        Ok(d) => d,
        Err(e) => return Err(GenError::SerializationError(e)),
    };

    return Ok(templ);
}

fn validate_render(rendered: &RenderedTemplate) -> bool {
    for dir in &rendered.directories {
        if dir.is_absolute() || dir.is_file() {
            return false;
        }
    }

    for (path, _) in &rendered.files {
        if path.is_absolute() || path.is_symlink() || path.exists() {
            return false;
        }
    }

    return true;
}

fn write_template(root: &Path, rendered: &RenderedTemplate) -> Result<(), GenError> {
    if !validate_render(rendered) {
        return Err(GenError::InvalidRenderedTemplateError);
    }

    match create_dir_all(root) {
        Ok(_) => {}
        Err(e) => return Err(GenError::IoError(e)),
    }

    for dir in &rendered.directories {
        let path = root.join(dir);
        match create_dir_all(path) {
            Ok(_) => {}
            Err(_) => return Err(GenError::WriteError),
        };
    }

    for (fpath, content) in &rendered.files {
        let path = root.join(fpath);
        if let Some(dir) = path.parent() {
            match create_dir_all(dir) {
                Ok(_) => {}
                Err(_) => return Err(GenError::WriteError),
            };
        } else {
            return Err(GenError::WriteError);
        }

        match write(path, content) {
            Ok(_) => {}
            Err(_) => return Err(GenError::WriteError),
        }
    }

    return Ok(());
}

pub fn generate(root: &Path, template: &Path, definitions: &Path) -> Result<(), GenError> {
    let defs = read_defs(definitions).expect("Error reading definitions file");
    let templ = read_template(template).expect("Error reading template file");

    let rendered = render(&templ, &defs);
    match write_template(&root, &rendered) {
        Err(e) => Err(e),
        Ok(()) => Ok(()),
    }
}
