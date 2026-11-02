//! Feature-gated, offline developer validation. Never writes provider records.
use std::path::PathBuf;
use trainer_collection_manager::{
    inventory::{AppraisalIvs, InventoryRepository, PokemonInput},
    reference::Identity,
    reference_store, source_adapter, source_files, strategy,
};

struct TemporaryDatabase(PathBuf);

impl Drop for TemporaryDatabase {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
        for suffix in ["-wal", "-shm"] {
            let mut sidecar = self.0.as_os_str().to_os_string();
            sidecar.push(suffix);
            let _ = std::fs::remove_file(PathBuf::from(sidecar));
        }
    }
}

fn hypothetical_sample(
    package: &reference_store::LocalPackage,
    dex: u16,
) -> Result<serde_json::Value, &'static str> {
    let species = package.catalog["species"]
        .as_array()
        .and_then(|species| {
            species
                .iter()
                .find(|species| species["dexNumber"].as_u64() == Some(u64::from(dex)))
        })
        .ok_or("Requested hypothetical sample Dex is not in the normalized source")?;
    let species_id = species["id"]
        .as_str()
        .ok_or("Sample species has no normalized identifier")?;
    let forms = package.catalog["forms"]
        .as_array()
        .ok_or("Normalized source has no forms")?
        .iter()
        .filter(|form| {
            form["speciesId"].as_str() == Some(species_id)
                && form["name"]
                    .as_str()
                    .is_some_and(|name| name.eq_ignore_ascii_case("Normal"))
        })
        .collect::<Vec<_>>();
    if forms.len() != 1 {
        return Err("Hypothetical sampling requires one explicit Normal form");
    }
    let form_id = forms[0]["id"]
        .as_str()
        .ok_or("Sample Normal form has no normalized identifier")?;
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|_| "Unable to create temporary validation storage")?
        .as_nanos();
    let temporary = TemporaryDatabase(std::env::temp_dir().join(format!(
        "tcm-hypothetical-source-{}-{nonce}.sqlite3",
        std::process::id()
    )));
    // Development-only synthetic individual in a disposable database. The
    // per-user application database and actual collection are never opened.
    let mut repository = InventoryRepository::open(&temporary.0)
        .map_err(|_| "Unable to open temporary hypothetical validation storage")?;
    repository
        .activate_reference(
            &serde_json::to_vec(&package.catalog)
                .map_err(|_| "Unable to encode sample reference catalog")?,
        )
        .map_err(|_| "Unable to activate sample reference in temporary storage")?;
    let ivs = AppraisalIvs {
        attack: 15,
        defense: 15,
        hp: 15,
    };
    let individual = repository
        .create(&PokemonInput {
            display_name: "Hypothetical source validation".into(),
            cp: None,
            ivs: Some(ivs),
        })
        .map_err(|_| "Unable to create temporary hypothetical individual")?;
    let confirmed = repository
        .correct_identity(
            &individual.id,
            None,
            Some(Identity {
                species: species_id.into(),
                form: Some(form_id.into()),
            }),
        )
        .map_err(|_| "Unable to confirm hypothetical sample identity")?;
    let mechanics = strategy::Mechanics {
        version: package.catalog["dataset"]["version"]
            .as_str()
            .ok_or("Sample source has no reference version")?
            .to_owned(),
        sha256: package.source_sha256.clone(),
        cp_multipliers: package
            .cp_multipliers
            .iter()
            .map(|&(level, multiplier)| strategy::CpMultiplier { level, multiplier })
            .collect(),
    };
    let gym = trainer_collection_manager::gym::GymModel::new(package)?
        .map(|model| model.assess(&confirmed.0, None));
    let report = strategy::assess(&[confirmed], Some(&mechanics))?
        .into_iter()
        .next()
        .ok_or("No hypothetical sample assessment was returned")?;
    let output = serde_json::json!({
        "label":"Hypothetical specimen for developer numerical validation only; not an observed user Pokémon",
        "dex":dex,"ivs":ivs,"leagues":report.leagues,"gym":gym
    });
    drop(repository);
    Ok(output)
}

fn main() {
    if let Err(message) = validate() {
        eprintln!("Source validation failed: {message}");
        std::process::exit(1);
    }
}

fn validate() -> Result<(), &'static str> {
    let mut args = std::env::args().skip(1).collect::<Vec<_>>();
    let mut sample_dex = None;
    let mut index = 0;
    while index < args.len() {
        if args[index] == "--sample-dex" {
            if sample_dex.is_some() {
                return Err("Provide --sample-dex only once");
            }
            let dex = args
                .get(index + 1)
                .and_then(|value| value.parse::<u16>().ok())
                .filter(|dex| *dex > 0)
                .ok_or("--sample-dex requires a positive National Dex number")?;
            sample_dex = Some(dex);
            drop(args.drain(index..=index + 1));
        } else {
            index += 1;
        }
    }
    if args.len() < 4 || args.len() > 10 {
        return Err(
            "Usage: validate-source [--sample-dex N] PROVIDER REVISION YYYY-MM-DD FILE [FILE FILE]",
        );
    }
    let paths = args[3..].iter().map(PathBuf::from).collect::<Vec<_>>();
    let input = source_files::read(&args[0], &paths, &args[1], &args[2])?;
    let normalized = source_adapter::normalize(&args[0], &input.bytes, &args[1], &args[2])?;
    let package = reference_store::LocalPackage {
        catalog: normalized.catalog,
        cp_multipliers: normalized.cp_multipliers,
        combat: normalized.combat,
        coverage: normalized.coverage,
        limitations: normalized.limitations,
        source_sha256: reference_store::digest(&input.bytes),
        provider: args[0].clone(),
        artifacts: input.artifacts,
    };
    package
        .validate()
        .map_err(|_| "Normalized package failed native validation")?;
    let mut output = serde_json::json!({
            "provider": package.provider,
            "namespace": package.catalog["dataset"]["datasetId"],
            "version": package.catalog["dataset"]["version"],
            "speciesCount":package.catalog["species"].as_array().map_or(0,Vec::len),
            "formCount":package.catalog["forms"].as_array().map_or(0,Vec::len),
            "multiplierCount":package.cp_multipliers.len(),
            "combatMoveCount":package.combat.as_ref().map_or(0, |combat|combat.moves.len()),
            "combatPoolCount":package.combat.as_ref().map_or(0, |combat|combat.pools.len()),
            "modeledMaximumLevel":package.cp_multipliers.last().map(|entry|entry.0),
            "sourceSha256":package.source_sha256,
            "artifacts":package.artifacts,
            "limitations":package.limitations
    });
    if let Some(dex) = sample_dex {
        output["hypotheticalSample"] = hypothetical_sample(&package, dex)?;
    }
    println!("{output}");
    Ok(())
}
