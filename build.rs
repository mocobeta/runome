use std::env;
use std::fs;
use std::path::Path;

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();

    let sysdic_src = Path::new(&manifest_dir).join("sysdic");
    let sysdic_dst = Path::new(&out_dir).join("sysdic");

    // Create destination directory
    if let Err(e) = fs::create_dir_all(&sysdic_dst) {
        panic!("Failed to create sysdic directory in OUT_DIR: {}", e);
    }

    // Copy all sysdic files to OUT_DIR
    if sysdic_src.exists() {
        copy_dir_contents(&sysdic_src, &sysdic_dst).unwrap_or_else(|e| {
            panic!("Failed to copy sysdic files: {}", e);
        });
    } else {
        panic!("sysdic directory not found at: {:?}", sysdic_src);
    }

    // Tell Cargo to rerun this build script if sysdic changes
    println!("cargo:rerun-if-changed=sysdic");

    // Set environment variable for runtime path lookup
    println!("cargo:rustc-env=SYSDIC_PATH={}", sysdic_dst.display());
}

fn copy_dir_contents(src: &Path, dst: &Path) -> std::io::Result<()> {
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if src_path.is_dir() {
            fs::create_dir_all(&dst_path)?;
            copy_dir_contents(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path)?;
        }
    }
    Ok(())
}
