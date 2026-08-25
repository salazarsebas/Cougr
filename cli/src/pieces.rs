//! Embedded, editable capability pieces for `cougr add`.

use std::fs;
use std::path::Path;

use rust_embed::RustEmbed;

use crate::error::CliError;

#[derive(RustEmbed)]
#[folder = "pieces/"]
struct Assets;

#[derive(Debug)]
struct Manifest {
    piece: Vec<Piece>,
}

#[derive(Debug)]
struct Piece {
    name: String,
    description: String,
    file: Vec<FileSpec>,
    wiring: Vec<String>,
    dependencies: Vec<String>,
}

#[derive(Debug)]
struct FileSpec {
    source: String,
    target: String,
}

pub fn run(name: Option<&str>, list: bool) -> Result<(), CliError> {
    let cwd = std::env::current_dir()
        .map_err(|err| CliError::io("resolve the current directory", ".", err))?;
    let manifest = manifest()?;

    if list {
        if name.is_some() {
            return Err(CliError::UnknownPiece {
                name: "--list with a piece name".to_string(),
                available: "--list takes no piece name".to_string(),
            });
        }
        for piece in &manifest.piece {
            println!("{:<24} {}", piece.name, piece.description);
        }
        return Ok(());
    }

    let name = name.ok_or_else(|| CliError::UnknownPiece {
        name: "<missing>".to_string(),
        available: available_names(&manifest),
    })?;
    add(name, &cwd, &manifest)
}

fn manifest() -> Result<Manifest, CliError> {
    let asset = Assets::get("pieces.toml").ok_or_else(|| CliError::MissingPieceAsset {
        piece: "manifest".to_string(),
        file: "pieces.toml".to_string(),
    })?;
    parse_manifest(&String::from_utf8_lossy(&asset.data)).map_err(|message| {
        CliError::io(
            "parse the embedded pieces manifest",
            "pieces.toml",
            std::io::Error::new(std::io::ErrorKind::InvalidData, message),
        )
    })
}

fn parse_manifest(source: &str) -> Result<Manifest, String> {
    let mut manifest = Manifest { piece: Vec::new() };
    let mut current_file = false;
    for raw_line in source.lines() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if line == "[[piece]]" {
            manifest.piece.push(Piece {
                name: String::new(),
                description: String::new(),
                file: Vec::new(),
                wiring: Vec::new(),
                dependencies: Vec::new(),
            });
            current_file = false;
            continue;
        }
        if line == "[[piece.file]]" {
            let piece = manifest.piece.last_mut().ok_or("file has no piece")?;
            piece.file.push(FileSpec {
                source: String::new(),
                target: String::new(),
            });
            current_file = true;
            continue;
        }
        let (key, value) = line.split_once('=').ok_or_else(|| format!("invalid line: {line}"))?;
        let key = key.trim();
        let piece = manifest.piece.last_mut().ok_or("field has no piece")?;
        if current_file {
            let file = piece.file.last_mut().ok_or("file field has no file")?;
            match key {
                "source" => file.source = parse_string(value)?,
                "target" => file.target = parse_string(value)?,
                _ => return Err(format!("unknown file field: {key}")),
            }
        } else {
            match key {
                "name" => piece.name = parse_string(value)?,
                "description" => piece.description = parse_string(value)?,
                "wiring" => piece.wiring = parse_string_array(value)?,
                "dependencies" => piece.dependencies = parse_string_array(value)?,
                _ => return Err(format!("unknown piece field: {key}")),
            }
        }
    }
    if manifest.piece.iter().any(|piece| piece.name.is_empty() || piece.file.is_empty()) {
        return Err("every piece needs a name and file".to_string());
    }
    Ok(manifest)
}

fn parse_string(value: &str) -> Result<String, String> {
    let value = value.trim();
    if value.len() >= 2 && value.starts_with('"') && value.ends_with('"') {
        Ok(value[1..value.len() - 1].replace("\\\"", "\""))
    } else {
        Err(format!("expected quoted string: {value}"))
    }
}

fn parse_string_array(value: &str) -> Result<Vec<String>, String> {
    let value = value.trim();
    if !value.starts_with('[') || !value.ends_with(']') {
        return Err(format!("expected string array: {value}"));
    }
    value[1..value.len() - 1]
        .split(',')
        .filter(|item| !item.trim().is_empty())
        .map(parse_string)
        .collect()
}

