use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, Mutex, PoisonError};

use nazar_core::installed_program::InstalledProgram;
use nazar_core::leftover_groups::{LeftoverGroup, find_leftover_groups};
use nazar_core::program_locations::LocationCandidates;
use nazar_core::uninstall_launch::uninstall_launch_of;
use nazar_windows::installed_programs::read_installed_programs;
use nazar_windows::location_candidates::read_location_candidates;
use nazar_windows::shell_launch::{Elevation, ShellLaunch, WindowVisibility, run_and_wait};
use tauri::State;

use crate::command_failure::CommandFailure;
use crate::program_view::{
    LEFTOVER_ID_PREFIX, ProgramDetails, ProgramOverview, details_of, leftover_details_of,
    leftover_id_of, leftover_overview_of, overview_of,
};

struct Catalogue {
    programs: Vec<InstalledProgram>,
    candidates: LocationCandidates,
    leftovers: Vec<LeftoverGroup>,
}

#[derive(Default)]
pub struct ProgramCatalogue {
    latest: Mutex<Option<Arc<Catalogue>>>,
    uninstalled_programs: Mutex<HashMap<String, InstalledProgram>>,
}

impl ProgramCatalogue {
    fn remember(&self, catalogue: Arc<Catalogue>) {
        let mut latest = self.latest.lock().unwrap_or_else(PoisonError::into_inner);
        *latest = Some(catalogue);
    }

    fn latest(&self) -> Option<Arc<Catalogue>> {
        self.latest
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }

    fn remember_uninstalled(&self, program: InstalledProgram) {
        self.uninstalled_programs
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .insert(program.registry_key_path.clone(), program);
    }

    fn uninstalled(&self, id: &str) -> Option<InstalledProgram> {
        self.uninstalled_programs
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .get(id)
            .cloned()
    }
}

#[tauri::command]
pub async fn list_installed_programs(
    catalogue: State<'_, ProgramCatalogue>,
) -> Result<Vec<ProgramOverview>, CommandFailure> {
    let read = tauri::async_runtime::spawn_blocking(read_catalogue).await;
    let latest = Arc::new(read.map_err(|_| CommandFailure::WorkInterrupted)??);
    catalogue.remember(Arc::clone(&latest));
    Ok(latest
        .programs
        .iter()
        .map(|program| overview_of(program, &latest.candidates))
        .collect())
}

#[tauri::command]
#[expect(
    clippy::needless_pass_by_value,
    reason = "Tauri hands managed state to commands by value"
)]
pub fn read_program(
    id: &str,
    catalogue: State<'_, ProgramCatalogue>,
) -> Result<ProgramDetails, CommandFailure> {
    let latest = catalogue.latest().ok_or(CommandFailure::ProgramNotListed)?;
    if id.starts_with(LEFTOVER_ID_PREFIX) {
        let group = latest
            .leftovers
            .iter()
            .find(|group| leftover_id_of(group) == id)
            .ok_or(CommandFailure::ProgramNotListed)?;
        return Ok(leftover_details_of(group, &latest.candidates));
    }
    let installed_program = latest
        .programs
        .iter()
        .find(|program| program.registry_key_path == id);
    if let Some(program) = installed_program {
        return Ok(details_of(program, &latest.candidates, true));
    }
    let program = catalogue
        .uninstalled(id)
        .ok_or(CommandFailure::ProgramNotListed)?;
    Ok(details_of(&program, &latest.candidates, false))
}

#[tauri::command]
#[expect(
    clippy::needless_pass_by_value,
    reason = "Tauri hands managed state to commands by value"
)]
pub fn list_leftovers(catalogue: State<'_, ProgramCatalogue>) -> Vec<ProgramOverview> {
    catalogue.latest().map_or_else(Vec::new, |latest| {
        latest.leftovers.iter().map(leftover_overview_of).collect()
    })
}

#[tauri::command]
pub async fn uninstall_program(
    id: String,
    catalogue: State<'_, ProgramCatalogue>,
) -> Result<(), CommandFailure> {
    let latest = catalogue.latest().ok_or(CommandFailure::ProgramNotListed)?;
    let program = latest
        .programs
        .iter()
        .find(|program| program.registry_key_path == id)
        .cloned()
        .ok_or(CommandFailure::ProgramNotListed)?;
    catalogue.remember_uninstalled(program.clone());
    let uninstalling = tauri::async_runtime::spawn_blocking(move || run_uninstaller(&program));
    uninstalling
        .await
        .map_err(|_| CommandFailure::WorkInterrupted)?
}

fn read_catalogue() -> Result<Catalogue, CommandFailure> {
    let programs = read_installed_programs()?.programs;
    let candidates = read_location_candidates();
    let leftovers = find_leftover_groups(&programs, &candidates);
    Ok(Catalogue {
        programs,
        candidates,
        leftovers,
    })
}

fn run_uninstaller(program: &InstalledProgram) -> Result<(), CommandFailure> {
    let launch = uninstall_launch_of(program).ok_or(CommandFailure::NoUninstaller)?;
    run_and_wait(&ShellLaunch {
        executable: Path::new(&launch.executable),
        parameters: &launch.parameters,
        elevation: Elevation::AsTheUser,
        window: WindowVisibility::Visible,
    })?;
    Ok(())
}
