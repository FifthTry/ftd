use std::io::Read;

extern crate self as fastn_update;

static GITHUB_PAGES_REGEX: once_cell::sync::Lazy<regex::Regex> =
    once_cell::sync::Lazy::new(|| regex::Regex::new(r"([^/]+)\.github\.io/([^/]+)").unwrap());

fn extract_github_details(pages_url: &str) -> Option<(String, String)> {
    if let Some(captures) = GITHUB_PAGES_REGEX.captures(pages_url) {
        let username = captures.get(1).unwrap().as_str().to_string();
        let repository = captures.get(2).unwrap().as_str().to_string();
        Some((username, repository))
    } else {
        None
    }
}

// https://api.github.com/repos/User/repo/:archive_format/:ref
// https://stackoverflow.com/questions/8377081/github-api-download-zip-or-tarball-link
async fn resolve_dependency_from_gh(
    username: &str,
    repository: &str,
) -> fastn_core::Result<Vec<u8>> {
    let url = format!(
        "https://api.github.com/repos/{}/{}/zipball",
        username, repository
    );
    let zipball = fastn_core::http::http_get(&url).await?;
    Ok(zipball)
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Manifest {
    pub packages: Vec<Package>,
}

impl Manifest {
    pub fn new(packages: Vec<Package>) -> Self {
        Manifest { packages }
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
pub enum PackageSource {
    Github {
        username: String,
        repository: String,
    },
}

impl PackageSource {
    pub fn github(username: String, repository: String) -> Self {
        PackageSource::Github {
            username,
            repository,
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Package {
    pub name: String,
    pub version: Option<String>,
    pub source: PackageSource,
    pub checksum: String,
    pub dependencies: Vec<String>,
}

impl Package {
    pub fn new(
        name: String,
        version: Option<String>,
        source: PackageSource,
        checksum: String,
        dependencies: Vec<String>,
    ) -> Self {
        Package {
            name,
            version,
            source,
            checksum,
            dependencies,
        }
    }
}

pub async fn resolve_dependencies(config: &fastn_core::Config) -> fastn_core::Result<()> {
    use std::io::Seek;

    let mut packages: Vec<Package> = vec![];

    for dependency in &config.package.dependencies {
        if let Some((username, repository)) =
            extract_github_details(dependency.package.name.as_str())
        {
            let zipball =
                resolve_dependency_from_gh(username.as_str(), repository.as_str()).await?;
            let checksum = fastn_core::utils::generate_hash(zipball.clone());
            let mut zipball_cursor = std::io::Cursor::new(zipball);
            zipball_cursor.seek(std::io::SeekFrom::Start(0))?;
            let mut archive = zip::ZipArchive::new(zipball_cursor)?;
            let dependency_path = &config.packages_root.join(&dependency.package.name);
            for i in 0..archive.len() {
                let mut entry = archive.by_index(i)?;

                if entry.is_file() {
                    let mut buffer = Vec::new();
                    entry.read_to_end(&mut buffer)?;
                    let name = entry
                        .name()
                        .split_once('/')
                        .map(|(_, name)| name)
                        .unwrap_or(entry.name());
                    config
                        .ds
                        .write_content(&dependency_path.join(name), buffer)
                        .await?;
                }
            }

            let source = PackageSource::github(username, repository);

            let package = Package::new(
                dependency.package.name.clone(),
                None, // todo: fix this when versioning is available
                source,
                checksum,
                dependency
                    .package
                    .dependencies
                    .iter()
                    .map(|d| d.package.name.clone())
                    .collect(),
            );

            packages.push(package);
        }
    }

    let manifest = Manifest::new(packages);
    let dot_fastn_dir = config.ds.root().join(".fastn");

    config
        .ds
        .write_content(
            dot_fastn_dir.join("manifest.json"),
            serde_json::to_vec_pretty(&manifest)?,
        )
        .await?;

    println!("Wrote manifest.json");

    Ok(())
}

pub async fn update(config: &fastn_core::Config) -> fastn_core::Result<()> {
    if let Err(e) = std::fs::remove_dir_all(config.ds.root().join(".packages")) {
        match e.kind() {
            std::io::ErrorKind::NotFound => {}
            _ => return Err(e.into()),
        }
    };

    let c = fastn_core::Config::read_current(false).await?;
    if c.package.dependencies.is_empty() {
        println!("No dependencies to update.");
        return Ok(());
    }

    resolve_dependencies(config).await?;

    if c.package.dependencies.len() == 1 {
        println!("Updated the package dependency.");
    } else {
        println!("Updated {} dependencies.", c.package.dependencies.len())
    }

    Ok(())
}
