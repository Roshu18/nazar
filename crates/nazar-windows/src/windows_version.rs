use crate::registry_access::{Hive, RegistryLocation, text_value};

const VERSION_LOCATION: RegistryLocation = RegistryLocation {
    hive: Hive::LocalMachine,
    path: "SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion",
};
const BUILD_NUMBER_VALUE: &str = "CurrentBuildNumber";
const FIRST_BUILD_WITH_MICA: u32 = 22000;

pub fn supports_mica() -> bool {
    read_build_number().is_some_and(|build_number| build_number >= FIRST_BUILD_WITH_MICA)
}

fn read_build_number() -> Option<u32> {
    let version_key = VERSION_LOCATION.open().ok().flatten()?;
    let build_number = text_value(&version_key, BUILD_NUMBER_VALUE)
        .ok()
        .flatten()?;
    build_number.trim().parse().ok()
}
