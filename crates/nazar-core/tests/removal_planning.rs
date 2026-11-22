use nazar_core::change_manifest::Change;
use nazar_core::net_effect::NetEffect;
use nazar_core::observation::{Original, RegistryValueContent};
use nazar_core::process_lineage::AttributionRule;
use nazar_core::removal_plan::{RemovalStep, ReviewReason, plan_removal};
use nazar_core::tracked_object::TrackedObject;

fn change(object: TrackedObject, effect: NetEffect) -> Change {
    Change {
        object,
        effect,
        attribution: AttributionRule::Descent,
    }
}

fn created(object: TrackedObject) -> Change {
    change(object, NetEffect::Created)
}

fn delete_file(path: &str) -> RemovalStep {
    RemovalStep::DeleteFileSystemEntry {
        path: path.to_owned(),
    }
}

#[test]
fn files_are_deleted_before_the_directories_that_hold_them() {
    let changes = vec![
        created(TrackedObject::file_system_entry("C:\\App")),
        created(TrackedObject::file_system_entry("C:\\App\\bin\\app.exe")),
        created(TrackedObject::file_system_entry("C:\\App\\bin")),
    ];

    let expected = vec![
        delete_file("C:\\App\\bin\\app.exe"),
        delete_file("C:\\App\\bin"),
        delete_file("C:\\App"),
    ];
    assert_eq!(plan_removal(&changes), expected);
}

#[test]
fn registry_values_are_deleted_before_registry_keys() {
    let changes = vec![
        created(TrackedObject::registry_key("HKLM\\SOFTWARE\\Vendor")),
        created(TrackedObject::registry_value(
            "HKLM\\SOFTWARE\\Vendor",
            "Path",
        )),
    ];

    let expected = vec![
        RemovalStep::DeleteRegistryValue {
            key_path: "HKLM\\SOFTWARE\\Vendor".to_owned(),
            name: "Path".to_owned(),
        },
        RemovalStep::DeleteRegistryKey {
            path: "HKLM\\SOFTWARE\\Vendor".to_owned(),
        },
    ];
    assert_eq!(plan_removal(&changes), expected);
}

#[test]
fn modified_registry_value_is_restored_to_its_original_content() {
    let original = RegistryValueContent {
        value_type: 4,
        bytes: vec![1, 0, 0, 0],
    };
    let effect = NetEffect::Modified {
        original: Original::RegistryValue(original.clone()),
    };
    let changes = vec![change(
        TrackedObject::registry_value("HKLM\\SOFTWARE\\Policy", "Enabled"),
        effect,
    )];

    let expected = vec![RemovalStep::RestoreRegistryValue {
        key_path: "HKLM\\SOFTWARE\\Policy".to_owned(),
        name: "Enabled".to_owned(),
        original,
    }];
    assert_eq!(plan_removal(&changes), expected);
}

#[test]
fn change_without_a_captured_original_is_left_for_manual_review() {
    let hosts_file = TrackedObject::file_system_entry("C:\\Windows\\System32\\drivers\\etc\\hosts");
    let effect = NetEffect::Modified {
        original: Original::NotCaptured,
    };
    let changes = vec![change(hosts_file.clone(), effect)];

    let expected = vec![RemovalStep::ReviewManually {
        object: hosts_file,
        reason: ReviewReason::OriginalNotCaptured,
    }];
    assert_eq!(plan_removal(&changes), expected);
}
