use std::ffi::{OsStr, OsString};
use std::path::Path;

const INSTALLER_PACKAGE_EXTENSION: &str = "msi";
const POWERSHELL_SCRIPT_EXTENSION: &str = "ps1";
const INSTALLER_EXECUTABLE: &str = "msiexec.exe";
const POWERSHELL_EXECUTABLE: &str = "powershell.exe";
const INSTALL_PACKAGE_ARGUMENTS: [&str; 1] = ["/i"];
const RUN_SCRIPT_ARGUMENTS: [&str; 4] = ["-NoProfile", "-ExecutionPolicy", "Bypass", "-File"];

#[derive(Debug, PartialEq, Eq)]
pub struct LaunchPlan {
    pub executable: OsString,
    pub arguments: Vec<OsString>,
}

pub fn plan_launch(program: &Path, arguments: &[String]) -> LaunchPlan {
    let extension = program
        .extension()
        .and_then(OsStr::to_str)
        .map(str::to_ascii_lowercase);
    match extension.as_deref() {
        Some(INSTALLER_PACKAGE_EXTENSION) => plan_through_host(
            INSTALLER_EXECUTABLE,
            &INSTALL_PACKAGE_ARGUMENTS,
            program,
            arguments,
        ),
        Some(POWERSHELL_SCRIPT_EXTENSION) => plan_through_host(
            POWERSHELL_EXECUTABLE,
            &RUN_SCRIPT_ARGUMENTS,
            program,
            arguments,
        ),
        _ => LaunchPlan {
            executable: program.into(),
            arguments: arguments.iter().map(OsString::from).collect(),
        },
    }
}

fn plan_through_host(
    host_executable: &str,
    host_arguments: &[&str],
    program: &Path,
    arguments: &[String],
) -> LaunchPlan {
    let host_arguments = host_arguments.iter().map(OsString::from);
    let program_arguments = arguments.iter().map(OsString::from);
    LaunchPlan {
        executable: host_executable.into(),
        arguments: host_arguments
            .chain([program.into()])
            .chain(program_arguments)
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn arguments_of(plan: &LaunchPlan) -> Vec<&str> {
        plan.arguments
            .iter()
            .filter_map(|argument| argument.to_str())
            .collect()
    }

    #[test]
    fn executable_is_launched_directly_with_its_arguments() {
        let plan = plan_launch(Path::new("C:\\Downloads\\setup.exe"), &["/S".to_owned()]);

        assert_eq!(plan.executable, "C:\\Downloads\\setup.exe");
        assert_eq!(arguments_of(&plan), ["/S"]);
    }

    #[test]
    fn installer_package_is_launched_through_windows_installer() {
        let plan = plan_launch(Path::new("C:\\Downloads\\App.MSI"), &["/qn".to_owned()]);

        assert_eq!(plan.executable, "msiexec.exe");
        assert_eq!(arguments_of(&plan), ["/i", "C:\\Downloads\\App.MSI", "/qn"]);
    }

    #[test]
    fn powershell_script_is_launched_through_powershell() {
        let plan = plan_launch(Path::new("install.ps1"), &[]);

        assert_eq!(plan.executable, "powershell.exe");
        assert_eq!(
            arguments_of(&plan),
            [
                "-NoProfile",
                "-ExecutionPolicy",
                "Bypass",
                "-File",
                "install.ps1"
            ]
        );
    }
}
