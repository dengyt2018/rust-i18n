use std::io::Write;
use std::path::{Path, PathBuf};

use crate::{get_extension, merge_value, open_file_to_string, parse_string_to_serde_json};

pub fn get_merged_string<S: AsRef<str>>(files: &[S]) -> String {
    let all_locales = open_locales_files(&get_locales_files_path(files));
    let mut all_merged_value = serde_json::Value::default();

    for (content, path) in all_locales {
        let ext = get_extension(path);
        if let Ok(tmp) = parse_string_to_serde_json(&content, &ext) {
            merge_value(&mut all_merged_value, &tmp);
        }
    }

    serde_json::to_string_pretty(&all_merged_value).unwrap()
}

fn open_locales_files(entry: &[PathBuf]) -> Vec<(String, PathBuf)> {
    entry
        .iter()
        .map(|path| (open_file_to_string(path), path.clone()))
        .collect::<Vec<_>>()
}

/// Format input parameters as available paths
fn get_locales_files_path<S: AsRef<str>>(files: &[S]) -> Vec<PathBuf> {
    let all_support_ext = ["yml", "yaml", "json", "toml"];
    let file_path = Path::new(files.first().unwrap().as_ref());
    let ext = get_extension(file_path);
    let mut all_files: Vec<PathBuf> = Vec::new();

    if files.len() >= 2 {
        //
        // All supported formats are available as parameters
        // Example `cargo i18n -m 1.yml 2.yaml 3.json 4.toml`
        files.iter().for_each(|p| {
            let path = Path::new(p.as_ref());
            let ext = get_extension(path);

            if all_support_ext.iter().any(|e| ext.eq(e)) {
                all_files.push(path.into());
            }
        });
    } else {
        let file_parent_path = file_path.parent().unwrap();

        // If only one parameter, The `TODO` file will be default.
        all_support_ext.iter().for_each(|ex| {
            if ext.eq(ex) {
                let todo_file_name = format!("TODO.{}", ex);
                all_files.push(file_parent_path.join(Path::new(todo_file_name.as_str())));
            }
        });
        all_files.push(file_path.into());
    };
    all_files
}

/// Convert serde Value to the correct format
fn convert_serde_to_string(value: serde_json::Value, format: &str) -> String {
    match format {
        "json" => serde_json::to_string_pretty(&value).unwrap(),
        "yaml" | "yml" => {
            let text = serde_yaml::to_string(&value).unwrap();
            // Remove leading `---`
            text.trim_start_matches("---").trim_start().to_string()
        }
        "toml" => toml::to_string_pretty(&value).unwrap(),
        _ => unreachable!(),
    }
}

pub fn write_to_file(content: &str, filename: &str) {
    let file_path = Path::new(filename);
    let ext = get_extension(file_path);

    let mut file = std::fs::File::create(file_path).unwrap();

    let v = serde_json::from_str::<serde_json::Value>(content).unwrap();

    let result = convert_serde_to_string(v, &ext);

    writeln!(&mut file, "{}", result)
        .unwrap_or_else(|_| panic!("Unable to create file {}.", file_path.to_str().unwrap()));
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! assert_vec_eq {
        ($path_buff:expr, $path_string:expr) => {
            $path_buff.iter().for_each(|p| {
                let path = p.to_str().unwrap();
                assert!($path_string.contains(&path));
            });
        };
    }

    #[test]
    fn test_parse_locales_file_to_path() {
        let path_string = [
            "./locales/foo.yml",
            "./locales/foobar.yaml",
            "./locales/bar.json",
            "./locales/barfoo.toml",
        ];

        let paths = get_locales_files_path(&path_string);
        assert_eq!(paths.len(), 4, "There should be 4 paths formatted here");

        assert_vec_eq!(paths, path_string);
    }

    #[test]
    fn test_parse_one_file_to_path() {
        let mut one_arg = vec!["./locales/one.yml"];
        let paths = get_locales_files_path(&one_arg);
        let parent_path = Path::new(one_arg[0]).parent().unwrap();
        let todo_path = parent_path.join("TODO.yml").to_str().unwrap().to_string();
        one_arg.push(todo_path.as_str());

        assert_eq!(paths.len(), 2, "There should be 2 paths formatted here");
        assert_vec_eq!(paths, one_arg);
    }
}
