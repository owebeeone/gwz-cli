// CLI build-time provenance. Kept here so release builds are standalone.
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use std::{fs, process::Command};

fn git(root: &Path, args: &[&str]) -> Option<String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(args)
        .output()
        .ok()?;
    output
        .status
        .success()
        .then(|| String::from_utf8_lossy(&output.stdout).trim().to_owned())
}

fn files(root: &Path, relative: &Path, out: &mut Vec<PathBuf>) {
    let path = root.join(relative);
    let metadata = match fs::symlink_metadata(&path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return,
        Err(error) => panic!("cannot inspect build input {}: {error}", path.display()),
    };
    if metadata.is_dir() {
        for entry in fs::read_dir(path).expect("read build input directory") {
            let entry = entry.expect("read build input entry");
            let name = entry.file_name();
            if ["target", ".git", "__pycache__", ".venv", ".regen-venv"]
                .iter()
                .any(|excluded| name == *excluded)
            {
                continue;
            }
            files(root, &relative.join(name), out);
        }
    } else if metadata.is_file() || metadata.file_type().is_symlink() {
        out.push(relative.to_owned());
    }
}

pub fn source_digest(root: &Path) -> String {
    let mut inputs = Vec::new();
    for relative in [
        "Cargo.toml",
        "Cargo.lock",
        "build.rs",
        "src",
        "crates",
        "build_support",
        "protocol/gwz.taut.py",
    ] {
        files(root, Path::new(relative), &mut inputs);
    }
    inputs.sort_by_cached_key(|path| path.to_string_lossy().replace('\\', "/"));
    let mut hash = Sha256::new();
    for relative in inputs {
        let path = root.join(&relative);
        let key = relative.to_string_lossy().replace('\\', "/");
        let symlink = fs::symlink_metadata(&path)
            .unwrap()
            .file_type()
            .is_symlink();
        let bytes = if symlink {
            fs::read_link(&path)
                .unwrap()
                .to_string_lossy()
                .as_bytes()
                .to_vec()
        } else {
            fs::read(&path).expect("read build input")
        };
        hash.update((key.len() as u64).to_be_bytes());
        hash.update(key.as_bytes());
        hash.update([u8::from(symlink)]);
        hash.update((bytes.len() as u64).to_be_bytes());
        hash.update(&bytes);
    }
    format!("{:x}", hash.finalize())
}

pub fn emit() {
    let root = PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").unwrap());
    for relative in [
        "Cargo.toml",
        "Cargo.lock",
        "build.rs",
        "src",
        "crates",
        "build_support",
        "protocol/gwz.taut.py",
    ] {
        if root.join(relative).exists() {
            println!("cargo:rerun-if-changed={relative}");
        }
    }
    let own_repository = git(&root, &["rev-parse", "--show-toplevel"])
        .and_then(|path| fs::canonicalize(path).ok())
        == fs::canonicalize(&root).ok();
    let revision = if own_repository {
        git(&root, &["rev-parse", "HEAD"])
    } else {
        None
    };
    let dirty = if own_repository {
        let diff = git(&root, &["diff", "--name-only", "HEAD", "--"]);
        let untracked = git(&root, &["ls-files", "--others", "--exclude-standard"]);
        match (diff, untracked) {
            (Some(diff), Some(untracked)) => {
                if diff.is_empty() && untracked.is_empty() {
                    "false"
                } else {
                    "true"
                }
            }
            _ => "unknown",
        }
    } else {
        "unknown"
    };
    // Ref and index changes must invalidate provenance even with identical source bytes.
    if own_repository {
        for name in ["HEAD", "index", "packed-refs"] {
            if let Some(path) = git(&root, &["rev-parse", "--git-path", name]) {
                println!("cargo:rerun-if-changed={}", root.join(path).display());
            }
        }
        if let Some(reference) = git(&root, &["symbolic-ref", "-q", "HEAD"])
            && let Some(path) = git(&root, &["rev-parse", "--git-path", &reference])
        {
            println!("cargo:rerun-if-changed={}", root.join(path).display());
        }
    }
    println!(
        "cargo:rustc-env=GWZ_BUILD_PROVENANCE=revision={} dirty={} source-sha256={} build=cargo",
        revision.as_deref().unwrap_or("unavailable"),
        dirty,
        source_digest(&root)
    );
}

