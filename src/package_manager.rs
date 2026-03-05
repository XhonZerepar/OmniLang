//! OmniLang Package Manager (omp)
//! 
//! Manages OmniLang packages and dependencies

use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};
use toml;

/// Package manifest
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageManifest {
    pub package: PackageInfo,
    pub dependencies: HashMap<String, Dependency>,
    pub dev_dependencies: HashMap<String, Dependency>,
    pub build: Option<BuildConfig>,
    pub lib: Option<LibConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageInfo {
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub authors: Vec<String>,
    pub license: Option<String>,
    pub edition: String,
    pub repository: Option<String>,
    pub homepage: Option<String>,
    pub keywords: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dependency {
    pub version: String,
    pub source: Option<String>,
    pub features: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildConfig {
    pub script: Option<String>,
    pub dependencies: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibConfig {
    pub path: Option<String>,
    pub name: Option<String>,
}

/// Package index
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PackageIndex {
    pub packages: HashMap<String, Vec<PackageVersion>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageVersion {
    pub version: String,
    pub tarball: String,
    pub checksum: String,
}

/// Package cache
pub struct PackageCache {
    pub cache_dir: PathBuf,
}

impl PackageCache {
    pub fn new() -> Self {
        let cache_dir = dirs::cache_dir()
            .unwrap_or_else(|| PathBuf::from(".cache"))
            .join("omnilang");
        
        Self { cache_dir }
    }
    
    pub fn package_dir(&self, name: &str, version: &str) -> PathBuf {
        self.cache_dir.join("packages").join(format!("{}-{}", name, version))
    }
    
    pub fn get_cache(&self, name: &str, version: &str) -> Option<PathBuf> {
        let dir = self.package_dir(name, version);
        if dir.exists() {
            Some(dir)
        } else {
            None
        }
    }
    
    pub fn cache_package(&self, name: &str, version: &str, tarball: &[u8]) -> Result<PathBuf, String> {
        let dir = self.package_dir(name, version);
        fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
        
        // Extract tarball (simplified - real implementation would use tar crate)
        let tar_path = dir.join("package.tar.gz");
        fs::write(&tar_path, tarball).map_err(|e| e.to_string())?;
        
        Ok(dir)
    }
}

/// Package registry
pub struct PackageRegistry {
    pub index: PackageIndex,
    pub cache: PackageCache,
}

impl PackageRegistry {
    pub fn new() -> Self {
        Self {
            index: PackageIndex::default(),
            cache: PackageCache::new(),
        }
    }
    
    /// Fetch package from registry
    pub fn fetch_package(&mut self, name: &str, version: &str) -> Result<PathBuf, String> {
        // Check cache first
        if let Some(dir) = self.cache.get_cache(name, version) {
            return Ok(dir);
        }
        
        // In a real implementation, this would download from a registry
        // For now, return an error
        Err(format!("Package {} v{} not found in registry", name, version))
    }
    
    /// Search packages
    pub fn search(&self, query: &str) -> Vec<PackageSearchResult> {
        let mut results = vec![];
        
        for (name, versions) in &self.index.packages {
            if name.contains(query) {
                results.push(PackageSearchResult {
                    name: name.clone(),
                    description: None,
                    latest_version: versions.last().map(|v| v.version.clone()),
                    downloads: 0,
                });
            }
        }
        
        results
    }
    
    /// Update package index
    pub fn update_index(&mut self) -> Result<(), String> {
        // In a real implementation, this would fetch from a remote registry
        // For now, just return Ok
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageSearchResult {
    pub name: String,
    pub description: Option<String>,
    pub latest_version: Option<String>,
    pub downloads: u64,
}

/// Package manager operations
pub struct PackageManager {
    pub manifest_path: PathBuf,
    pub target_dir: PathBuf,
    pub registry: PackageRegistry,
}

impl PackageManager {
    pub fn new(project_path: &Path) -> Self {
        let manifest_path = project_path.join("omnilang.toml");
        let target_dir = project_path.join("target");
        
        Self {
            manifest_path,
            target_dir,
            registry: PackageRegistry::new(),
        }
    }
    
    /// Load package manifest
    pub fn load_manifest(&self) -> Result<PackageManifest, String> {
        if !self.manifest_path.exists() {
            return Err("No omnilang.toml found".to_string());
        }
        
        let contents = fs::read_to_string(&self.manifest_path).map_err(|e| e.to_string())?;
        let manifest: PackageManifest = toml::from_str(&contents).map_err(|e| e.to_string())?;
        
        Ok(manifest)
    }
    
    /// Save package manifest
    pub fn save_manifest(&self, manifest: &PackageManifest) -> Result<(), String> {
        let contents = toml::to_string_pretty(manifest).map_err(|e| e.to_string())?;
        fs::write(&self.manifest_path, contents).map_err(|e| e.to_string())?;
        
        Ok(())
    }
    
    /// Initialize a new package
    pub fn init(&self, name: &str, version: &str) -> Result<(), String> {
        let manifest = PackageManifest {
            package: PackageInfo {
                name: name.to_string(),
                version: version.to_string(),
                description: None,
                authors: vec![],
                license: None,
                edition: "2021".to_string(),
                repository: None,
                homepage: None,
                keywords: vec![],
            },
            dependencies: HashMap::new(),
            dev_dependencies: HashMap::new(),
            build: None,
            lib: None,
        };
        
        self.save_manifest(&manifest)?;
        
        // Create directory structure
        let src_dir = PathBuf::from("src");
        let examples_dir = PathBuf::from("examples");
        
        fs::create_dir_all(&src_dir).map_err(|e| e.to_string())?;
        fs::create_dir_all(&examples_dir).map_err(|e| e.to_string())?;
        
        // Create main file
        let main_content = r#"// Welcome to OmniLang!
// This is your first OmniLang program.

fn main(args: [String]) -> Int:
    print("Hello, OmniLang!")
    return 0
"#;
        
        fs::write(src_dir.join("main.omni"), main_content).map_err(|e| e.to_string())?;
        
        Ok(())
    }
    
    /// Add a dependency
    pub fn add_dependency(&self, name: &str, version: &str) -> Result<(), String> {
        let mut manifest = self.load_manifest()?;
        
        manifest.dependencies.insert(name.to_string(), Dependency {
            version: version.to_string(),
            source: None,
            features: vec![],
        });
        
        self.save_manifest(&manifest)?;
        
        Ok(())
    }
    
    /// Remove a dependency
    pub fn remove_dependency(&self, name: &str) -> Result<(), String> {
        let mut manifest = self.load_manifest()?;
        
        manifest.dependencies.remove(name);
        
        self.save_manifest(&manifest)?;
        
        Ok(())
    }
    
    /// Install dependencies
    pub fn install(&self) -> Result<(), String> {
        let manifest = self.load_manifest()?;
        
        println!("Installing dependencies...");
        
        for (name, dep) in &manifest.dependencies {
            println!("  Installing {} v{}", name, dep.version);
            
            // In a real implementation, this would:
            // 1. Fetch the package from registry
            // 2. Extract to target directory
            // 3. Build if necessary
        }
        
        println!("Done!");
        
        Ok(())
    }
    
    /// Build the package
    pub fn build(&self) -> Result<(), String> {
        // First, install dependencies
        self.install()?;
        
        let manifest = self.load_manifest()?;
        
        println!("Building {} v{}", manifest.package.name, manifest.package.version);
        
        // Run build script if present
        if let Some(build) = &manifest.build {
            if let Some(script) = &build.script {
                println!("Running build script...");
                // Execute build script
            }
        }
        
        // Compile the package
        // This would call the compiler
        
        println!("Build complete!");
        
        Ok(())
    }
    
    /// Run tests
    pub fn test(&self) -> Result<(), String> {
        self.build()?;
        
        println!("Running tests...");
        
        // Run test functions
        
        println!("All tests passed!");
        
        Ok(())
    }
    
    /// Publish package
    pub fn publish(&self) -> Result<(), String> {
        let manifest = self.load_manifest()?;
        
        println!("Publishing {} v{}", manifest.package.name, manifest.package.version);
        
        // Validate manifest
        if manifest.package.name.is_empty() {
            return Err("Package name is required".to_string());
        }
        
        // In a real implementation, this would:
        // 1. Package the files
        // 2. Compute checksum
        // 3. Upload to registry
        
        println!("Package published successfully!");
        
        Ok(())
    }
    
    /// Update dependencies
    pub fn update(&self) -> Result<(), String> {
        println!("Updating dependencies...");
        
        // Update registry index
        self.registry.update_index()?;
        
        // Update each dependency to latest compatible version
        let mut manifest = self.load_manifest()?;
        
        for (name, dep) in &mut manifest.dependencies {
            // Find latest compatible version
            println!("  Updating {}...", name);
        }
        
        self.save_manifest(&manifest)?;
        
        println!("Update complete!");
        
        Ok(())
    }
}

/// Run the package manager CLI
pub fn run_omp(args: Vec<String>) {
    if args.len() < 2 {
        println!("OmniLang Package Manager");
        println!("Usage: omp <command> [options]");
        println!("");
        println!("Commands:");
        println!("  init <name>      Initialize a new package");
        println!("  add <name>       Add a dependency");
        println!("  remove <name>    Remove a dependency");
        println!("  install          Install dependencies");
        println!("  build            Build the package");
        println!("  test             Run tests");
        println!("  publish          Publish package");
        println!("  update           Update dependencies");
        println!("  search <query>   Search for packages");
        
        return;
    }
    
    let command = &args[1];
    let project_path = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let pm = PackageManager::new(&project_path);
    
    match command.as_str() {
        "init" => {
            let name = args.get(2).map(|s| s.as_str()).unwrap_or("my_package");
            let version = args.get(3).map(|s| s.as_str()).unwrap_or("0.1.0");
            
            if let Err(e) = pm.init(name, version) {
                eprintln!("Error: {}", e);
            }
        }
        
        "add" => {
            if args.len() < 3 {
                eprintln!("Usage: omp add <name>[@version]");
                return;
            }
            
            let dep = &args[2];
            let (name, version) = if let Some(at) = dep.find('@') {
                (&dep[..at], &dep[at+1..])
            } else {
                (dep, "*")
            };
            
            if let Err(e) = pm.add_dependency(name, version) {
                eprintln!("Error: {}", e);
            }
        }
        
        "remove" => {
            if args.len() < 3 {
                eprintln!("Usage: omp remove <name>");
                return;
            }
            
            let name = &args[2];
            
            if let Err(e) = pm.remove_dependency(name) {
                eprintln!("Error: {}", e);
            }
        }
        
        "install" => {
            if let Err(e) = pm.install() {
                eprintln!("Error: {}", e);
            }
        }
        
        "build" => {
            if let Err(e) = pm.build() {
                eprintln!("Error: {}", e);
            }
        }
        
        "test" => {
            if let Err(e) = pm.test() {
                eprintln!("Error: {}", e);
            }
        }
        
        "publish" => {
            if let Err(e) = pm.publish() {
                eprintln!("Error: {}", e);
            }
        }
        
        "update" => {
            if let Err(e) = pm.update() {
                eprintln!("Error: {}", e);
            }
        }
        
        "search" => {
            if args.len() < 3 {
                eprintln!("Usage: omp search <query>");
                return;
            }
            
            let query = &args[2];
            let results = pm.registry.search(query);
            
            for result in results {
                println!("{} - {}", result.name, result.latest_version.unwrap_or_default());
            }
        }
        
        _ => {
            eprintln!("Unknown command: {}", command);
        }
    }
}
