use nazar_core::change_manifest::Change;
use nazar_core::change_recorder::ChangeRecorder;
use nazar_core::net_effect::NetEffect;
use nazar_core::observation::{Observation, Original, ProcessId, RegistryValueContent, Transition};
use nazar_core::process_lineage::AttributionRule;
use nazar_core::tracked_object::TrackedObject;

const LAUNCHER: ProcessId = ProcessId(100);
const SETUP: ProcessId = ProcessId(200);
const BYSTANDER: ProcessId = ProcessId(300);
const SYSTEM_SERVICES: ProcessId = ProcessId(4);
const RUN_KEY: &str = "HKLM\\SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Run";

fn recorder_with_setup_running() -> ChangeRecorder {
    let mut recorder = ChangeRecorder::new(LAUNCHER);
    recorder.record(Observation::ProcessStarted {
        process: SETUP,
        parent: LAUNCHER,
        image_path: "C:\\Downloads\\setup.exe".to_owned(),
    });
    recorder
}

fn object_changed(actor: ProcessId, object: TrackedObject, transition: Transition) -> Observation {
    Observation::ObjectChanged {
        actor,
        object,
        transition,
    }
}

fn appeared(actor: ProcessId, object: TrackedObject) -> Observation {
    object_changed(actor, object, Transition::Appeared)
}

fn vanished(actor: ProcessId, object: TrackedObject) -> Observation {
    let transition = Transition::Vanished {
        original: Original::NotCaptured,
    };
    object_changed(actor, object, transition)
}

fn value_set(actor: ProcessId, name: &str, previous_bytes: &[u8]) -> Observation {
    let original = Original::RegistryValue(RegistryValueContent {
        value_type: 1,
        bytes: previous_bytes.to_vec(),
    });
    let value = TrackedObject::registry_value(RUN_KEY, name);
    object_changed(actor, value, Transition::Changed { original })
}

fn file(path: &str) -> TrackedObject {
    TrackedObject::file_system_entry(path)
}

fn created(object: TrackedObject, attribution: AttributionRule) -> Change {
    Change {
        object,
        effect: NetEffect::Created,
        attribution,
    }
}

#[test]
fn temporary_file_leaves_no_record() {
    let mut recorder = recorder_with_setup_running();

    recorder.record(appeared(SETUP, file("C:\\Temp\\unpack.tmp")));
    recorder.record(vanished(SETUP, file("C:\\Temp\\unpack.tmp")));

    assert_eq!(recorder.into_changes(), vec![]);
}

#[test]
fn file_written_under_a_temporary_name_is_recorded_once_under_its_final_name() {
    let mut recorder = recorder_with_setup_running();

    recorder.record(appeared(SETUP, file("C:\\App\\app.exe.tmp")));
    recorder.record(Observation::FileSystemEntryRenamed {
        actor: SETUP,
        previous_path: "C:\\App\\app.exe.tmp".to_owned(),
        path: "C:\\App\\app.exe".to_owned(),
    });

    let expected = created(file("C:\\App\\app.exe"), AttributionRule::Descent);
    assert_eq!(recorder.into_changes(), vec![expected]);
}

#[test]
fn renaming_a_directory_moves_the_files_created_inside_it() {
    let mut recorder = recorder_with_setup_running();

    recorder.record(appeared(SETUP, file("C:\\App.new")));
    recorder.record(appeared(SETUP, file("C:\\App.new\\bin\\app.exe")));
    recorder.record(Observation::FileSystemEntryRenamed {
        actor: SETUP,
        previous_path: "C:\\App.new".to_owned(),
        path: "C:\\App".to_owned(),
    });

    let expected = vec![
        created(file("C:\\App"), AttributionRule::Descent),
        created(file("C:\\App\\bin\\app.exe"), AttributionRule::Descent),
    ];
    assert_eq!(recorder.into_changes(), expected);
}

#[test]
fn changes_by_untracked_processes_are_ignored() {
    let mut recorder = recorder_with_setup_running();

    recorder.record(appeared(BYSTANDER, file("C:\\Users\\me\\notes.txt")));

    assert_eq!(recorder.into_changes(), vec![]);
}

