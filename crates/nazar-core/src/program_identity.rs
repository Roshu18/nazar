use std::collections::BTreeSet;

use crate::install_folder::{
    InstallFolder, install_folder_of, last_component_of, referenced_files_of,
};
use crate::installed_program::InstalledProgram;
use crate::program_name::{is_distinctive, name_key};

const PATH_SEPARATOR: char = '\\';
const GENERATED_KEY_NAME_START: char = '{';
const FILE_EXTENSION_SEPARATOR: char = '.';
const SHARED_START_MINIMUM_LENGTH: usize = 4;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProgramIdentity {
    names: BTreeSet<String>,
    publisher_name: Option<String>,
    install_folder: Option<InstallFolder>,
}

impl ProgramIdentity {
    pub fn of(program: &InstalledProgram) -> Self {
        let install_folder = install_folder_of(program);
        let folder_name = install_folder
            .as_ref()
            .map(|folder| last_component_of(&folder.path).to_owned());
        let candidate_names = [
            Some(program.display_name.clone()),
            name_without_publisher(program),
            folder_name,
            uninstall_key_name(program),
        ];
        let names = candidate_names
            .into_iter()
            .flatten()
            .chain(referenced_file_names(program))
            .map(|name| name_key(&name))
            .filter(|key| is_distinctive(key))
            .collect();
        let publisher_name = program
            .publisher
            .as_deref()
            .map(name_key)
            .filter(|key| is_distinctive(key));
        Self {
            names,
            publisher_name,
            install_folder,
        }
    }

    pub fn install_folder(&self) -> Option<&InstallFolder> {
        self.install_folder.as_ref()
    }

    pub fn is_named(&self, name: &str) -> bool {
        self.names.contains(&name_key(name))
    }

    pub fn is_published_by(&self, name: &str) -> bool {
        let candidate = name_key(name);
        let Some(publisher_name) = &self.publisher_name else {
            return false;
        };
        is_distinctive(&candidate)
            && (publisher_name.starts_with(&candidate) || candidate.starts_with(publisher_name))
    }

    pub fn has_name_starting_like(&self, name: &str) -> bool {
        let candidate = name_key(name);
        let is_long_enough = |key: &str| key.chars().count() >= SHARED_START_MINIMUM_LENGTH;
        is_long_enough(&candidate)
            && self.names.iter().any(|own_name| {
                own_name.starts_with(&candidate)
                    || (is_long_enough(own_name) && candidate.starts_with(own_name.as_str()))
            })
    }

    pub fn is_installed_within(&self, folder_path: &str) -> bool {
        let Some(install_folder) = &self.install_folder else {
            return false;
        };
        let outer_folder = format!("{}{PATH_SEPARATOR}", folder_path.to_lowercase());
        install_folder
            .path
            .to_lowercase()
            .starts_with(&outer_folder)
    }

    pub fn has_inside_install_folder(&self, text: &str) -> bool {
        let Some(install_folder) = &self.install_folder else {
            return false;
        };
        let folder_prefix = format!("{}{PATH_SEPARATOR}", install_folder.path.to_lowercase());
        text.to_lowercase().contains(&folder_prefix)
    }
}

fn name_without_publisher(program: &InstalledProgram) -> Option<String> {
    let first_publisher_word = program.publisher.as_deref()?.split_whitespace().next()?;
    let (first_name_word, remaining_name) = program.display_name.trim().split_once(' ')?;
    first_name_word
        .eq_ignore_ascii_case(first_publisher_word)
        .then(|| remaining_name.to_owned())
}

fn uninstall_key_name(program: &InstalledProgram) -> Option<String> {
    let key_name = last_component_of(&program.registry_key_path);
    (!key_name.starts_with(GENERATED_KEY_NAME_START)).then(|| key_name.to_owned())
}

fn referenced_file_names(program: &InstalledProgram) -> Vec<String> {
    referenced_files_of(program)
        .iter()
        .map(|file| last_component_of(file))
        .filter_map(|file_name| file_name.rsplit_once(FILE_EXTENSION_SEPARATOR))
        .map(|(stem, _)| stem.to_owned())
        .collect()
}
