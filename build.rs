use std::collections::HashMap;
use std::env;
use std::fs::{self, File};
use std::io::{BufRead, BufReader};
use std::path::Path;

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let registry_file = Path::new(&out_dir).join("translations_registry.txt");
    let dest_file = Path::new(&out_dir).join("translations.json");

    let mut translations = HashMap::new();

    if registry_file.exists() {
        let file = File::open(registry_file).unwrap();
        let reader = BufReader::new(file);

        for line in reader.lines() {
            if let Ok(line) = line {
                if let Some((key, text)) = line.split_once('|') {
                    translations.insert(key.to_string(), text.to_string());
                }
            }
        }
    }
    let json = serde_json::to_string_pretty(&translations).unwrap();
    fs::write(dest_file, json).unwrap();

    println!("cargo:rerun-if-changed=src");
}