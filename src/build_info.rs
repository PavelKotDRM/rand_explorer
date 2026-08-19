#[derive(Clone, Debug)]
pub struct BuildInfo {
    pub git_sha: String,
    pub git_branch: String,
    pub git_timestamp: String,
    pub rustc_version: String,
    pub target_triple: String,
    pub cargo_profile: String,
    pub package_version: String,
    pub dependency_versions: Vec<String>,
}

impl Default for BuildInfo {
    fn default() -> Self {
        Self::new()
    }
}

impl BuildInfo {
    pub fn new() -> Self {
        let dependency_versions = option_env!("RAND_EXPLORER_BUILD_DEPS")
            .unwrap_or("unknown=unknown")
            .split(" | ")
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(str::to_owned)
            .collect::<Vec<_>>();

        Self {
            git_sha: option_env!("VERGEN_GIT_SHA").unwrap_or("unknown").to_string(),
            git_branch: option_env!("VERGEN_GIT_BRANCH").unwrap_or("unknown").to_string(),
            git_timestamp: option_env!("VERGEN_GIT_COMMIT_TIMESTAMP")
                .unwrap_or("unknown")
                .to_string(),
            rustc_version: option_env!("VERGEN_RUSTC_SEMVER").unwrap_or("unknown").to_string(),
            target_triple: option_env!("VERGEN_CARGO_TARGET_TRIPLE")
                .unwrap_or("unknown")
                .to_string(),
            cargo_profile: option_env!("PROFILE").unwrap_or("unknown").to_string(),
            package_version: option_env!("CARGO_PKG_VERSION").unwrap_or("unknown").to_string(),
            dependency_versions,
        }
    }
}
