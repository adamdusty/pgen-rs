use serde::{Deserialize, Serialize};
use std::{collections::HashMap, io::Read, path::PathBuf};

#[derive(Serialize, Deserialize, PartialEq, Eq)]
pub struct TemplateFile {
    pub path: String,
    pub content: String,
}

#[derive(Serialize, Deserialize, PartialEq, Eq)]
pub struct ProjectTemplate {
    pub variables: Vec<String>,
    pub directories: Vec<String>,
    pub files: Vec<TemplateFile>,
}
pub struct RenderedTemplate {
    pub directories: Vec<PathBuf>,
    pub files: HashMap<PathBuf, String>,
}

pub fn read_template(src: impl Read) -> Result<ProjectTemplate, impl std::error::Error> {
    return serde_yaml::from_reader(src);
}

#[cfg(test)]
mod tests {
    use crate::template::{ProjectTemplate, TemplateFile};

    use super::read_template;

    #[test]
    fn read() {
        let yaml = r"
        variables:
            - name
            - age
        directories:
            - src
            - include
            - docs
        files:
            - path: src/main.cpp
              content: hello world
        "
        .as_bytes();

        let pt = read_template(yaml).expect("Error reading template");

        assert!(pt.variables.iter().any(|s| s == "name"));
        assert!(pt.variables.iter().any(|s| s == "age"));

        assert!(pt.directories.iter().any(|s| s == "src"));
        assert!(pt.directories.iter().any(|s| s == "include"));
        assert!(pt.directories.iter().any(|s| s == "docs"));

        assert!(pt.files.contains(&TemplateFile {
            path: "src/main.cpp".to_string(),
            content: "hello world".to_string(),
        }));
    }

    #[test]
    fn deserialize() {
        let yaml = r"
        variables:
            - name
            - age
        directories:
            - src
            - include
            - docs
        files:
            - path: src/main.cpp
              content: hello world
        ";

        let pt: ProjectTemplate = serde_yaml::from_str(yaml).expect("Error deserializing");

        assert!(pt.variables.iter().any(|s| s == "name"));
        assert!(pt.variables.iter().any(|s| s == "age"));

        assert!(pt.directories.iter().any(|s| s == "src"));
        assert!(pt.directories.iter().any(|s| s == "include"));
        assert!(pt.directories.iter().any(|s| s == "docs"));

        assert!(pt.files.contains(&TemplateFile {
            path: "src/main.cpp".to_string(),
            content: "hello world".to_string(),
        }));
    }

    #[test]
    fn deserialize_missing_vars() {
        let yaml = r"
        variables:
        directories:
            - src
            - include
            - docs
        files:
            - path: src/main.cpp
              content: hello world
        ";

        let pt: ProjectTemplate = serde_yaml::from_str(yaml).expect("Error deserializing");

        assert!(pt.variables.is_empty());

        assert!(pt.directories.iter().any(|s| s == "src"));
        assert!(pt.directories.iter().any(|s| s == "include"));
        assert!(pt.directories.iter().any(|s| s == "docs"));

        assert!(pt.files.contains(&TemplateFile {
            path: "src/main.cpp".to_string(),
            content: "hello world".to_string(),
        }));
    }

    #[test]
    fn deserialize_missing_dirs() {
        let yaml = r"
        variables:
            - name
            - age
        directories:
        files:
            - path: src/main.cpp
              content: hello world
        ";

        let pt: ProjectTemplate = serde_yaml::from_str(yaml).expect("Error deserializing");

        assert!(pt.variables.iter().any(|s| s == "name"));
        assert!(pt.variables.iter().any(|s| s == "age"));

        assert!(pt.directories.is_empty());

        assert!(pt.files.contains(&TemplateFile {
            path: "src/main.cpp".to_string(),
            content: "hello world".to_string(),
        }));
    }

    #[test]
    fn deserialize_missing_vars_missing_dirs() {
        let yaml = r"
        variables:
        directories:
        files:
            - path: src/main.cpp
              content: hello world
        ";

        let pt: ProjectTemplate = serde_yaml::from_str(yaml).expect("Error deserializing");

        assert!(pt.variables.is_empty());

        assert!(pt.directories.is_empty());

        assert!(pt.files.contains(&TemplateFile {
            path: "src/main.cpp".to_string(),
            content: "hello world".to_string(),
        }));
    }

    #[test]
    fn deserialize_missing_all() {
        let yaml = r"
        variables:
        directories:
        files:
        ";

        let pt: ProjectTemplate = serde_yaml::from_str(yaml).expect("Error deserializing");

        assert!(pt.variables.is_empty());
        assert!(pt.directories.is_empty());
        assert!(pt.files.is_empty());
    }
}
