use nazar_core::installed_program::InstalledProgram;
use nazar_core::leftover_groups::find_leftover_groups;
use nazar_core::program_locations::{Confidence, LocationRole, find_program_locations};
use nazar_windows::installed_programs::{InstalledProgramsError, read_installed_programs};
use nazar_windows::location_candidates::read_location_candidates;

#[derive(Debug, clap::Args)]
pub struct LocateArguments {
    #[arg(value_name = "NAME", help = "Part of the name of an installed program")]
    pub name: String,
}

pub fn list_programs() -> Result<(), InstalledProgramsError> {
    let inventory = read_installed_programs()?;
    for program in &inventory.programs {
        let publisher = program.publisher.as_deref().unwrap_or("unknown publisher");
        println!("{}  ({publisher})", program.display_name);
    }
    println!("{} programs", inventory.programs.len());
    if inventory.unreadable_entry_count > 0 {
        println!(
            "{} entries could not be read",
            inventory.unreadable_entry_count
        );
    }
    Ok(())
}

pub fn locate_programs(arguments: &LocateArguments) -> Result<(), InstalledProgramsError> {
    let wanted_name = arguments.name.to_lowercase();
    let inventory = read_installed_programs()?;
    let candidates = read_location_candidates();
    let wanted_programs = inventory
        .programs
        .iter()
        .filter(|program| program.display_name.to_lowercase().contains(&wanted_name));
    for program in wanted_programs {
        print_locations(program, &candidates);
    }
    println!(
        "{} locations could not be read while searching",
        candidates.unreadable_location_count
    );
    Ok(())
}

fn print_locations(
    program: &InstalledProgram,
    candidates: &nazar_core::program_locations::LocationCandidates,
) {
    println!("{}", program.display_name);
    for location in find_program_locations(program, candidates) {
        let certainty = match location.confidence {
            Confidence::Certain => "certain",
            Confidence::Likely => "likely",
        };
        println!(
            "  {:<16}{certainty:<9}{}",
            role_label(location.role),
            location.object
        );
    }
    println!();
}

fn role_label(role: LocationRole) -> &'static str {
    match role {
        LocationRole::UninstallEntry => "uninstall entry",
        LocationRole::InstallFolder => "install folder",
        LocationRole::ProgramFolder => "program folder",
        LocationRole::DataFolder => "data folder",
        LocationRole::Shortcut => "shortcut",
        LocationRole::SettingsKey => "settings",
        LocationRole::Autostart => "autostart",
        LocationRole::Service => "service",
        LocationRole::AppPath => "app path",
    }
}

pub fn list_leftovers() -> Result<(), InstalledProgramsError> {
    let inventory = read_installed_programs()?;
    let candidates = read_location_candidates();
    let groups = find_leftover_groups(&inventory.programs, &candidates);
    for group in &groups {
        println!("{}", group.name);
        for location in &group.locations {
            println!("  {:<16}{}", role_label(location.role), location.object);
        }
    }
    println!("{} possible leftovers", groups.len());
    Ok(())
}
