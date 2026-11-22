use nazar_core::installed_program::InstalledProgram;
use nazar_core::program_locations::{
    Confidence, LocationCandidates, LocationRole, NamedLocation, PathReference, ProgramLocation,
    find_program_locations,
};
use nazar_core::program_name::name_key;
use nazar_core::tracked_object::TrackedObject;

const UNINSTALL_KEY: &str =
    "HKLM\\SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Uninstall\\{0A1B2C3D}";

fn firefox() -> InstalledProgram {
    InstalledProgram {
        registry_key_path: UNINSTALL_KEY.to_owned(),
        display_name: "Mozilla Firefox (x64 en-US)".to_owned(),
        publisher: Some("Mozilla".to_owned()),
        version: Some("131.0".to_owned()),
        install_location: Some("C:\\Program Files\\Mozilla Firefox\\".to_owned()),
        display_icon: Some("C:\\Program Files\\Mozilla Firefox\\firefox.exe,0".to_owned()),
        uninstall_command: None,
        install_date: None,
        estimated_size_in_kilobytes: None,
    }
}

fn named_folder(path: &str, name: &str, publisher_name: Option<&str>) -> NamedLocation {
    NamedLocation {
        object: TrackedObject::file_system_entry(path),
        name: name.to_owned(),
        publisher_name: publisher_name.map(str::to_owned),
        role: LocationRole::DataFolder,
    }
}

fn locations_with(candidates: &LocationCandidates) -> Vec<ProgramLocation> {
    find_program_locations(&firefox(), candidates)
}

fn roles_of(locations: &[ProgramLocation]) -> Vec<LocationRole> {
    locations.iter().map(|location| location.role).collect()
}

#[test]
fn name_key_ignores_versions_architecture_and_punctuation() {
    assert_eq!(name_key("Mozilla Firefox (x64 en-US)"), "mozillafirefox");
    assert_eq!(name_key("7-Zip 24.08 (x64)"), "7zip");
    assert_eq!(name_key("Python 3.12.4 (64-bit)"), "python");
    assert_eq!(name_key("Node.js"), "nodejs");
}

#[test]
fn uninstall_entry_and_declared_install_folder_are_certain() {
    let locations = locations_with(&LocationCandidates::default());

    let expected = vec![
        ProgramLocation {
            object: TrackedObject::file_system_entry("C:\\Program Files\\Mozilla Firefox"),
            role: LocationRole::InstallFolder,
            confidence: Confidence::Certain,
        },
        ProgramLocation {
            object: TrackedObject::registry_key(UNINSTALL_KEY),
            role: LocationRole::UninstallEntry,
            confidence: Confidence::Certain,
        },
    ];
    assert_eq!(locations, expected);
}

#[test]
fn folder_named_after_the_program_under_its_publisher_is_a_likely_match() {
    let candidates = LocationCandidates {
        named_locations: vec![
            named_folder(
                "C:\\Users\\me\\AppData\\Roaming\\Mozilla\\Firefox",
                "Firefox",
                Some("Mozilla"),
            ),
            named_folder(
                "C:\\Users\\me\\AppData\\Roaming\\Opera\\Firefox",
                "Firefox",
                Some("Opera"),
            ),
            named_folder("C:\\ProgramData\\Mozilla Firefox", "Mozilla Firefox", None),
            named_folder(
                "C:\\ProgramData\\Mozilla Thunderbird",
                "Mozilla Thunderbird",
                None,
            ),
        ],
        ..LocationCandidates::default()
    };

    let likely_paths: Vec<String> = locations_with(&candidates)
        .into_iter()
        .filter(|location| location.confidence == Confidence::Likely)
        .map(|location| location.object.to_string())
        .collect();

    assert_eq!(
        likely_paths,
        vec![
            "C:\\ProgramData\\Mozilla Firefox",
            "C:\\Users\\me\\AppData\\Roaming\\Mozilla\\Firefox",
        ]
    );
}

#[test]
fn autostart_entry_pointing_into_the_install_folder_is_certain() {
    let run_key = "HKLM\\SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Run";
    let candidates = LocationCandidates {
        path_references: vec![
            PathReference {
                object: TrackedObject::registry_value(run_key, "Firefox Updater"),
                referenced_text: "\"C:\\Program Files\\Mozilla Firefox\\updater.exe\" /silent"
                    .to_owned(),
                role: LocationRole::Autostart,
            },
            PathReference {
                object: TrackedObject::registry_value(run_key, "Other"),
                referenced_text: "C:\\Program Files\\Other\\other.exe".to_owned(),
                role: LocationRole::Autostart,
            },
        ],
        ..LocationCandidates::default()
    };

    let locations = locations_with(&candidates);

    assert!(roles_of(&locations).contains(&LocationRole::Autostart));
    assert_eq!(locations.len(), 3);
}

#[test]
fn install_folder_is_derived_from_the_icon_when_none_is_declared() {
    let program = InstalledProgram {
        install_location: None,
        display_icon: Some("\"C:\\Tools\\Editor\\editor.exe\",0".to_owned()),
        ..firefox()
    };

    let locations = find_program_locations(&program, &LocationCandidates::default());

    assert_eq!(
        locations[0],
        ProgramLocation {
            object: TrackedObject::file_system_entry("C:\\Tools\\Editor"),
            role: LocationRole::InstallFolder,
            confidence: Confidence::Likely,
        }
    );
}

#[test]
fn shared_windows_folder_is_never_taken_as_an_install_folder() {
    let program = InstalledProgram {
        install_location: None,
        display_icon: Some("C:\\Windows\\Installer\\{0A1B2C3D}\\icon.ico".to_owned()),
        uninstall_command: Some("MsiExec.exe /X{0A1B2C3D}".to_owned()),
        ..firefox()
    };

    let locations = find_program_locations(&program, &LocationCandidates::default());

    assert_eq!(roles_of(&locations), vec![LocationRole::UninstallEntry]);
}

#[test]
fn location_inside_another_matched_location_is_not_listed_twice() {
    let candidates = LocationCandidates {
        named_locations: vec![
            named_folder("C:\\ProgramData\\Mozilla Firefox", "Mozilla Firefox", None),
            named_folder(
                "C:\\ProgramData\\Mozilla Firefox\\Firefox",
                "Firefox",
                Some("Mozilla Firefox"),
            ),
        ],
        ..LocationCandidates::default()
    };

    let data_folders: Vec<String> = locations_with(&candidates)
        .into_iter()
        .filter(|location| location.role == LocationRole::DataFolder)
        .map(|location| location.object.to_string())
        .collect();

    assert_eq!(data_folders, vec!["C:\\ProgramData\\Mozilla Firefox"]);
}
