use std::{env, process::ExitCode};

const HELP: &str = "WorldHunt: an offline hot-and-cold geography guessing game\n\nUsage: worldhunt\n\nOptions:\n  -h, --help     Show this help\n  -V, --version  Show the version";

fn main() -> std::io::Result<ExitCode> {
    let mut arguments = env::args().skip(1);
    if let Some(argument) = arguments.next() {
        match argument.as_str() {
            "-h" | "--help" => {
                println!("{HELP}");
                return Ok(ExitCode::SUCCESS);
            }
            "-V" | "--version" => {
                println!("worldhunt {}", env!("CARGO_PKG_VERSION"));
                return Ok(ExitCode::SUCCESS);
            }
            _ => {
                eprintln!(
                    "worldhunt: unknown argument: {argument}\nTry 'worldhunt --help' for more information."
                );
                return Ok(ExitCode::from(2));
            }
        }
    }
    let world_data = worldhunt::infrastructure::world_data::WorldData::decode_embedded()
        .map_err(std::io::Error::other)?;
    let mut selector = worldhunt::infrastructure::random::RandomTargetSelector::new();
    worldhunt::infrastructure::tui::run(&world_data, &world_data, &mut selector)?;
    Ok(ExitCode::SUCCESS)
}
