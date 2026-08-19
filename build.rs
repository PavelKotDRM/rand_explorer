use std::process::Command;

fn main() {
    let git = vergen_gitcl::Gitcl::all_git();
    let cargo = vergen_gitcl::Cargo::all_cargo();
    let rustc = vergen_gitcl::Rustc::all_rustc();

    if let Err(err) = vergen_gitcl::Emitter::default()
        .add_instructions(&git)
        .and_then(|emitter| emitter.add_instructions(&cargo))
        .and_then(|emitter| emitter.add_instructions(&rustc))
        .and_then(|emitter| emitter.emit())
    {
        panic!("failed to emit vergen metadata: {err}");
    }

    let output = Command::new("cargo")
        .args(["metadata", "--format-version", "1"])
        .output()
        .expect("failed to run cargo metadata during build");

    let metadata: serde_json::Value = serde_json::from_slice(&output.stdout)
        .expect("failed to parse cargo metadata JSON");

    let mut deps = metadata["packages"]
        .as_array()
        .map(|packages| {
            let mut entries = packages
                .iter()
                .filter_map(|pkg| {
                    let name = pkg["name"].as_str()?;
                    let version = pkg["version"].as_str()?;
                    Some(format!("{name}={version}"))
                })
                .collect::<Vec<_>>();
            entries.sort();
            entries
        })
        .unwrap_or_default();

    if deps.is_empty() {
        deps.push("unknown=unknown".to_string());
    }

    println!("cargo:rustc-env=RAND_EXPLORER_BUILD_DEPS={}", deps.join(" | "));
}
