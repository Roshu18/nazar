use std::collections::{BTreeMap, BTreeSet};

use crate::installed_program::InstalledProgram;
use crate::program_identity::ProgramIdentity;
use crate::program_locations::{
    Confidence, LocationCandidates, LocationRole, NamedLocation, ProgramLocation,
};
use crate::program_name::{is_distinctive, name_key};

const MINIMUM_CORROBORATING_LOCATION_COUNT: usize = 2;
const STORE_PACKAGES_FOLDER: &str = "packages";
const PACKAGE_PUBLISHER_SEPARATOR: char = '_';
const PACKAGE_NAME_SEPARATOR: char = '.';
const NAMES_WINDOWS_USES: &[&str] = &[
    "accessibility",
    "accessories",
    "acer",
    "administrativetools",
    "amd",
    "appdatalow",
    "applicationdata",
    "applicationverifier",
    "asus",
    "ati",
    "atitechnologies",
    "classes",
    "clients",
    "comms",
    "common",
    "commonfiles",
    "connecteddevicesplatform",
    "crashdumps",
    "cvsm",
    "d3dscache",
    "dbg",
    "defaultuserenvironment",
    "dell",
    "designer",
    "desktop",
    "documents",
    "dotnet",
    "hewlettpackard",
    "history",
    "hpcommrecovery",
    "inetcache",
    "installshieldinstallationinformation",
    "intel",
    "internetexplorer",
    "khronos",
    "lenovo",
    "maintenance",
    "microsoftnet",
    "modifiablewindowsapps",
    "msbuild",
    "nazar",
    "nvidia",
    "nvidiacorporation",
    "odbc",
    "oem",
    "onedrive",
    "openssh",
    "packagecache",
    "packages",
    "partner",
    "policies",
    "powershell",
    "publishers",
    "realtek",
    "referenceassemblies",
    "regid",
    "registeredapplications",
    "softwaredistribution",
    "ssl",
    "system",
    "ssh",
    "startmenu",
    "startup",
    "systemtools",
    "temp",
    "uninstallinformation",
    "virtualstore",
    "wow6432node",
];
const NAME_STARTS_WINDOWS_USES: [&str; 2] = ["windows", "microsoft"];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LeftoverGroup {
    pub key: String,
    pub name: String,
    pub locations: Vec<ProgramLocation>,
}

pub fn find_leftover_groups(
    programs: &[InstalledProgram],
    candidates: &LocationCandidates,
) -> Vec<LeftoverGroup> {
    let identities: Vec<ProgramIdentity> = programs.iter().map(ProgramIdentity::of).collect();
    let store_package_names = store_package_names_in(candidates);
    let mut groups: BTreeMap<String, LeftoverGroup> = BTreeMap::new();
    let top_level_locations = candidates
        .named_locations
        .iter()
        .filter(|location| location.publisher_name.is_none());
    for location in top_level_locations {
        let key = name_key(&location.name);
        let is_claimed = belongs_to_windows(&key)
            || store_package_names.contains(&key)
            || belongs_to_a_program(&identities, location);
        if is_claimed {
            continue;
        }
        let group = groups.entry(key.clone()).or_insert_with(|| LeftoverGroup {
            key,
            name: location.name.clone(),
            locations: Vec::new(),
        });
        group.locations.push(ProgramLocation {
            object: location.object.clone(),
            role: location.role,
            confidence: Confidence::Likely,
        });
    }
    groups.into_values().filter(is_corroborated).collect()
}

fn belongs_to_windows(key: &str) -> bool {
    !is_distinctive(key)
        || NAMES_WINDOWS_USES.contains(&key)
        || NAME_STARTS_WINDOWS_USES
            .iter()
            .any(|name_start| key.starts_with(name_start))
}

fn belongs_to_a_program(identities: &[ProgramIdentity], location: &NamedLocation) -> bool {
    let path = location.object.container_path();
    identities.iter().any(|identity| {
        identity.has_name_starting_like(&location.name)
            || identity.is_published_by(&location.name)
            || identity.is_installed_within(path)
            || identity.has_inside_install_folder(&format!("{path}\\"))
    })
}

fn store_package_names_in(candidates: &LocationCandidates) -> BTreeSet<String> {
    candidates
        .named_locations
        .iter()
        .filter(|location| {
            location
                .publisher_name
                .as_deref()
                .is_some_and(|folder| folder.eq_ignore_ascii_case(STORE_PACKAGES_FOLDER))
        })
        .flat_map(|location| names_in_package(&location.name))
        .collect()
}

fn names_in_package(package_folder_name: &str) -> Vec<String> {
    let package_name = package_folder_name
        .split(PACKAGE_PUBLISHER_SEPARATOR)
        .next()
        .unwrap_or(package_folder_name);
    package_name
        .split(PACKAGE_NAME_SEPARATOR)
        .chain([package_name])
        .map(name_key)
        .filter(|key| is_distinctive(key))
        .collect()
}

fn is_corroborated(group: &LeftoverGroup) -> bool {
    let has_program_folder = group
        .locations
        .iter()
        .any(|location| location.role == LocationRole::ProgramFolder);
    has_program_folder || group.locations.len() >= MINIMUM_CORROBORATING_LOCATION_COUNT
}
