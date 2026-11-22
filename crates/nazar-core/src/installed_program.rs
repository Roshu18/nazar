#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstalledProgram {
    pub registry_key_path: String,
    pub display_name: String,
    pub publisher: Option<String>,
    pub version: Option<String>,
    pub install_location: Option<String>,
    pub display_icon: Option<String>,
    pub uninstall_command: Option<String>,
    pub install_date: Option<String>,
    pub estimated_size_in_kilobytes: Option<u64>,
}
