---
trigger: always_on
description: "Compact inventory of available local CLI tools, WSL-wrapped commands, and dev environment conventions"
---

# Local CLI & Environment Inventory

Available shell commands on this Windows machine (excluding `rg`/`fd`/`lsj` covered by `CLI_TOOLCHAIN.md`, `git` covered by `git-agent-flow`, and `uv` covered by `python_uv.md`):

| Domain | Available Commands | Notes & Preferences |
| :--- | :--- | :--- |
| **Node.js / JS / TS** | `pnpm`, `pnpx`, `node`, `nvm` | Always prefer `pnpm` / `pnpx` over `npm` or `yarn` (`NVM_HOME` managed). |
| **Rust** | `cargo`, `rustc`, `rustup`, `rust-analyzer`, `cargo clippy`, `rustfmt` | Use `cargo check` / `cargo clippy` for verification. |
| **Containers** | `docker`, `docker compose` | `DOCKER_HOST=tcp://localhost:2375`; `$PROFILE` wraps WSL `docker-debian`. Prefer `docker compose` (v2). |
| **Cloud / Colab** | `gcloud`, `colab` | `colab` runs in WSL `colab-debian` (use `colab` in profile shells, or `wsl.exe -d colab-debian -- colab` in non-profile shells). |
| **Build / Infra** | `cmake`, `dotnet`, `terraform`, `task` | Standard system PATH binaries. |
| **Code / Data / Media** | `ast-grep` (`sg`), `jq`, `ffmpeg`, `ffprobe`, `litert-lm` | Structural AST search (`sg`), JSON filtering (`jq`), media (`ffmpeg`). |

## Environment Conventions
- **Encoding**: `PYTHONUTF8=1` is set globally.
- **Non-Profile Subshells**: Automated `pwsh` subshells that skip `$PROFILE` will not load profile functions (`colab`, WSL `docker` wrapper); fall back to `wsl.exe -d <distro> -- <cmd>` or the Windows executable directly.
