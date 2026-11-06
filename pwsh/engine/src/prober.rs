use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use serde::Deserialize;

#[derive(Debug, Clone)]
pub struct ContextAction {
    pub label: String,
    pub command: String,
    pub source: String, // e.g. "package.json", "Cargo.toml", "uv", "docker-compose"
}

#[derive(Debug, Clone, Default)]
pub struct ProjectContext {
    pub root: Option<PathBuf>,
    pub git_root: Option<PathBuf>,
    pub project_type: Option<String>,
    pub actions: Vec<ContextAction>,
}

#[derive(Deserialize)]
struct PackageJson {
    scripts: Option<HashMap<String, String>>,
}

pub struct Prober;

impl Prober {
    pub fn probe(start_dir: &Path) -> ProjectContext {
        let mut context = ProjectContext::default();
        let canonical = dunce::canonicalize(start_dir).unwrap_or_else(|_| start_dir.to_path_buf());
        let home_dir = dirs::home_dir().map(|h| dunce::canonicalize(&h).unwrap_or(h));

        // First pass: locate git root if any
        for dir in canonical.ancestors() {
            if dir.join(".git").exists() {
                context.git_root = Some(dir.to_path_buf());
                break;
            }
            if let Some(ref home) = home_dir {
                if dir == home {
                    break;
                }
            }
        }

        // Second pass: locate nearest project boundary (bounded by git root or home)
        for dir in canonical.ancestors() {
            // Check Cargo
            if dir.join("Cargo.toml").exists() {
                if context.root.is_none() {
                    context.root = Some(dir.to_path_buf());
                    context.project_type = Some("Rust (Cargo)".to_string());
                }
                context.actions.push(ContextAction {
                    label: "cargo check".to_string(),
                    command: "cargo check".to_string(),
                    source: "Cargo.toml".to_string(),
                });
                context.actions.push(ContextAction {
                    label: "cargo run".to_string(),
                    command: "cargo run".to_string(),
                    source: "Cargo.toml".to_string(),
                });
                context.actions.push(ContextAction {
                    label: "cargo test".to_string(),
                    command: "cargo test".to_string(),
                    source: "Cargo.toml".to_string(),
                });
                context.actions.push(ContextAction {
                    label: "cargo build --release".to_string(),
                    command: "cargo build --release".to_string(),
                    source: "Cargo.toml".to_string(),
                });
                break;
            }

            // Check Node / package.json
            let pkg_path = dir.join("package.json");
            if pkg_path.exists() {
                if context.root.is_none() {
                    context.root = Some(dir.to_path_buf());
                }

                let runner = if dir.join("pnpm-lock.yaml").exists() {
                    "pnpm"
                } else if dir.join("yarn.lock").exists() {
                    "yarn"
                } else if dir.join("bun.lockb").exists() || dir.join("bun.lock").exists() {
                    "bun"
                } else {
                    "npm"
                };

                context.project_type = Some(format!("Node ({})", runner));

                if let Ok(content) = fs::read_to_string(&pkg_path) {
                    if let Ok(pkg) = serde_json::from_str::<PackageJson>(&content) {
                        if let Some(scripts) = pkg.scripts {
                            let priority_scripts = ["dev", "start", "build", "test", "lint"];
                            for &name in &priority_scripts {
                                if scripts.contains_key(name) {
                                    let cmd = if runner == "npm" {
                                        format!("npm run {}", name)
                                    } else {
                                        format!("{} {}", runner, name)
                                    };
                                    context.actions.push(ContextAction {
                                        label: format!("{} (package.json: {})", cmd, name),
                                        command: cmd,
                                        source: format!("{}:scripts", runner),
                                    });
                                }
                            }
                            for (name, _) in scripts {
                                if !priority_scripts.contains(&name.as_str()) {
                                    let cmd = if runner == "npm" {
                                        format!("npm run {}", name)
                                    } else {
                                        format!("{} {}", runner, name)
                                    };
                                    context.actions.push(ContextAction {
                                        label: format!("{} (package.json: {})", cmd, name),
                                        command: cmd,
                                        source: format!("{}:scripts", runner),
                                    });
                                }
                            }
                        }
                    }
                }
                break;
            }

            // Check Python (uv / poetry / requirements)
            let is_uv = dir.join("uv.lock").exists();
            let is_pyproject = dir.join("pyproject.toml").exists();
            if is_uv || is_pyproject || dir.join("requirements.txt").exists() {
                if context.root.is_none() {
                    context.root = Some(dir.to_path_buf());
                    context.project_type = Some(if is_uv {
                        "Python (uv)".to_string()
                    } else {
                        "Python".to_string()
                    });
                }

                if is_uv || is_pyproject {
                    context.actions.push(ContextAction {
                        label: "uv run python main.py".to_string(),
                        command: "uv run python main.py".to_string(),
                        source: "uv".to_string(),
                    });
                    context.actions.push(ContextAction {
                        label: "uv sync".to_string(),
                        command: "uv sync".to_string(),
                        source: "uv".to_string(),
                    });
                }

                if dir.join(".venv").exists() {
                    context.actions.push(ContextAction {
                        label: ".\\.venv\\Scripts\\Activate.ps1".to_string(),
                        command: ".\\.venv\\Scripts\\Activate.ps1".to_string(),
                        source: "venv".to_string(),
                    });
                }
                break;
            }

            // Check Docker Compose
            if dir.join("docker-compose.yml").exists() || dir.join("compose.yaml").exists() {
                if context.root.is_none() {
                    context.root = Some(dir.to_path_buf());
                    context.project_type = Some("Docker Compose".to_string());
                }
                context.actions.push(ContextAction {
                    label: "docker compose up -d".to_string(),
                    command: "docker compose up -d".to_string(),
                    source: "docker-compose".to_string(),
                });
                context.actions.push(ContextAction {
                    label: "docker compose logs -f".to_string(),
                    command: "docker compose logs -f".to_string(),
                    source: "docker-compose".to_string(),
                });
                context.actions.push(ContextAction {
                    label: "docker compose down".to_string(),
                    command: "docker compose down".to_string(),
                    source: "docker-compose".to_string(),
                });
                break;
            }

            // Do not cross git root or home directory when looking for project markers
            if let Some(ref git_root) = context.git_root {
                if dir == git_root {
                    break;
                }
            }
            if let Some(ref home) = home_dir {
                if dir == home {
                    break;
                }
            }
        }

        // Add git actions if in git repo
        if context.git_root.is_some() {
            context.actions.push(ContextAction {
                label: "git status -s".to_string(),
                command: "git status -s".to_string(),
                source: "git".to_string(),
            });
            context.actions.push(ContextAction {
                label: "git log -n 5 --oneline".to_string(),
                command: "git log -n 5 --oneline".to_string(),
                source: "git".to_string(),
            });
        }

        context
    }
}
