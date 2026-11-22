use nazar_core::location_removal::{in_removal_order, is_protected};
use nazar_core::tracked_object::TrackedObject;

fn folder(path: &str) -> TrackedObject {
    TrackedObject::file_system_entry(path)
}

fn key(path: &str) -> TrackedObject {
    TrackedObject::registry_key(path)
}

#[test]
fn folders_that_belong_to_a_program_can_be_removed() {
    assert!(!is_protected(&folder("C:\\Program Files\\PhotoStudio")));
    assert!(!is_protected(&folder(
        "C:\\Users\\me\\AppData\\Roaming\\PhotoStudio"
    )));
    assert!(!is_protected(&folder("D:\\Games\\Chess")));
}

#[test]
fn folders_that_windows_or_every_program_share_are_protected() {
    assert!(is_protected(&folder("C:")));
    assert!(is_protected(&folder("C:\\Windows")));
    assert!(is_protected(&folder("C:\\Windows\\System32\\drivers")));
    assert!(is_protected(&folder("C:\\Program Files")));
    assert!(is_protected(&folder(
        "C:\\Program Files (x86)\\Common Files"
    )));
    assert!(is_protected(&folder("C:\\ProgramData\\Microsoft")));
    assert!(is_protected(&folder("C:\\Users\\me")));
    assert!(is_protected(&folder("C:\\Users\\me\\AppData\\Roaming")));
}

#[test]
fn registry_keys_that_belong_to_a_program_can_be_removed() {
    assert!(!is_protected(&key("HKLM\\SOFTWARE\\PhotoStudio")));
    assert!(!is_protected(&key(
        "HKLM\\SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Uninstall\\PhotoStudio"
    )));
    assert!(!is_protected(&key(
        "HKLM\\SYSTEM\\CurrentControlSet\\Services\\PhotoStudioUpdater"
    )));
}

#[test]
fn registry_keys_that_hold_other_programs_are_protected() {
    assert!(is_protected(&key("HKLM")));
    assert!(is_protected(&key("HKLM\\SOFTWARE")));
    assert!(is_protected(&key("HKLM\\Vendor")));
    assert!(is_protected(&key("HKCU\\Software\\Microsoft")));
    assert!(is_protected(&key(
        "HKLM\\SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Run"
    )));
    assert!(is_protected(&key(
        "HKLM\\SYSTEM\\CurrentControlSet\\Services"
    )));
}

#[test]
fn registry_values_are_never_protected_by_their_position() {
    let autostart = TrackedObject::registry_value(
        "HKLM\\SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Run",
        "PhotoStudio Updater",
    );

    assert!(!is_protected(&autostart));
}

#[test]
fn deeper_locations_are_removed_before_the_folders_that_hold_them() {
    let ordered = in_removal_order(vec![
        folder("C:\\App"),
        folder("C:\\App\\bin\\tool.exe"),
        folder("C:\\App\\bin"),
    ]);

    assert_eq!(
        ordered,
        vec![
            folder("C:\\App\\bin\\tool.exe"),
            folder("C:\\App\\bin"),
            folder("C:\\App"),
        ]
    );
}
