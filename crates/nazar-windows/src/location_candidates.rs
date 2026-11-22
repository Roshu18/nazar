use nazar_core::program_locations::LocationCandidates;

use crate::folder_candidates::collect_folder_candidates;
use crate::registry_candidates::collect_registry_candidates;

pub fn read_location_candidates() -> LocationCandidates {
    let mut candidates = LocationCandidates::default();
    collect_folder_candidates(&mut candidates);
    collect_registry_candidates(&mut candidates);
    candidates
}
