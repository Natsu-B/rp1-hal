use std::{
    collections::BTreeMap,
    env, fs,
    path::{Path, PathBuf},
    process,
};

use anyhow::{Context, Result, bail};
use rp1_regdb::{gen_rust, load, model, validate};

fn main() {
    if let Err(error) = run() {
        eprintln!("error: {error:#}");
        process::exit(1);
    }
}

fn run() -> Result<()> {
    let mut args = env::args().skip(1);
    let command = args.next().unwrap_or_else(|| "help".into());
    match command.as_str() {
        "validate" => {
            let device = args
                .next()
                .map(PathBuf::from)
                .unwrap_or_else(default_device);
            let db = load::load_database(&device)?;
            validate_database(&db)
        }
        "generate" => {
            let check = args.next().is_some_and(|argument| argument == "--check");
            if !check && args.len() > 2 {
                bail!("usage: rp1-regdb generate [--check] [device TOML] [PAC source]");
            }
            let remaining = if check {
                args.collect::<Vec<_>>()
            } else {
                env::args().skip(2).collect::<Vec<_>>()
            };
            let device = remaining
                .first()
                .map(PathBuf::from)
                .unwrap_or_else(default_device);
            let pac_src = remaining
                .get(1)
                .map(PathBuf::from)
                .unwrap_or_else(default_pac_src);
            let db = load::load_database(&device)?;
            validate_database(&db)?;
            if check {
                check_generation(&db, &pac_src)
            } else {
                gen_rust::generate(&db, &pac_src)?;
                gen_rust::write_coverage(&db, &db.root.join("COVERAGE.generated.md"))
            }
        }
        _ => bail!("usage: rp1-regdb <validate|generate> ..."),
    }
}

fn default_device() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data/device.toml")
}

fn default_pac_src() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../rp1-pac/src")
}

fn validate_database(db: &model::Database) -> Result<()> {
    match validate::validate(db) {
        Ok(()) => Ok(()),
        Err(errors) => bail!("validation failed:\n{}", errors.join("\n")),
    }
}

fn check_generation(db: &model::Database, expected: &Path) -> Result<()> {
    let temporary = tempfile::tempdir()?;
    let actual = temporary.path().join("rp1-pac/src");
    fs::create_dir_all(&actual)?;
    gen_rust::generate(db, &actual)?;

    let expected_files = generated_files(expected)?;
    let actual_files = generated_files(&actual)?;
    if expected_files.keys().ne(actual_files.keys()) {
        bail!(
            "generated file set differs:\nexpected: {:?}\nactual: {:?}",
            expected_files.keys().collect::<Vec<_>>(),
            actual_files.keys().collect::<Vec<_>>()
        );
    }
    let differences: Vec<_> = expected_files
        .iter()
        .filter_map(|(path, expected_bytes)| {
            (actual_files.get(path) != Some(expected_bytes)).then_some(path)
        })
        .collect();
    if !differences.is_empty() {
        bail!("generated files are stale: {differences:?}");
    }
    Ok(())
}

fn generated_files(root: &Path) -> Result<BTreeMap<PathBuf, Vec<u8>>> {
    let mut files = BTreeMap::new();
    let pac_root = root.parent().context("PAC src directory has no parent")?;
    files.insert(PathBuf::from("device.x"), read(&pac_root.join("device.x"))?);
    for name in ["peripherals.rs", "interrupts.rs", "metadata.rs"] {
        let path = root.join(name);
        files.insert(PathBuf::from(name), read(&path)?);
    }
    let generated = root.join("generated");
    let entries =
        fs::read_dir(&generated).with_context(|| format!("reading {}", generated.display()))?;
    for entry in entries {
        let entry = entry?;
        if entry.file_type()?.is_file() {
            files.insert(
                PathBuf::from("generated").join(entry.file_name()),
                read(&entry.path())?,
            );
        }
    }
    Ok(files)
}

fn read(path: &Path) -> Result<Vec<u8>> {
    fs::read(path).with_context(|| format!("reading {}", path.display()))
}
