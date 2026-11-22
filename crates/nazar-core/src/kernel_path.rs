const PATH_SEPARATOR: char = '\\';
const REGISTRY_ROOTS: [(&str, &str); 2] =
    [("\\REGISTRY\\MACHINE", "HKLM"), ("\\REGISTRY\\USER", "HKU")];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VolumeName {
    pub device_path: String,
    pub drive: String,
}

#[derive(Debug, Clone, Default)]
pub struct VolumeNames {
    volumes: Vec<VolumeName>,
}

impl VolumeNames {
    pub fn new(volumes: Vec<VolumeName>) -> Self {
        Self { volumes }
    }

    pub fn translate_file_path(&self, kernel_path: &str) -> String {
        self.volumes
            .iter()
            .find_map(|volume| replace_root(kernel_path, &volume.device_path, &volume.drive))
            .unwrap_or_else(|| kernel_path.to_owned())
    }
}

pub fn translate_registry_path(kernel_path: &str) -> String {
    REGISTRY_ROOTS
        .iter()
        .find_map(|(kernel_root, familiar_root)| {
            replace_root(kernel_path, kernel_root, familiar_root)
        })
        .unwrap_or_else(|| kernel_path.to_owned())
}

fn replace_root(path: &str, root: &str, replacement: &str) -> Option<String> {
    let path_root = path.get(..root.len())?;
    let remainder = path.get(root.len()..)?;
    let ends_at_separator = remainder.is_empty() || remainder.starts_with(PATH_SEPARATOR);
    let is_under_root = path_root.eq_ignore_ascii_case(root) && ends_at_separator;
    is_under_root.then(|| format!("{replacement}{remainder}"))
}
