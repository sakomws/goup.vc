use std::{
    collections::{BTreeMap, HashSet},
    env, fs,
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::{Context, Result, bail};
use sha2::{Digest, Sha256};
use which::which;

/// Environment variable that may provide the current commit SHA.
const COMMIT_SHA_ENV_VAR: &str = "OCG_COMMIT_SHA";

/// Static asset directories whose files should be content hashed.
const HASHED_ASSET_DIRS: [&str; 2] = ["css", "js"];

/// Path to the static assets distribution directory relative to this crate.
const STATIC_DIST_PATH: &str = "../../ocg-server/dist/static";

/// Path to the static assets directory relative to this crate.
const STATIC_PATH: &str = "../../ocg-server/static";

/// Path to the templates distribution directory relative to this crate.
const TEMPLATES_DIST_PATH: &str = "../../ocg-server/dist/templates";

/// Path to the templates directory relative to this crate.
const TEMPLATES_PATH: &str = "../../ocg-server/templates";

/// Mapping from plain static asset paths to their final hashed paths.
type AssetsManifest = BTreeMap<String, String>;

/// Static asset that receives a content-hashed filename.
struct HashedAsset {
    /// Plain static asset paths referenced by this asset.
    dependencies: Vec<String>,
    /// Source file path.
    source_path: PathBuf,
}

fn main() -> Result<()> {
    println!("cargo:rerun-if-changed={STATIC_PATH}");
    println!("cargo:rerun-if-changed={TEMPLATES_PATH}");
    println!("cargo:rerun-if-env-changed={COMMIT_SHA_ENV_VAR}");
    println!("cargo:rerun-if-env-changed=OCG_SKIP_ASSETS");
    println!("cargo:rustc-env=OCG_COMMIT_SHA={}", commit_sha());

    if env::var_os("OCG_SKIP_ASSETS").is_some() {
        return Ok(());
    }

    if which("tailwindcss").is_err() {
        bail!("tailwindcss not found in PATH (required)");
    }

    let out_dir = PathBuf::from(env::var("OUT_DIR").context("OUT_DIR not set")?);
    let generated_styles_path = out_dir.join("styles.css");
    let generated_styles_str = generated_styles_path
        .to_str()
        .context("OUT_DIR path is not valid UTF-8")?;

    // `@source` paths in styles.src.css are resolved from the working directory.
    // They point at `templates` and `static/js` inside ocg-server. Cargo runs this
    // build script from crates/ocg-assets, so Tailwind has to be started in the
    // server directory or those sources are skipped and the stylesheet is empty.
    let server_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../ocg-server")
        .canonicalize()
        .context("failed to resolve the ocg-server directory for tailwindcss")?;
    run(
        "tailwindcss",
        &[
            "-i",
            "static/css/styles.src.css",
            "-o",
            generated_styles_str,
        ],
        Some(&server_dir),
    )?;

    let mut expected_static_files = HashSet::new();
    sync_unhashed_static_dir(
        Path::new(STATIC_PATH),
        Path::new(STATIC_PATH),
        Path::new(STATIC_DIST_PATH),
        &mut expected_static_files,
    )?;

    let assets_manifest =
        write_hashed_static_assets(&generated_styles_path, &mut expected_static_files)?;
    prune_unexpected_files(Path::new(STATIC_DIST_PATH), &expected_static_files)?;

    let sorted_refs = asset_refs_by_length(&assets_manifest);
    let mut expected_template_files = HashSet::new();
    sync_templates_dir(
        Path::new(TEMPLATES_PATH),
        Path::new(TEMPLATES_DIST_PATH),
        &sorted_refs,
        &mut expected_template_files,
    )?;
    prune_unexpected_files(Path::new(TEMPLATES_DIST_PATH), &expected_template_files)?;

    Ok(())
}

/// Returns the commit SHA for the current build.
fn commit_sha() -> String {
    if let Some(sha) = env::var(COMMIT_SHA_ENV_VAR)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
    {
        return sha;
    }

    "unknown".to_string()
}

/// Writes content-hashed static assets and returns their final path manifest.
fn write_hashed_static_assets(
    generated_styles_path: &Path,
    expected_files: &mut HashSet<PathBuf>,
) -> Result<AssetsManifest> {
    let graph = build_hashed_assets_graph(generated_styles_path)?;
    let mut manifest = AssetsManifest::new();
    let mut visiting = Vec::new();

    for plain_path in graph.keys() {
        write_hashed_static_asset(
            plain_path,
            &graph,
            &mut manifest,
            &mut visiting,
            expected_files,
        )?;
    }

    Ok(manifest)
}

/// Builds the graph of hashable static assets and the references between them.
fn build_hashed_assets_graph(
    generated_styles_path: &Path,
) -> Result<BTreeMap<String, HashedAsset>> {
    let mut assets_to_hash: Vec<(String, PathBuf)> = Vec::new();
    assets_to_hash.push((
        "/static/css/styles.css".to_string(),
        generated_styles_path.to_path_buf(),
    ));

    let mut source_paths = Vec::new();
    for dir in HASHED_ASSET_DIRS {
        collect_file_paths(&Path::new(STATIC_PATH).join(dir), &mut source_paths)?;
    }

    let stale_styles_path = Path::new(STATIC_PATH).join("css/styles.css");
    for source_path in source_paths {
        if source_path == stale_styles_path {
            continue;
        }
        let plain_path = static_asset_path(&source_path)?;
        assets_to_hash.push((plain_path, source_path));
    }

    let mut plain_paths: Vec<String> = assets_to_hash.iter().map(|(p, _)| p.clone()).collect();
    plain_paths.sort();

    let mut graph = BTreeMap::new();
    for (plain_path, source_path) in assets_to_hash {
        let content = fs::read_to_string(&source_path)?;
        let dependencies = plain_paths
            .iter()
            .filter(|dependency_path| content_contains_asset_ref(&content, dependency_path))
            .cloned()
            .collect();

        graph.insert(
            plain_path,
            HashedAsset {
                dependencies,
                source_path,
            },
        );
    }

    Ok(graph)
}

/// Collects all file paths in a directory and its subdirectories.
fn collect_file_paths(path: &Path, file_paths: &mut Vec<PathBuf>) -> Result<()> {
    if path.is_dir() {
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                collect_file_paths(&path, file_paths)?;
            } else if path.is_file() {
                file_paths.push(path);
            }
        }
    }

    Ok(())
}

