#[derive(Debug, serde::Deserialize)]
pub struct Config {
    #[serde(default)]
    pub name: String,
    pub project: std::path::PathBuf,
    pub remote_debug: Option<String>,
}

impl Config {
    pub fn try_from(manifest_path: &std::path::Path) -> crate::Result<Self> {
        let metadata = cargo_metadata::MetadataCommand::new()
            .manifest_path(manifest_path)
            .exec()?;

        let root = root(&metadata).unwrap();
        let package = metadata.packages.iter().find(|x| &x.id == root).unwrap();

        let mut configuration: Self = serde_json::from_value(package.metadata["godot"].clone())?;
        configuration.project = manifest_path
            .parent()
            .unwrap()
            .join(&configuration.project)
            .canonicalize()
            .unwrap();
        if configuration.name.is_empty() {
            configuration.name.clone_from(&package.name);
        }

        Ok(configuration)
    }

    pub fn into_args(self) -> Vec<String> {
        let mut args = vec![
            "--path".to_string(),
            self.project.to_str().unwrap().to_string(),
        ];

        if let Some(remote_debug) = self.remote_debug {
            args.push("--remote-debug".to_string());
            args.push(remote_debug);
        }

        args
    }
}

fn root(metadata: &cargo_metadata::Metadata) -> Option<&cargo_metadata::PackageId> {
    metadata.resolve.as_ref()?.root.as_ref()
}
