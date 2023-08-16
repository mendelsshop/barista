pub struct LockFile {
    version: usize,
    brews: Vec<Packages>,
}

pub struct Packages {
    name: String,
    version: usize,
    source: Option<String>,
    dependencies: Option<Vec<String>>,
}
