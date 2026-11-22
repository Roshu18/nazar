const PATH_SEPARATOR: char = '\\';

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum Namespace {
    FileSystem,
    Registry,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ObjectIdentity {
    namespace: Namespace,
    container_path: String,
    value_name: Option<String>,
}

impl ObjectIdentity {
    pub fn of_file_system_entry(path: &str) -> Self {
        Self::of_container(Namespace::FileSystem, path)
    }

    pub fn of_registry_key(path: &str) -> Self {
        Self::of_container(Namespace::Registry, path)
    }

    pub fn of_registry_value(key_path: &str, name: &str) -> Self {
        Self {
            value_name: Some(name.to_lowercase()),
            ..Self::of_registry_key(key_path)
        }
    }

    pub fn is_value_of(&self, key: &Self) -> bool {
        self.namespace == key.namespace
            && self.value_name.is_some()
            && key.value_name.is_none()
            && self.container_path == key.container_path
    }

    pub fn is_inside(&self, container: &Self) -> bool {
        self.namespace == container.namespace
            && self
                .container_path
                .strip_prefix(&container.container_path)
                .is_some_and(|remainder| remainder.starts_with(PATH_SEPARATOR))
    }

    fn of_container(namespace: Namespace, path: &str) -> Self {
        Self {
            namespace,
            container_path: path.to_lowercase(),
            value_name: None,
        }
    }
}
