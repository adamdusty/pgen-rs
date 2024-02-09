use crate::template::{ProjectTemplate, RenderedTemplate};
use regex::Regex;
use std::{collections::HashMap, path::PathBuf};

fn sub(mut str: String, defs: &HashMap<String, String>) -> String {
    for (id, repl) in defs {
        let regex_str = format!(r"\{{@\s*(?P<var>{})\s*@\}}", id);
        let replace = repl;
        let re = Regex::new(&regex_str).expect("Error compiling regex");
        str = re.replace_all(&str, replace).to_string();
    }

    return str;
}

pub fn render_template(templ: ProjectTemplate, defs: &HashMap<String, String>) -> RenderedTemplate {
    return RenderedTemplate {
        directories: templ
            .directories
            .into_iter()
            .map(|s| PathBuf::from(sub(s, defs)))
            .collect(),
        files: templ
            .files
            .into_iter()
            .map(|f| (PathBuf::from(sub(f.path, defs)), sub(f.content, defs)))
            .collect(),
    };
}

#[cfg(test)]
mod tests {
    use super::render_template;
    use crate::template::{ProjectTemplate, TemplateFile};
    use std::{collections::HashMap, path::PathBuf};

    #[test]
    fn render_test() {
        let pt = ProjectTemplate {
            variables: vec!["namespace".to_string(), "project_name".to_string()],
            directories: vec!["docs".to_string(), "include".to_string(), "src".to_string()],
            files: vec![TemplateFile {
                path: r"include\{@ namespace @}\{@project_name@}.hpp".to_string(),
                content: "{@ namespace@}{@project_name @}".to_string(),
            }],
        };

        let defs: HashMap<String, String> = HashMap::from([
            ("namespace".to_string(), "passion".to_string()),
            ("project_name".to_string(), "fruit".to_string()),
        ]);

        let rendered = render_template(pt, &defs);

        assert!(rendered.directories.contains(&PathBuf::from("docs")));
        assert!(rendered.directories.contains(&PathBuf::from("include")));
        assert!(rendered.directories.contains(&PathBuf::from("src")));
        assert!(rendered
            .files
            .contains_key(&PathBuf::from(r"include\passion\fruit.hpp")));
        assert!(
            rendered.files[&PathBuf::from(r"include\passion\fruit.hpp")]
                == "passionfruit".to_string()
        );
    }
}