fn add(name: &str, project: &Path, manifest: &Manifest) -> Result<(), CliError> {
    let piece = manifest
        .piece
        .iter()
        .find(|piece| piece.name == name)
        .ok_or_else(|| CliError::UnknownPiece {
            name: name.to_string(),
            available: available_names(manifest),
        })?;

    let lib_path = project.join("src/lib.rs");
    let cargo_path = project.join("Cargo.toml");
    if !cargo_path.is_file() || !lib_path.is_file() {
        return Err(CliError::InvalidProject {
            path: project.to_path_buf(),
        });
    }

    let mut writes = Vec::new();
    for file in &piece.file {
        let asset = Assets::get(&file.source).ok_or_else(|| CliError::MissingPieceAsset {
            piece: piece.name.clone(),
            file: file.source.clone(),
        })?;
        writes.push((project.join(&file.target), String::from_utf8_lossy(&asset.data).into_owned()));
    }

    let lib = fs::read_to_string(&lib_path)
        .map_err(|err| CliError::io("read the project module file", &lib_path, err))?;
    let cargo = fs::read_to_string(&cargo_path)
        .map_err(|err| CliError::io("read the project manifest", &cargo_path, err))?;
    let wiring = piece
        .wiring
        .iter()
        .filter(|line| !lib.lines().any(|existing| existing.trim() == line.trim()))
        .cloned()
        .collect::<Vec<_>>();
    let dependencies = piece
        .dependencies
        .iter()
        .filter(|line| !cargo.lines().any(|existing| existing.trim() == line.trim()))
        .cloned()
        .collect::<Vec<_>>();

    let conflicts = writes
        .iter()
        .filter(|(path, _)| path.exists())
        .map(|(path, _)| path.clone())
        .collect::<Vec<_>>();
    if !conflicts.is_empty() {
        return Err(CliError::PieceConflict {
            piece: piece.name.clone(),
            files: writes.iter().map(|(path, _)| path.clone()).collect(),
        });
    }

    for (path, contents) in &writes {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|err| CliError::io("create the piece directory", parent, err))?;
        }
        fs::write(path, contents).map_err(|err| CliError::io("write the piece file", path, err))?;
    }

    if !wiring.is_empty() {
        let mut updated = lib;
        updated.push('\n');
        for line in &wiring {
            updated.push_str(line);
            updated.push('\n');
        }
        fs::write(&lib_path, updated)
            .map_err(|err| CliError::io("update the project module file", &lib_path, err))?;
    }

    if !dependencies.is_empty() {
        let marker = "[dependencies]\n";
        let insert_at = cargo.find(marker).map(|index| index + marker.len()).ok_or_else(|| {
            CliError::InvalidProject {
                path: project.to_path_buf(),
            }
        })?;
        let dependency_text = dependencies
            .iter()
            .map(|line| format!("{line}\n"))
            .collect::<String>();
        let mut updated = cargo;
        updated.insert_str(insert_at, &dependency_text);
        fs::write(&cargo_path, updated)
            .map_err(|err| CliError::io("update the project manifest", &cargo_path, err))?;
    }

    println!("Added `{}`: {}", piece.name, piece.description);
    for (path, _) in &writes {
        println!("  {}", path.strip_prefix(project).unwrap_or(path).display());
    }
    for line in wiring {
        println!("  src/lib.rs: {line}");
    }
    for line in dependencies {
        println!("  Cargo.toml: {line}");
    }
    Ok(())
}

fn available_names(manifest: &Manifest) -> String {
    manifest
        .piece
        .iter()
        .map(|piece| piece.name.as_str())
        .collect::<Vec<_>>()
        .join(", ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::new;
    use crate::template::Template;

    #[test]
    fn lists_v1_pieces() {
        let manifest = manifest().unwrap();
        assert!(manifest.piece.iter().any(|piece| piece.name == "session-auth"));
        assert!(manifest.piece.iter().any(|piece| piece.name == "hidden-hand"));
        assert!(manifest.piece.iter().any(|piece| piece.name == "standards/pausable"));
    }

    #[test]
    fn adds_wiring_and_refuses_a_second_write() {
        let dir = tempfile::tempdir().unwrap();
        new::run("demo", Template::Starter, Some(dir.path())).unwrap();
        let project = dir.path().join("demo");
        add("session-auth", &project, &manifest().unwrap()).unwrap();
        let lib = fs::read_to_string(project.join("src/lib.rs")).unwrap();
        assert!(lib.contains("pub mod session_auth;"));
        let error = add("session-auth", &project, &manifest().unwrap()).unwrap_err();
        assert!(matches!(error, CliError::PieceConflict { .. }));
    }
}