#[test]
fn deleting_a_created_registry_key_forgets_the_values_created_under_it() {
    let mut recorder = recorder_with_setup_running();
    let key = TrackedObject::registry_key("HKLM\\SOFTWARE\\Vendor");
    let value = TrackedObject::registry_value("HKLM\\SOFTWARE\\Vendor", "InstallPath");

    recorder.record(appeared(SETUP, key.clone()));
    recorder.record(appeared(SETUP, value));
    recorder.record(vanished(SETUP, key));

    assert_eq!(recorder.into_changes(), vec![]);
}

#[test]
fn modified_registry_value_keeps_the_content_it_had_before_the_program() {
    let mut recorder = recorder_with_setup_running();

    recorder.record(value_set(SETUP, "Updater", b"before"));
    recorder.record(value_set(SETUP, "Updater", b"intermediate"));

    let changes = recorder.into_changes();
    let expected_original = Original::RegistryValue(RegistryValueContent {
        value_type: 1,
        bytes: b"before".to_vec(),
    });
    assert_eq!(
        changes[0].effect,
        NetEffect::Modified {
            original: expected_original
        }
    );
}

#[test]
fn service_started_from_an_installed_executable_is_attributed_to_the_program() {
    let mut recorder = recorder_with_setup_running();
    recorder.record(appeared(SETUP, file("C:\\App\\service.exe")));
    recorder.record(Observation::ProcessStopped { process: SETUP });

    recorder.record(Observation::ProcessStarted {
        process: BYSTANDER,
        parent: SYSTEM_SERVICES,
        image_path: "C:\\App\\service.exe".to_owned(),
    });
    recorder.record(appeared(BYSTANDER, file("C:\\ProgramData\\App\\state.db")));

    let expected = vec![
        created(file("C:\\App\\service.exe"), AttributionRule::Descent),
        created(
            file("C:\\ProgramData\\App\\state.db"),
            AttributionRule::DroppedImage,
        ),
    ];
    assert_eq!(recorder.into_changes(), expected);
    assert!(recorder_is_still_watching_after_setup_exit());
}

fn recorder_is_still_watching_after_setup_exit() -> bool {
    let mut recorder = recorder_with_setup_running();
    recorder.record(appeared(SETUP, file("C:\\App\\service.exe")));
    recorder.record(Observation::ProcessStopped { process: SETUP });
    recorder.record(Observation::ProcessStarted {
        process: BYSTANDER,
        parent: SYSTEM_SERVICES,
        image_path: "C:\\App\\service.exe".to_owned(),
    });
    recorder.has_live_tracked_process()
}

#[test]
fn value_written_under_a_key_the_program_created_counts_as_created() {
    let mut recorder = recorder_with_setup_running();
    let key = TrackedObject::registry_key("HKLM\\SOFTWARE\\Vendor");
    let value = TrackedObject::registry_value("HKLM\\SOFTWARE\\Vendor", "InstallPath");
    let written_without_known_original = Transition::Changed {
        original: Original::NotCaptured,
    };

    recorder.record(appeared(SETUP, key.clone()));
    recorder.record(object_changed(
        SETUP,
        value.clone(),
        written_without_known_original,
    ));

    let expected = vec![
        created(key, AttributionRule::Descent),
        created(value, AttributionRule::Descent),
    ];
    assert_eq!(recorder.into_changes(), expected);
}

#[test]
fn value_written_under_an_older_key_is_modified_until_its_original_is_known() {
    let mut recorder = recorder_with_setup_running();
    let value = TrackedObject::registry_value(RUN_KEY, "Updater");
    let written_without_known_original = Transition::Changed {
        original: Original::NotCaptured,
    };

    recorder.record(object_changed(SETUP, value, written_without_known_original));

    let changes = recorder.into_changes();
    assert_eq!(
        changes[0].effect,
        NetEffect::Modified {
            original: Original::NotCaptured
        }
    );
}
