use crate::template::{ProjectTemplate, TemplateFile};
use std::{
    collections::HashSet,
    path::{Path, PathBuf},
};
use thiserror::Error;
use walkdir::WalkDir;

#[derive(Debug, Error)]
pub enum FromDirectoryError {
    #[error("File already exists at output")]
    OutputAlreadyExists(PathBuf),

    #[error("Output file could not be created")]
    OutputFileCreationError(PathBuf),

    #[error("Unable to read content from file")]
    FileReadError(PathBuf, String),

    #[error("Unable to convert path to string")]
    PathConversionError(PathBuf),

    #[error("Unable to read path in directory")]
    PathReadError(Option<PathBuf>),

    #[error("Error creating template file")]
    TemplateFileCreationError,

    #[error("Error generating yaml")]
    SerializationError,
}

pub fn fd(src: &Path, output: &Path, force: bool) -> Result<(), FromDirectoryError> {
    if output.exists() && !force {
        return Err(FromDirectoryError::OutputAlreadyExists(
            output.to_path_buf(),
        ));
    }

    let templ = generate_template(src)?;

    let Ok(out) = std::fs::File::create(output) else {
        return Err(FromDirectoryError::TemplateFileCreationError);
    };

    match serde_yaml::to_writer(out, &templ) {
        Ok(()) => Ok(()),
        Err(_) => Err(FromDirectoryError::SerializationError),
    }
}

fn generate_template(root: &Path) -> Result<ProjectTemplate, FromDirectoryError> {
    // Recursively collect all paths in src
    let paths = get_paths_from_root(root)?;

    // Read contents of paths that are files
    let dirs = paths
        .iter()
        .filter(|p| p.is_dir())
        .map(|p| {
            let Ok(rel_path) = p.strip_prefix(root) else {
                return Err(FromDirectoryError::PathConversionError(p.to_path_buf()));
            };

            let Some(str_path) = rel_path.to_str() else {
                return Err(FromDirectoryError::PathConversionError(p.to_path_buf()));
            };

            return Ok(str_path.to_string());
        })
        .collect::<Result<Vec<String>, FromDirectoryError>>()?;

    let files = paths
        .into_iter()
        .filter(|p| p.is_file())
        .map(|p| {
            let Ok(rel_path) = p.strip_prefix(root) else {
                return Err(FromDirectoryError::PathConversionError(p.to_path_buf()));
            };

            let Some(str_path) = rel_path.to_str() else {
                return Err(FromDirectoryError::PathConversionError(p.to_path_buf()));
            };

            let content = match std::fs::read_to_string(&p) {
                Ok(c) => c.replace("\r\n", "\n"),
                Err(e) => return Err(FromDirectoryError::FileReadError(p, e.to_string())),
            };

            return Ok(TemplateFile {
                path: str_path.to_string(),
                content: content,
            });
        })
        .collect::<Result<Vec<TemplateFile>, FromDirectoryError>>()?;

    // Iterate through dirs, file paths, and file contents to find vars
    let vars = get_template_vars(&dirs, &files);

    return Ok(ProjectTemplate {
        variables: Vec::from_iter(vars),
        directories: dirs,
        files: files,
    });
}

fn get_template_vars(dirs: &[String], files: &[TemplateFile]) -> HashSet<String> {
    return files
        .iter()
        .flat_map(|f| [&f.path, &f.content])
        .chain(dirs)
        .flat_map(|s| {
            return get_vars_from_string(s);
        })
        .collect::<HashSet<String>>();
}

fn get_vars_from_string(str: &str) -> HashSet<String> {
    let re = regex::Regex::new(r"\{@\s*(?P<var>\w+)\s*@\}").expect("Error compiling regex");

    return re
        .captures_iter(str)
        .map(|c| c["var"].to_string())
        .collect::<HashSet<String>>();
}

fn get_paths_from_root(root: &Path) -> Result<Vec<PathBuf>, FromDirectoryError> {
    return WalkDir::new(root)
        .into_iter()
        .map(|entry| {
            return match entry {
                Ok(e) => Ok(e.path().to_path_buf()),
                Err(e) => {
                    return Err(FromDirectoryError::PathReadError(
                        e.path().and_then(|p| Some(p.to_path_buf())),
                    ))
                }
            };
        })
        .collect();
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use crate::template::TemplateFile;

    use super::{get_template_vars, get_vars_from_string};

    #[test]
    fn test_get_vars_from_string() {
        let vars = get_vars_from_string("{@ namespace@}{@project_name @}");

        assert!(vars == HashSet::from(["namespace".to_string(), "project_name".to_string()]));
    }

    #[test]
    fn test_get_template_vars() {
        let dirs = vec![
            "{@ namespace @}/{@project_name@}".to_string(),
            "include".to_string(),
            "src".to_string(),
        ];
        let files = vec![TemplateFile {
            path: "test/{@project_name@}.cpp".to_string(),
            content: "{@namespace@}{project_name@}".to_string(),
        }];

        let actual = get_template_vars(&dirs, &files);

        assert!(actual == HashSet::from(["namespace".to_string(), "project_name".to_string()]));
    }
}
// use crate::{ProjectTemplate, TemplateFile};
// use fancy_regex::Regex;
// use std::{
//     collections::HashSet,
//     path::{Path, PathBuf},
// };
// use thiserror::Error;
// use walkdir::WalkDir;

