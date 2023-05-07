use std::{
    fs::{read_dir, read_to_string},
    path::{Path, PathBuf},
    str::FromStr,
};

use clap::Parser;

fn main() {
    let cli = Cli::parse();
    let target = match &cli.target {
        Some(target) => all_file_path(target),
        None => all_file_path("./"),
    };
    let count = target.iter().fold(0, |mut acc, f| {
        if cli.extension.is(f) && !&cli.ignored.contain(f) {
            acc += read_to_string(f).unwrap().lines().count();
        };
        acc
    });
    println!("line is {}", count);
}

#[derive(Parser)]
struct Cli {
    #[clap(short, long)]
    extension: Extension,
    #[clap(short, long)]
    target: Option<String>,
    #[clap(short, long, value_delimiter = ',')]
    ignored: IgnorePath,
}

#[derive(Debug, Clone, PartialEq)]
struct Extension(String);
impl Extension {
    fn new(s: &str) -> Self {
        let mut chars = s.chars();
        if let Some(first) = chars.next() {
            if first == '.' {
                return Self(chars.collect::<String>());
            }
        }
        Self(s.to_string())
    }
    fn is(&self, f: &PathBuf) -> bool {
        f.extension().map(|f| f.to_str()) == Some(Some(&self.0))
    }
}
impl FromStr for Extension {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::new(s))
    }
}

#[derive(Debug, Clone, PartialEq)]
struct IgnorePath {
    paths: Vec<String>,
}
impl IgnorePath {
    fn contain(&self, f: &PathBuf) -> bool {
        let filename = f.to_str().unwrap_or_default();
        self.paths.iter().any(|s| filename.contains(s.as_str()))
    }
}
impl FromStr for IgnorePath {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self {
            paths: s.split(",").map(|s| s.trim_start().to_string()).collect(),
        })
    }
}

pub fn all_file_path(root_dir_path: impl AsRef<Path>) -> Vec<PathBuf> {
    match read_dir(root_dir_path.as_ref()) {
        Ok(root_dir) => root_dir
            .filter_map(|entry| entry.ok())
            .filter_map(|entry| match entry.file_type() {
                Ok(file_type) => Some((file_type, entry.path())),
                Err(_) => None,
            })
            .fold(Vec::new(), |mut acc, (file_type, path)| {
                if file_type.is_dir() {
                    let mut files = all_file_path(path);
                    acc.append(&mut files);
                    return acc;
                }
                acc.push(path);
                acc
            }),
        Err(e) => {
            println!("{}", e.to_string());
            panic!("not found path = {:?}", root_dir_path.as_ref())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn extensionは文字列から生成できる() {
        let extension = Extension::from_str("rs");
        assert_eq!(extension, Ok(Extension("rs".to_string())));
        let extension = Extension::from_str(".rs");
        assert_eq!(extension, Ok(Extension("rs".to_string())));
    }
    #[test]
    fn pathから拡張子を判別できる() {
        let extension = Extension::from_str("rs").unwrap();
        let path = PathBuf::from("src/main.rs");
        assert!(extension.is(&path));
        let path = PathBuf::from("src/main");
        assert!(!extension.is(&path));
    }
    #[test]
    fn ignore_pathは文字列から生成できる() {
        let ignore_path = IgnorePath::from_str("target");
        assert_eq!(
            ignore_path,
            Ok(IgnorePath {
                paths: vec!["target".to_string()]
            })
        );
        let ignore_path = IgnorePath::from_str("target, Cargo");
        assert_eq!(
            ignore_path,
            Ok(IgnorePath {
                paths: vec!["target".to_string(), "Cargo".to_string()]
            })
        );
    }
}
