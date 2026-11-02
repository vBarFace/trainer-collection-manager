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
        .install_reference(package.clone())
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
    // Explicit developer-only hypothetical starting facts, never inferred from
    // CP or written into the user's actual app-data database.
    let details = repository
        .individual_details(&individual.id)
        .map_err(|_| "Unable to read hypothetical details")?;
    let first = gym.as_ref().and_then(|g| g.builds.first());
    repository
        .save_individual_details(
            &individual.id,
            details
                .observation_revision
                .parse()
                .map_err(|_| "Invalid hypothetical revision")?,
            &details.evidence_token,
            trainer_collection_manager::individual_details::IndividualDetails {
                level: Some(39.5),
                level_evidence: Some(
                    "Explicit hypothetical developer input, not an observed level".into(),
                ),
                fast_move: first.map(|b| b.fast_move.clone()),
                charged_move: first.map(|b| b.charged_move.clone()),
                ..Default::default()
            },
        )
        .map_err(|_| "Unable to set hypothetical details")?;
    repository
        .save_preparation_resources(
            &individual.id,
            0,
            trainer_collection_manager::preparation_resources::ResourceValues {
                candy: Some(1_000_000),
                candy_xl: Some(1_000_000),
                stardust: Some(10_000_000),
                maximum_level: Some(50.0),
                normal_costs_confirmed: true,
                selected_path_id: None,
                route_intent: None,
            },
        )
        .map_err(|_| "Unable to set hypothetical allocations")?;
    let native = repository
        .collection_strategy()
        .map_err(|_| "Unable to calculate hypothetical preparation")?;
    let preparation = native.reports.first().and_then(|r| r.preparation.clone());
    // Exercise real bounded roadmap persistence, not only direct domain logic.
    repository
        .generate_roadmap(&individual.id, &native.context)
        .map_err(|_| "Hypothetical roadmap exceeded persistence bounds or failed")?;
    let report = strategy::assess(&[confirmed], Some(&mechanics))?
        .into_iter()
        .next()
        .ok_or("No hypothetical sample assessment was returned")?;
    let output = serde_json::json!({
        "label":"Hypothetical specimen for developer numerical validation only; not an observed user Pokémon",
        "dex":dex,"ivs":ivs,"leagues":report.leagues,"gym":gym,
        "preparationInput":"Hypothetical level39.5, ordinary cost profile and large allocations, not user observations",
        "preparation":preparation,"roles":native.reports.first().map(|r|&r.roles)
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
    if args.len() < 4 || args.len() > 14 {
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
        preparation: normalized.preparation,
        trainer_combat: normalized.trainer_combat,
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
            "powerUpIntervalCount":package.preparation.as_ref().map_or(0,|p|p.power_up_costs.len()),
            "evolutionBranchCount":package.preparation.as_ref().map_or(0,|p|p.evolutions.len()),
            "trainerMoveCount":package.trainer_combat.as_ref().map_or(0,|p|p.moves.len()),
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
