use nazar_core::installed_program::InstalledProgram;
use nazar_core::uninstall_launch::uninstall_launch_of;

fn program_uninstalled_with(command: Option<&str>) -> InstalledProgram {
    InstalledProgram {
        registry_key_path: "HKLM\\SOFTWARE\\Uninstall\\Example".to_owned(),
        display_name: "Example".to_owned(),
        publisher: None,
        version: None,
        install_location: None,
        display_icon: None,
        uninstall_command: command.map(str::to_owned),
        install_date: None,
        estimated_size_in_kilobytes: None,
    }
}

fn executable_and_parameters_for(command: &str) -> Option<(String, String)> {
    uninstall_launch_of(&program_uninstalled_with(Some(command)))
        .map(|launch| (launch.executable, launch.parameters))
}

fn pair(executable: &str, parameters: &str) -> (String, String) {
    (executable.to_owned(), parameters.to_owned())
}

#[test]
fn quoted_uninstaller_is_separated_from_its_parameters() {
    let launch =
        executable_and_parameters_for("\"C:\\Program Files\\Example\\unins000.exe\" /SILENT");

    let expected = pair("C:\\Program Files\\Example\\unins000.exe", "/SILENT");
    assert_eq!(launch, Some(expected));
}

#[test]
fn unquoted_uninstaller_with_spaces_in_its_path_ends_at_the_extension() {
    let launch = executable_and_parameters_for("C:\\Program Files\\Example\\Uninstall.EXE /S");

    let expected = pair("C:\\Program Files\\Example\\Uninstall.EXE", "/S");
    assert_eq!(launch, Some(expected));
}

#[test]
fn windows_installer_is_asked_to_remove_instead_of_maintain() {
    let launch = executable_and_parameters_for("MsiExec.exe /I{0A1B2C3D-0000}");

    assert_eq!(launch, Some(pair("MsiExec.exe", "/X{0A1B2C3D-0000}")));
}

#[test]
fn program_without_an_uninstaller_has_nothing_to_launch() {
    assert_eq!(uninstall_launch_of(&program_uninstalled_with(None)), None);
    assert_eq!(
        uninstall_launch_of(&program_uninstalled_with(Some("  "))),
        None
    );
}
