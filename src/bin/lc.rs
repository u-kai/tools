use std::{
    fs::{read_dir, read_to_string},
    path::{Path, PathBuf},
    str::FromStr,
};

use clap::Parser;

fn main() {
    let cli = Cli::parse();
    let count = cli.count_line();
    println!("line is {}", count);
}

#[derive(Parser)]
struct Cli {
    #[clap(short, long)]
    extension: Extension,
    #[clap(short, long)]
    target: Option<TargetDir>,
    #[clap(short, long, value_delimiter = ',')]
    ignored: Option<Vec<IgnorePath>>,
}

impl Cli {
    fn count_line(&self) -> usize {
        let target = self
            .target
            .as_ref()
            .map(|t| t.all_file_path())
            .unwrap_or_else(|| TargetDir::default().all_file_path());

        target.iter().fold(0, |mut acc, f| {
            if self.extension.is(f)
                && !&self
                    .ignored
                    .as_ref()
                    .map(|i| i.iter().any(|i| i.do_ignore(f)))
                    .unwrap_or_default()
            {
                acc += read_to_string(f).unwrap().lines().count();
            };
            acc
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
struct TargetDir(PathBuf);

impl TargetDir {
    fn all_file_path(&self) -> Vec<PathBuf> {
        all_file_path(&self.0)
    }
}

impl Default for TargetDir {
    fn default() -> Self {
        let current_dir = std::env::current_dir().unwrap();
        Self(current_dir)
    }
}

impl FromStr for TargetDir {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(PathBuf::from(s)))
    }
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
    path: String,
}
impl IgnorePath {
    fn do_ignore(&self, f: &PathBuf) -> bool {
        f.file_name().map(|f| f.to_str()) == Some(Some(&self.path))
    }
}
impl FromStr for IgnorePath {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self {
            path: s.trim().to_string(),
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
    fn make_test_dir(dir_name: &str) {
        std::fs::create_dir_all(dir_name).unwrap();
        std::fs::write(format!("{}/main.rs", dir_name), "fn main() {}").unwrap();
        std::fs::write(format!("{}/lib.rs", dir_name), "fn lib() {}").unwrap();
        std::fs::write(format!("{}/main.py", dir_name), "def main(): pass").unwrap();
    }
    fn remove_dir(dir_name: &str) {
        let path: &Path = dir_name.as_ref();
        if path.exists() {
            std::fs::remove_dir_all(dir_name).unwrap();
        }
    }
    #[test]
    #[ignore = "watchで無限ループになる"]
    fn cliはディレクトリ内の指定した拡張子の行数をignoredを除いてカウントする() {
        let dir = "test_dir";
        remove_dir(dir);
        let cli = Cli::parse_from(&[
            "lc",
            "-e",
            "rs",
            "-t",
            dir,
            "-i",
            "main.rs, main.py, lib.rs",
        ]);
        make_test_dir(dir);
        let count = cli.count_line();
        remove_dir(dir);
        assert_eq!(count, 0);
    }
    #[test]
    #[ignore = "watchで無限ループになる"]
    fn cliはディレクトリ内の指定した拡張子の行数をカウントする() {
        let dir = "test_dir";
        remove_dir(dir);
        let cli = Cli::parse_from(&["lc", "-e", "rs", "-t", dir]);
        fn make_test_dir(dir_name: &str) {
            std::fs::create_dir_all(dir_name).unwrap();
            std::fs::write(format!("{}/main.rs", dir_name), "fn main() {}").unwrap();
            std::fs::write(format!("{}/lib.rs", dir_name), "fn lib() {}").unwrap();
            std::fs::write(format!("{}/main.py", dir_name), "def main(): pass").unwrap();
        }
        fn remove_dir(dir_name: &str) {
            let path: &Path = dir_name.as_ref();
            if path.exists() {
                std::fs::remove_dir_all(dir_name).unwrap();
            }
        }
        make_test_dir(dir);
        let count = cli.count_line();
        remove_dir(dir);
        assert_eq!(count, 2);
    }
    #[test]
    fn cliはディレクトリおよび拡張子および無視するパスの設定を指定できる() {
        let cli = Cli::parse_from(&[
            "lc",
            "-e",
            "rs",
            "-t",
            "src",
            "-i",
            "main.rs, main.py, lib.rs",
        ]);
        assert_eq!(cli.extension, Extension("rs".to_string()));
        assert_eq!(cli.target, Some(TargetDir(PathBuf::from("src"))));
        assert_eq!(
            cli.ignored,
            Some(vec![
                IgnorePath {
                    path: "main.rs".to_string()
                },
                IgnorePath {
                    path: "main.py".to_string()
                },
                IgnorePath {
                    path: "lib.rs".to_string()
                }
            ])
        );
    }
    #[test]
    fn target_dirはwindows_osも対応できる() {
        let target = TargetDir::from_str("C:\\Users\\user\\Desktop\\rust\\lc");
        assert_eq!(
            target,
            Ok(TargetDir(PathBuf::from(
                "C:\\Users\\user\\Desktop\\rust\\lc"
            )))
        );
    }
    #[test]
    fn target_defaultはカレントディレクトリを返す() {
        let target = TargetDir::default();
        assert_eq!(
            target,
            TargetDir(PathBuf::from(std::env::current_dir().unwrap()))
        );
    }
    #[test]
    fn targetは文字列から生成できる() {
        let target = TargetDir::from_str("src");
        assert_eq!(target, Ok(TargetDir(PathBuf::from("src"))));
    }
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
                path: "target".to_string()
            })
        );
    }
}