/// Writes an asset after all of its dependencies have final hashed paths.
fn write_hashed_static_asset(
    plain_path: &str,
    graph: &BTreeMap<String, HashedAsset>,
    manifest: &mut AssetsManifest,
    visiting: &mut Vec<String>,
    expected_files: &mut HashSet<PathBuf>,
) -> Result<()> {
    if manifest.contains_key(plain_path) {
        return Ok(());
    }

    if let Some(position) = visiting.iter().position(|path| path == plain_path) {
        let mut cycle = visiting[position..].to_vec();
        cycle.push(plain_path.to_string());
        bail!("circular hashed asset references: {}", cycle.join(" -> "));
    }

    let Some(asset) = graph.get(plain_path) else {
        bail!("hashed asset not found in dependency graph: {plain_path}");
    };

    visiting.push(plain_path.to_string());
    for dependency_path in &asset.dependencies {
        write_hashed_static_asset(dependency_path, graph, manifest, visiting, expected_files)?;
    }
    visiting.pop();

    let original_content = fs::read_to_string(&asset.source_path)?;
    let sorted_refs = asset_refs_by_length(manifest);
    let final_content = replace_hashed_assets_refs_in_content(&original_content, &sorted_refs);
    let hash = calculate_hash(final_content.as_bytes());
    let hashed_path = hashed_static_asset_path(plain_path, &hash);

    let hashed_dist_path = static_dist_path(&hashed_path)?;
    write_if_changed(&hashed_dist_path, final_content.as_bytes())?;
    expected_files.insert(hashed_dist_path);

    manifest.insert(plain_path.to_string(), hashed_path);

    Ok(())
}

/// Returns the public path for a source static asset.
fn static_asset_path(path: &Path) -> Result<String> {
    let rel = path
        .strip_prefix(STATIC_PATH)
        .with_context(|| format!("expected path under {STATIC_PATH}: {}", path.display()))?;
    Ok(format!("/static/{}", rel.display()))
}

/// Returns the final public path for a content-hashed static asset.
fn hashed_static_asset_path(plain_path: &str, hash: &str) -> String {
    let path = Path::new(plain_path);
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
    let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");

    let hashed_file_name = if ext.is_empty() {
        format!("{}.{}", stem, &hash[..8])
    } else {
        format!("{}.{}.{}", stem, &hash[..8], ext)
    };
    let hashed_path = path
        .parent()
        .map(|parent| parent.join(&hashed_file_name))
        .unwrap_or_else(|| Path::new(&hashed_file_name).to_path_buf());

    hashed_path.display().to_string()
}

/// Returns the dist filesystem path for a static asset public path.
fn static_dist_path(asset_path: &str) -> Result<PathBuf> {
    Ok(Path::new(STATIC_DIST_PATH).join(Path::new(asset_path).strip_prefix("/static/")?))
}

/// Calculate sha256 hash of content.
fn calculate_hash(content: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content);
    hex::encode(hasher.finalize())
}

/// Syncs non-hashed static assets incrementally to avoid copying unchanged files.
fn sync_unhashed_static_dir(
    root: &Path,
    src: &Path,
    dst: &Path,
    expected_files: &mut HashSet<PathBuf>,
) -> Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let src_path = entry.path();
        let file_name = entry.file_name();
        if src == root
            && let Some(name) = file_name.to_str()
            && HASHED_ASSET_DIRS.contains(&name)
        {
            continue;
        }
        let dst_path = dst.join(&file_name);
        if entry.file_type()?.is_dir() {
            sync_unhashed_static_dir(root, &src_path, &dst_path, expected_files)?;
        } else {
            copy_if_changed(&src_path, &dst_path)?;
            expected_files.insert(dst_path);
        }
    }
    Ok(())
}

