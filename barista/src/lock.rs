use semver::Version;
use serde::Serialize;
#[derive(Serialize, Debug)]
pub struct LockFile {
    name: String,
    version: Version,
    brews: Vec<Package>,
}

impl LockFile {
    pub fn new(name: String, version: Version) -> Self {
        Self {
            name,
            version,
            brews: vec![],
        }
    }

    pub fn push(&mut self, value: Package) {
        self.brews.push(value)
    }
}
#[derive(Serialize, Debug)]

pub struct Package {
    name: String,
    // TODO: make this be a Version type
    version: String,
    authors: String,
    url: String,
    source: Option<String>,
    dependencies: Option<Vec<String>>,
}

impl Package {
    pub fn new(
        name: String,
        version: String,
        authors: String,
        url: String,
        source: Option<String>,
        dependencies: Option<Vec<String>>,
    ) -> Self {
        Self {
            name,
            version,
            authors,
            url,
            source,
            dependencies,
        }
    }

    pub fn set_dependencies(&mut self, dependencies: Vec<String>) {
        self.dependencies = Some(dependencies);
    }
}
