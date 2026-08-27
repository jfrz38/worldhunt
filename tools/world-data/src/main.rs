use std::{env, path::PathBuf, process::ExitCode};

fn main() -> ExitCode {
    let mut arguments = env::args_os();
    let _program = arguments.next();

    let command = arguments.next();
    let check = arguments.next().as_deref() == Some("--check".as_ref());
    if arguments.next().is_some() {
        eprintln!("usage: world-data <validate|generate|preview> [--check]");
        return ExitCode::from(2);
    }

    let repository_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("world-data must remain at tools/world-data")
        .to_path_buf();

    let result = match command.as_deref() {
        Some(command) if command == "validate" && !check => world_data::validate_repository(&repository_root)
            .map(|report| format!("validated {} playable countries, {} source mappings, and {} non-playable source records", report.country_count, report.source_mapping_count, report.non_playable_record_count)),
        Some(command) if command == "generate" => world_data::generate_asset(&repository_root, check),
        Some(command) if command == "preview" && !check => world_data::preview_asset(&repository_root),
        _ => Err("usage: world-data <validate|generate|preview> [--check]".to_owned()),
    };

    match result {
        Ok(message) => {
            println!("{message}");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("validation failed: {error}");
            ExitCode::FAILURE
        }
    }
}
