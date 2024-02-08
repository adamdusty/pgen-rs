pub mod cmd;
use core::panic;
use fancy_regex::Regex;
use leon::Template;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::PathBuf};

#[derive(Serialize, Deserialize, PartialEq, Eq)]
pub struct TemplateFile {
    pub path: String,
    pub content: String,
}

#[derive(Serialize, Deserialize, PartialEq, Eq)]
pub struct ProjectTemplate {
    pub variables: Option<Vec<String>>,
    pub directories: Option<Vec<String>>,
    pub files: Vec<TemplateFile>,
}
pub struct RenderedTemplate {
    pub directories: Vec<PathBuf>,
    pub files: HashMap<PathBuf, String>,
}
#[derive(Debug, Clone)]
pub struct ProjectWriteError;

#[derive(Debug, Clone)]
pub struct TemplateLoadError;

pub fn render(templ: &ProjectTemplate, defs: &HashMap<String, String>) -> RenderedTemplate {
    let lbracket_re = Regex::new(r"(?P<left>\{(?!@))").expect("Error compiling regex");
    let rbracket_re = Regex::new(r"(?P<right>(?<!@)})").expect("Error compiling regex");

    let dirs = match &templ.directories {
        Some(dirs) => dirs
            .iter()
            .map(|d| {
                let mut dir = lbracket_re.replace_all(d, r"\{").to_string();
                dir = rbracket_re.replace_all(&dir, r"\}").to_string();
                return dir.replace("@", "");
            })
            .collect(),
        None => Vec::new(),
    };

    let files: Vec<TemplateFile> = templ
        .files
        .iter()
        .map(|tf| {
            let mut path = lbracket_re.replace_all(&tf.path, r"\{").to_string();
            path = rbracket_re.replace_all(&path, r"\}").to_string();
            path = path.replace("@", "");

            let mut content = lbracket_re.replace_all(&tf.content, r"\{").to_string();
            content = rbracket_re.replace_all(&content, r"\}").to_string();
            content = content.replace("@", "");

            return TemplateFile { path, content };
        })
        .collect();

    // Render dirs
    let rendered_directories: Vec<PathBuf> = dirs
        .iter()
        .map(|d| {
            match Template::parse(&d) {
                Ok(t) => return PathBuf::from(t.render(defs).expect("Error rendering dir")),
                Err(e) => {
                    eprintln!("Error parsing template: {:?}", e);
                    panic!("Invalid directory template: {:?}", d);
                }
            };
            // if let Ok(template) = Template::parse(&d) {
            //     //.expect("Invalid directory");
            //     return PathBuf::from(template.render(defs).expect("Error rendering dir"));
            // } else {
            //     panic!("Invalid directory template: {:?}", d);
            // }
        })
        .collect();

    // Render file paths and content
    let rendered_files: HashMap<PathBuf, String> = files
        .iter()
        .map(|tf| {
            let path_template = Template::parse(&tf.path).expect("Invalid file path");
            let content_template = Template::parse(&tf.content).expect("Invalid file content");
            return (
                PathBuf::from(
                    path_template
                        .render(defs)
                        .expect("Error rendering file path."),
                ),
                content_template
                    .render(defs)
                    .expect("Error rendering file content."),
            );
        })
        .collect();

    return RenderedTemplate {
        directories: rendered_directories,
        files: rendered_files,
    };
}

#[cfg(test)]
mod tests {
    use crate::{render, ProjectTemplate, TemplateFile};
    use std::{collections::HashMap, path::PathBuf};

    #[test]
    fn test_serial() {
        let templ = serde_json::json!({
            "variables": [
                "project_name",
                "namespace"
            ],
            "directories": [
                "src",
                "docs"
            ],
            "files": [
                {
                    "path": "src/main.cpp",
                    "content": "hello world"
                }
            ]
        });

        let result: super::ProjectTemplate = serde_json::from_value(templ).unwrap();

        assert!(result.variables.is_some());
        assert!(result.directories.is_some());

        assert!(
            result.files[0]
                == super::TemplateFile {
                    path: String::from("src/main.cpp"),
                    content: String::from("hello world")
                }
        );
    }

    #[test]
    fn test_serial_no_vars() {
        let templ = serde_json::json!({
            "directories": [
                "src",
                "docs"
            ],
            "files": [
                {
                    "path": "src/main.cpp",
                    "content": "hello world"
                }
            ]
        });

        let result: super::ProjectTemplate = serde_json::from_value(templ).unwrap();

        assert!(result.variables.is_none());
        assert!(result.directories.is_some());
        assert!(result.directories.unwrap().contains(&String::from("src")));
        assert!(
            result.files[0]
                == super::TemplateFile {
                    path: String::from("src/main.cpp"),
                    content: String::from("hello world")
                }
        );
    }

    #[test]
    fn test_serial_no_dirs_no_vars() {
        let templ = serde_json::json!({
            "files": [
                {
                    "path": "src/main.cpp",
                    "content": "hello world"
                }
            ]
        });

        let result: super::ProjectTemplate = serde_json::from_value(templ).unwrap();

        assert!(result.variables.is_none());
        assert!(result.directories.is_none());
        assert!(
            result.files[0]
                == super::TemplateFile {
                    path: String::from("src/main.cpp"),
                    content: String::from("hello world")
                }
        );
    }

    #[test]
    fn render_template() {
        let tmpl = ProjectTemplate {
            variables: Some(vec!["project_name".to_string(), "namespace".to_string()]),
            directories: Some(vec![
                "src".to_string(),
                "tests_{@ project_name @}".to_string(),
            ]),
            files: vec![
                TemplateFile {
                    path: "src/main.cpp".to_string(),
                    content: r"namespace {@namespace@} { auto main() -> int { return 0; }"
                        .to_string(),
                },
                TemplateFile {
                    path: "tests_{@project_name@}/test.cpp".to_string(),
                    content: r"{@namespace@} {@project_name@}".to_string(),
                },
            ],
        };

        let defs: HashMap<String, String> = HashMap::from([
            ("namespace".to_string(), "test_namespace".to_string()),
            ("project_name".to_string(), "theproj".to_string()),
        ]);

        let rendered = render(&tmpl, &defs);

        assert!(!rendered.directories.is_empty());
        assert!(!rendered.files.is_empty());

        assert!(rendered.directories.contains(&PathBuf::from("src")));
        assert!(rendered
            .directories
            .contains(&PathBuf::from("tests_theproj")));

        assert!(rendered.files.contains_key(&PathBuf::from("src/main.cpp")));
        assert!(rendered
            .files
            .contains_key(&PathBuf::from("tests_theproj/test.cpp")));

        assert_eq!(
            rendered.files.get(&PathBuf::from("src/main.cpp")).unwrap(),
            &"namespace test_namespace { auto main() -> int { return 0; }".to_string()
        );

        assert_eq!(
            rendered
                .files
                .get(&PathBuf::from("tests_theproj/test.cpp"))
                .unwrap(),
            &"test_namespace theproj".to_string()
        );
    }
}
