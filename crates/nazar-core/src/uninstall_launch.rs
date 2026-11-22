use crate::installed_program::InstalledProgram;

const QUOTE: char = '"';
const EXECUTABLE_EXTENSION: &str = ".exe";
const WINDOWS_INSTALLER_NAME: &str = "msiexec";
const INSTALLER_MAINTENANCE_SWITCH: &str = "/i{";
const INSTALLER_REMOVAL_SWITCH: &str = "/X{";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UninstallLaunch {
    pub executable: String,
    pub parameters: String,
}

pub fn uninstall_launch_of(program: &InstalledProgram) -> Option<UninstallLaunch> {
    let command = program.uninstall_command.as_deref()?.trim();
    let (executable, parameters) = split_command(command)?;
    let parameters = if is_windows_installer(executable) {
        asking_for_removal(parameters)
    } else {
        parameters.to_owned()
    };
    Some(UninstallLaunch {
        executable: executable.to_owned(),
        parameters,
    })
}

fn split_command(command: &str) -> Option<(&str, &str)> {
    if let Some(after_opening_quote) = command.strip_prefix(QUOTE) {
        let (executable, parameters) = after_opening_quote.split_once(QUOTE)?;
        return Some((executable, parameters.trim()))
            .filter(|(executable, _)| !executable.is_empty());
    }
    let executable_end = match command.to_lowercase().find(EXECUTABLE_EXTENSION) {
        Some(extension_start) => extension_start + EXECUTABLE_EXTENSION.len(),
        None => command.find(char::is_whitespace).unwrap_or(command.len()),
    };
    let executable = command.get(..executable_end)?;
    let parameters = command.get(executable_end..)?;
    Some((executable, parameters.trim())).filter(|(executable, _)| !executable.is_empty())
}

fn is_windows_installer(executable: &str) -> bool {
    let file_name = executable.rsplit('\\').next().unwrap_or(executable);
    file_name.to_lowercase().starts_with(WINDOWS_INSTALLER_NAME)
}

fn asking_for_removal(parameters: &str) -> String {
    match parameters.to_lowercase().find(INSTALLER_MAINTENANCE_SWITCH) {
        Some(switch_start) => {
            let switch_end = switch_start + INSTALLER_MAINTENANCE_SWITCH.len();
            let before_switch = parameters.get(..switch_start).unwrap_or_default();
            let after_switch = parameters.get(switch_end..).unwrap_or_default();
            format!("{before_switch}{INSTALLER_REMOVAL_SWITCH}{after_switch}")
        }
        None => parameters.to_owned(),
    }
}
