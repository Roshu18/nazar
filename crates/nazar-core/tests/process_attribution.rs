use nazar_core::observation::ProcessId;
use nazar_core::process_lineage::{AttributionRule, ProcessLineage};

const LAUNCHER: ProcessId = ProcessId(100);
const SETUP: ProcessId = ProcessId(200);
const HELPER: ProcessId = ProcessId(201);
const BYSTANDER: ProcessId = ProcessId(300);
const SYSTEM_SERVICES: ProcessId = ProcessId(4);
const INSTALLER_SERVICE: ProcessId = ProcessId(500);
const INSTALLER_CLIENT: ProcessId = ProcessId(202);

fn lineage_with_setup_running() -> ProcessLineage {
    let mut lineage = ProcessLineage::new(LAUNCHER);
    lineage.note_process_start(SETUP, LAUNCHER, "C:\\Downloads\\setup.exe");
    lineage
}

#[test]
fn launched_program_is_tracked_but_the_launcher_is_not() {
    let lineage = lineage_with_setup_running();

    assert_eq!(
        lineage.attribution_of(SETUP),
        Some(AttributionRule::Descent)
    );
    assert_eq!(lineage.attribution_of(LAUNCHER), None);
}

#[test]
fn descendants_of_the_program_are_tracked() {
    let mut lineage = lineage_with_setup_running();

    lineage.note_process_start(HELPER, SETUP, "C:\\Temp\\unpack.exe");

    assert_eq!(
        lineage.attribution_of(HELPER),
        Some(AttributionRule::Descent)
    );
}

#[test]
fn unrelated_process_is_not_tracked() {
    let mut lineage = lineage_with_setup_running();

    lineage.note_process_start(
        BYSTANDER,
        SYSTEM_SERVICES,
        "C:\\Windows\\System32\\svchost.exe",
    );

    assert_eq!(lineage.attribution_of(BYSTANDER), None);
}

#[test]
fn process_started_from_a_dropped_image_is_tracked() {
    let mut lineage = lineage_with_setup_running();
    lineage.note_dropped_image("C:\\Program Files\\App\\Service.exe");

    lineage.note_process_start(
        BYSTANDER,
        SYSTEM_SERVICES,
        "c:\\program files\\app\\service.exe",
    );

    assert_eq!(
        lineage.attribution_of(BYSTANDER),
        Some(AttributionRule::DroppedImage)
    );
}

#[test]
fn installer_service_is_attributed_only_while_a_tracked_client_is_alive() {
    let mut lineage = lineage_with_setup_running();
    lineage.note_running_process(INSTALLER_SERVICE, "msiexec.exe");
    assert_eq!(lineage.attribution_of(INSTALLER_SERVICE), None);

    lineage.note_process_start(
        INSTALLER_CLIENT,
        SETUP,
        "C:\\Windows\\System32\\msiexec.exe",
    );
    assert_eq!(
        lineage.attribution_of(INSTALLER_SERVICE),
        Some(AttributionRule::InstallerService)
    );

    lineage.note_process_stop(INSTALLER_CLIENT);
    assert_eq!(lineage.attribution_of(INSTALLER_SERVICE), None);
}

#[test]
fn stopped_process_is_no_longer_tracked() {
    let mut lineage = lineage_with_setup_running();

    lineage.note_process_stop(SETUP);

    assert_eq!(lineage.attribution_of(SETUP), None);
    assert!(!lineage.has_live_tracked_process());
}
