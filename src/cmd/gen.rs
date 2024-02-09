use crate::{
    render::render_template,
    template::{read_template, ProjectTemplate, RenderedTemplate},
};
use std::{
    collections::HashMap,
    fs::File,
    io::Write,
    path::{Path, PathBuf},
};
use thiserror::Error;

#[derive(Debug, Error)]
#[error("Error generating project")]
pub enum GenerationError {
    #[error("Root already exists")]
    RootExistsError(PathBuf),

    #[error("Error opening template file")]
    TemplateFileError(PathBuf),

    #[error("Unable to read template")]
    TemplateReadError(String),

    #[error("Error opening definitions file")]
    DefsFileError(PathBuf),

    #[error("Unable to read defs")]
    DefsReadError(String),

    #[error("Error rendering template")]
    TemplateRenderError,

    #[error("Error writing template to desitination")]
    TemplateWriteError(PathBuf),

    #[error("Error creating parent directory")]
    ParentDirectoryCreateError(PathBuf),
}

pub fn gen(root: &Path, templ_path: &Path, defs_path: &Path) -> Result<(), GenerationError> {
    // Check if root exists
    if root.exists() {
        return Err(GenerationError::RootExistsError(root.to_path_buf()));
    }

    // Read template at templ_path
    let Ok(templ_file) = File::open(templ_path) else {
        return Err(GenerationError::TemplateFileError(templ_path.to_path_buf()));
    };

    let templ = match read_template(templ_file) {
        Ok(t) => t,
        Err(e) => return Err(GenerationError::TemplateReadError(e.to_string())),
    };

    // Read defs at defs path
    let Ok(defs_file) = File::open(defs_path) else {
        return Err(GenerationError::DefsFileError(defs_path.to_path_buf()));
    };

    let defs = match serde_yaml::from_reader(defs_file) {
        Ok(d) => d,
        Err(e) => return Err(GenerationError::DefsReadError(e.to_string())),
    };

    // Generate project
    return generate_from_template(root, templ, &defs);
}

fn generate_from_template(
    root: &Path,
    templ: ProjectTemplate,
    defs: &HashMap<String, String>,
) -> Result<(), GenerationError> {
    // Render template
    let rendered = render_template(templ, defs);

    // Write to desitination
    if let Err(e) = write_rendered_template(root, &rendered) {
        eprintln!("Error writing rendered template");
        std::fs::remove_dir_all(root).expect("Error removing root directory");
        return Err(e);
    }

    return Ok(());
}
fn write_rendered_template(root: &Path, templ: &RenderedTemplate) -> Result<(), GenerationError> {
    for dir in &templ.directories {
        if let Err(_) = std::fs::create_dir_all(root.join(dir)) {
            return Err(GenerationError::TemplateWriteError(dir.to_path_buf()));
        }
    }

    for (p, c) in &templ.files {
        let path = root.join(p);
        if let Some(par) = path.parent() {
            if !par.exists() {
                if let Err(_) = std::fs::create_dir_all(par) {
                    return Err(GenerationError::ParentDirectoryCreateError(
                        par.to_path_buf(),
                    ));
                }
            }
        }

        let Ok(mut f) = std::fs::File::create(root.join(p)) else {
            return Err(GenerationError::TemplateWriteError(p.to_path_buf()));
        };

        if let Err(_) = write!(f, "{}", c) {
            return Err(GenerationError::TemplateWriteError(p.to_path_buf()));
        }
    }

    return Ok(());
}
