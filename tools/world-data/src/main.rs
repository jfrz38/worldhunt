use std::{env, path::PathBuf, process::ExitCode};

fn main() -> ExitCode {
    let mut arguments = env::args_os();
    let _program = arguments.next();

    if arguments.next().as_deref() != Some("validate".as_ref()) || arguments.next().is_some() {
        eprintln!("usage: world-data validate");
        return ExitCode::from(2);
    }

    let repository_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("world-data must remain at tools/world-data")
        .to_path_buf();

    match world_data::validate_repository(&repository_root) {
        Ok(report) => {
            println!(
                "validated {} playable countries, {} source mappings, and {} non-playable source records",
                report.country_count, report.source_mapping_count, report.non_playable_record_count
            );
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("validation failed: {error}");
            ExitCode::FAILURE
        }
    }
}
