use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Serialize, Deserialize)]
pub struct Manifest {
    pub package: Package,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Package {
    pub name: String,
    pub version: String,
}

pub fn init_project(name: &str) -> Result<(), String> {
    let project_dir = Path::new(name);
    
    if project_dir.exists() {
        return Err(format!("Directory '{}' already exists.", name));
    }

    fs::create_dir_all(project_dir.join("src")).map_err(|e| e.to_string())?;

    let manifest = Manifest {
        package: Package {
            name: name.to_string(),
            version: "0.1.0".to_string(),
        }
    };

    let toml_string = toml::to_string(&manifest).map_err(|e| e.to_string())?;
    fs::write(project_dir.join("viyal.toml"), toml_string).map_err(|e| e.to_string())?;

    let main_code = format!("class Main {{\n    void run() {{\n        print(\"Hello from {}!\");\n    }}\n}}\n", name);
    fs::write(project_dir.join("src").join("main.vy"), main_code).map_err(|e| e.to_string())?;

    Ok(())
}

pub fn find_project_root() -> Result<PathBuf, String> {
    let mut current_dir = std::env::current_dir().map_err(|e| e.to_string())?;
    
    loop {
        let manifest_path = current_dir.join("viyal.toml");
        if manifest_path.exists() {
            return Ok(current_dir);
        }
        
        if !current_dir.pop() {
            return Err("Could not find viyal.toml in the current or parent directories.".to_string());
        }
    }
}

pub fn read_manifest(path: &Path) -> Result<Manifest, String> {
    let content = fs::read_to_string(path).map_err(|e| e.to_string())?;
    let manifest: Manifest = toml::from_str(&content).map_err(|e| e.to_string())?;
    Ok(manifest)
}
