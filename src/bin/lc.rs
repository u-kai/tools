use std::{
    fs::{read_dir, read_to_string},
    path::{Path, PathBuf},
};

use clap::Parser;

fn main() {
    let cli = Cli::parse();
    let target = match &cli.target {
        Some(target) => all_file_path(target),
        None => all_file_path("./"),
    };
    let count = target.iter().fold(0, |mut acc, f| {
        if f.extension().map(|f| f.to_str()) == Some(Some(&cli.extension))
            && !contain(f, &cli.ignored)
        {
            acc += read_to_string(f).unwrap().lines().count();
        };
        acc
    });
    println!("line is {}", count);
}
fn contain(f: &PathBuf, s: &Vec<String>) -> bool {
    let filename = f.to_str().unwrap_or_default();
    s.iter().any(|s| filename.contains(s.as_str()))
}

#[derive(Parser)]
struct Cli {
    extension: String,
    #[clap(short, long)]
    target: Option<String>,
    #[clap(short, long, value_delimiter = ',')]
    ignored: Vec<String>,
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
