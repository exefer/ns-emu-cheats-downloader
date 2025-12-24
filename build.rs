use std::fs;
use std::process::Command;

fn main() {
    let mut blps = Vec::new();

    if let Ok(entries) = fs::read_dir("data/resources/ui") {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("blp") {
                println!("cargo:rerun-if-changed={}", path.display());
                blps.push(path.to_string_lossy().to_string());
            }
        }
    }

    if !blps.is_empty() {
        let status = Command::new("blueprint-compiler")
            .arg("batch-compile")
            .arg("data/resources/ui")
            .arg("data/resources/ui")
            .args(&blps)
            .status()
            .expect("failed to execute blueprint-compiler");

        if !status.success() {
            panic!("blueprint-compiler batch-compile failed");
        }
    }

    glib_build_tools::compile_resources(
        &["data/resources"],
        "data/resources/resources.gresource.xml",
        "resources.gresource",
    );
}
