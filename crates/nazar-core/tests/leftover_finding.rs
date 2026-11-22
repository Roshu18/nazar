use nazar_core::installed_program::InstalledProgram;
use nazar_core::leftover_groups::find_leftover_groups;
use nazar_core::program_locations::{LocationCandidates, LocationRole, NamedLocation};
use nazar_core::tracked_object::TrackedObject;

fn firefox() -> InstalledProgram {
    InstalledProgram {
        registry_key_path: "HKLM\\SOFTWARE\\Uninstall\\Mozilla Firefox".to_owned(),
        display_name: "Mozilla Firefox (x64 en-US)".to_owned(),
        publisher: Some("Mozilla".to_owned()),
        version: None,
        install_location: Some("C:\\Program Files\\Mozilla Firefox".to_owned()),
        display_icon: None,
        uninstall_command: None,
        install_date: None,
        estimated_size_in_kilobytes: None,
    }
}

fn folder(path: &str, name: &str, role: LocationRole) -> NamedLocation {
    NamedLocation {
        object: TrackedObject::file_system_entry(path),
        name: name.to_owned(),
        publisher_name: None,
        role,
    }
}

fn settings_key(path: &str, name: &str) -> NamedLocation {
    NamedLocation {
        object: TrackedObject::registry_key(path),
        name: name.to_owned(),
        publisher_name: None,
        role: LocationRole::SettingsKey,
    }
}

fn leftover_names(named_locations: Vec<NamedLocation>) -> Vec<String> {
    let candidates = LocationCandidates {
        named_locations,
        ..LocationCandidates::default()
    };
    find_leftover_groups(&[firefox()], &candidates)
        .into_iter()
        .map(|group| group.name)
        .collect()
}

#[test]
fn name_found_in_two_places_without_an_installed_program_is_a_leftover() {
    let names = leftover_names(vec![
        folder(
            "C:\\Users\\me\\AppData\\Roaming\\OldChat",
            "OldChat",
            LocationRole::DataFolder,
        ),
        settings_key("HKCU\\Software\\OldChat", "OldChat"),
    ]);

    assert_eq!(names, vec!["OldChat"]);
}

#[test]
fn folder_left_in_program_files_is_a_leftover_on_its_own() {
    let names = leftover_names(vec![folder(
        "C:\\Program Files\\OldEditor",
        "OldEditor",
        LocationRole::ProgramFolder,
    )]);

    assert_eq!(names, vec!["OldEditor"]);
}

#[test]
fn single_data_folder_is_not_enough_to_call_it_a_leftover() {
    let names = leftover_names(vec![folder(
        "C:\\Users\\me\\AppData\\Local\\SomeCache",
        "SomeCache",
        LocationRole::DataFolder,
    )]);

    assert_eq!(names, Vec::<String>::new());
}

#[test]
fn locations_of_installed_programs_and_their_publishers_are_not_leftovers() {
    let names = leftover_names(vec![
        folder(
            "C:\\Program Files\\Mozilla Firefox",
            "Mozilla Firefox",
            LocationRole::ProgramFolder,
        ),
        folder(
            "C:\\Users\\me\\AppData\\Roaming\\Mozilla",
            "Mozilla",
            LocationRole::DataFolder,
        ),
        settings_key("HKLM\\SOFTWARE\\Mozilla", "Mozilla"),
    ]);

    assert_eq!(names, Vec::<String>::new());
}

#[test]
fn folders_and_keys_windows_uses_are_not_leftovers() {
    let names = leftover_names(vec![
        folder(
            "C:\\Program Files\\Windows Defender",
            "Windows Defender",
            LocationRole::ProgramFolder,
        ),
        folder(
            "C:\\Program Files\\Common Files",
            "Common Files",
            LocationRole::ProgramFolder,
        ),
        folder(
            "C:\\ProgramData\\Microsoft",
            "Microsoft",
            LocationRole::DataFolder,
        ),
        settings_key("HKLM\\SOFTWARE\\Microsoft", "Microsoft"),
    ]);

    assert_eq!(names, Vec::<String>::new());
}
