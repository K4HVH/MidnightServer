use std::{env, fs, path::PathBuf};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out = PathBuf::from(env::var("OUT_DIR").unwrap());
    let descriptor_file = out.join("descriptors.bin");

    tonic_prost_build::configure()
        .build_server(true)
        .build_client(false)
        .file_descriptor_set_path(&descriptor_file)
        .compile_protos(
            &[
                "proto/midnight/midnight.proto",
                "proto/midnight/midnight_services.proto",
            ],
            &["proto/midnight/"],
        )?;

    let generated_dir = PathBuf::from("src/proto/generated");
    fs::create_dir_all(&generated_dir)?;

    fs::copy(&descriptor_file, generated_dir.join("descriptors.bin"))?;

    for entry in fs::read_dir(&out)? {
        let entry = entry?;
        let path = entry.path();
        if let Some(name) = path.file_name()
            && name.to_string_lossy().ends_with(".rs")
        {
            fs::copy(&path, generated_dir.join(name))?;
        }
    }

    println!("cargo:rerun-if-changed=proto/");
    println!("cargo:rerun-if-changed=build.rs");

    Ok(())
}
