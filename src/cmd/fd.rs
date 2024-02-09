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
