use semver::Version;

pub struct LockFile {
    version: Version,
    brews: Vec<Packages>,
}

pub struct Packages {
    name: String,
    version: Version,
    source: Option<String>,
    dependencies: Option<Vec<String>>,
}