// #[derive(Debug, Error)]
// pub enum FdError {
//     #[error("Error reading path")]
//     PathReadError(Option<PathBuf>),

//     #[error("Error converting Path to string")]
//     PathConversionError(PathBuf),

//     #[error("Missing metadata for path")]
//     MissingMetadata(PathBuf),

//     #[error("Error reading file")]
//     FileReadError(PathBuf),

//     #[error("Unsupported file type")]
//     UnsupportedFileTypeError(PathBuf),

//     #[error("Invalid regex")]
//     InvalidRegexError(String),

//     #[error("Error generating yaml from template")]
//     YamlGenerationError,

//     #[error("Error writing generated template")]
//     WriteError(PathBuf),
// }

// fn from_directory(root: &Path) -> Result<ProjectTemplate, FdError> {
//     let mut vars: HashSet<String> = HashSet::new();
//     let mut dirs: Vec<String> = Vec::new();
//     let mut files: Vec<TemplateFile> = Vec::new();

//     for entry in WalkDir::new(root) {
//         let entry = match entry {
//             Ok(d) => d,
//             Err(e) => match e.path() {
//                 Some(p) => return Err(FdError::PathReadError(Some(p.to_path_buf()))),
//                 None => return Err(FdError::PathReadError(None)),
//             },
//         };

//         let rel_path = match entry.path().strip_prefix(root) {
//             Ok(p) => p,
//             Err(_) => return Err(FdError::PathConversionError(entry.path().to_path_buf())),
//         };

//         let path_str = match rel_path.to_str() {
//             Some(s) => s,
//             None => return Err(FdError::PathConversionError(entry.path().to_path_buf())),
//         };

//         match entry.metadata() {
//             Err(_) => return Err(FdError::MissingMetadata(entry.path().to_path_buf())),
//             Ok(md) => {
//                 if md.is_dir() {
//                     if !path_str.is_empty() {
//                         dirs.push(String::from(path_str));
//                     }
//                 } else if md.is_file() {
//                     let content = match std::fs::read_to_string(entry.path()) {
//                         Ok(c) => c.replace("\r\n", "\n"),
//                         Err(_) => return Err(FdError::FileReadError(entry.path().to_path_buf())),
//                     };
//                     files.push(TemplateFile {
//                         path: String::from(path_str),
//                         content,
//                     });
//                 } else {
//                     return Err(FdError::UnsupportedFileTypeError(
//                         entry.path().to_path_buf(),
//                     ));
//                 }
//             }
//         }
//     }

//     let var_re = match Regex::new(r"\{@\s*(?P<var>\w+)\s*@}") {
//         Ok(r) => r,
//         Err(e) => return Err(FdError::InvalidRegexError(e.to_string())),
//     };

//     for dir in &dirs {
//         let dir_vars: HashSet<String> = var_re
//             .captures_iter(&dir)
//             .filter_map(|c| c.ok())
//             .map(|c| c["var"].to_string())
//             .collect();

//         vars.extend(dir_vars);
//     }

//     for file in &files {
//         let path_vars: HashSet<String> = var_re
//             .captures_iter(&file.path)
//             .filter_map(|c| c.ok())
//             .map(|c| c["var"].to_string())
//             .collect();

//         let content_vars: HashSet<String> = var_re
//             .captures_iter(&file.content)
//             .filter_map(|c| c.ok())
//             .map(|c| c["var"].to_string())
//             .collect();

//         vars.extend(path_vars);
//         vars.extend(content_vars);
//     }

//     return Ok(ProjectTemplate {
//         variables: Some(vars.into_iter().collect()),
//         directories: Some(dirs),
//         files,
//     });
// }

// pub fn fd(dir: &Path, out: &Path) -> Result<(), FdError> {
//     let templ = from_directory(dir)?;
//     let yaml = match serde_yaml::to_string(&templ) {
//         Ok(y) => y,
//         Err(_) => return Err(FdError::YamlGenerationError),
//     };

//     return match std::fs::write(out, &yaml) {
//         Ok(_) => Ok(()),
//         Err(_) => Err(FdError::WriteError(out.to_path_buf())),
//     };
// }