/// Copies templates and rewrites hashed asset references only when content changes.
fn sync_templates_dir(
    src: &Path,
    dst: &Path,
    sorted_refs: &[(String, String)],
    expected_files: &mut HashSet<PathBuf>,
) -> Result<()> {
    if !src.is_dir() {
        return Ok(());
    }
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            sync_templates_dir(&src_path, &dst_path, sorted_refs, expected_files)?;
        } else if entry.file_type()?.is_file() {
            match fs::read_to_string(&src_path) {
                Ok(content) => {
                    let rewritten = replace_hashed_assets_refs_in_content(&content, sorted_refs);
                    write_if_changed(&dst_path, rewritten.as_bytes())?;
                }
                Err(err) if err.kind() == std::io::ErrorKind::InvalidData => {
                    copy_if_changed(&src_path, &dst_path)?;
                }
                Err(err) => bail!(err),
            }
            expected_files.insert(dst_path);
        }
    }
    Ok(())
}

/// Writes bytes to destination only when missing or different.
fn write_if_changed(dst: &Path, content: &[u8]) -> Result<()> {
    if let Ok(existing) = fs::read(dst)
        && existing == content
    {
        return Ok(());
    }
    if let Some(parent) = dst.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(dst, content)?;
    Ok(())
}

/// Copies file only when destination is missing or differs in size/mtime.
fn copy_if_changed(src: &Path, dst: &Path) -> Result<()> {
    if let (Ok(src_meta), Ok(dst_meta)) = (fs::metadata(src), fs::metadata(dst))
        && src_meta.len() == dst_meta.len()
        && let (Ok(src_mtime), Ok(dst_mtime)) = (src_meta.modified(), dst_meta.modified())
        && src_mtime <= dst_mtime
    {
        return Ok(());
    }
    if let Some(parent) = dst.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::copy(src, dst)?;
    Ok(())
}

/// Removes files under `dir` that are no longer part of `expected_files`.
fn prune_unexpected_files(dir: &Path, expected_files: &HashSet<PathBuf>) -> Result<()> {
    if !dir.is_dir() {
        return Ok(());
    }
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if entry.file_type()?.is_dir() {
            prune_unexpected_files(&path, expected_files)?;
            if fs::read_dir(&path)?.next().is_none() {
                fs::remove_dir(&path)?;
            }
        } else if !expected_files.contains(&path) {
            fs::remove_file(&path)?;
        }
    }
    Ok(())
}

/// Replaces asset path references in a string using sorted manifest pairs.
fn replace_hashed_assets_refs_in_content(
    content: &str,
    sorted_refs: &[(String, String)],
) -> String {
    if !content.contains("/static/") {
        return content.to_string();
    }

    let mut new_content = content.to_string();
    for (plain_path, hashed_path) in sorted_refs {
        if new_content.contains(plain_path.as_str()) {
            new_content = new_content.replace(plain_path, hashed_path);
        }
    }

    new_content
}

/// Returns true when the content includes a supported reference to an asset path.
fn content_contains_asset_ref(content: &str, asset_path: &str) -> bool {
    asset_ref_patterns(asset_path)
        .iter()
        .any(|pattern| content.contains(pattern))
}

/// Returns supported static asset reference patterns for a path.
fn asset_ref_patterns(asset_path: &str) -> Vec<String> {
    vec![
        format!("\"{asset_path}\""),
        format!("'{asset_path}'"),
        format!("`{asset_path}`"),
        format!("url({asset_path})"),
        format!("url(\"{asset_path}\")"),
        format!("url('{asset_path}')"),
    ]
}

/// Returns manifest entries ordered by longest plain path first.
fn asset_refs_by_length(manifest: &AssetsManifest) -> Vec<(String, String)> {
    let mut refs: Vec<(String, String)> = manifest
        .iter()
        .map(|(plain_path, hashed_path)| (plain_path.clone(), hashed_path.clone()))
        .collect();
    refs.sort_by(|(left_path, _), (right_path, _)| {
        right_path
            .len()
            .cmp(&left_path.len())
            .then_with(|| left_path.cmp(right_path))
    });

    refs
}

/// Helper function to run a command.
fn run(program: &str, args: &[&str], current_dir: Option<&Path>) -> Result<()> {
    let mut cmd = new_cmd(program);
    if let Some(dir) = current_dir {
        cmd.current_dir(dir);
    }
    cmd.args(args);

    let output = cmd.output()?;
    if !output.status.success() {
        bail!(
            "\n\n> {cmd:?} (stderr)\n{}\n> {cmd:?} (stdout)\n{}\n",
            String::from_utf8(output.stderr)?,
            String::from_utf8(output.stdout)?
        );
    }

    Ok(())
}

/// Helper function to setup a command based on the target OS.
fn new_cmd(program: &str) -> Command {
    if cfg!(target_os = "windows") {
        let mut cmd = Command::new("cmd");
        cmd.args(["/C", program]);
        cmd
    } else {
        Command::new(program)
    }
}
