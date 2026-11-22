use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};

use crate::observation::ProcessId;

const INSTALLER_IMAGE_NAME: &str = "msiexec.exe";
const PATH_SEPARATOR: char = '\\';

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AttributionRule {
    Descent,
    DroppedImage,
    InstallerService,
}

#[derive(Debug, Clone, Copy)]
struct TrackedProcess {
    rule: AttributionRule,
    is_installer_client: bool,
}

#[derive(Debug)]
pub struct ProcessLineage {
    launcher: ProcessId,
    tracked_processes: HashMap<ProcessId, TrackedProcess>,
    installer_processes: HashSet<ProcessId>,
    dropped_image_paths: HashSet<String>,
}

impl ProcessLineage {
    pub fn new(launcher: ProcessId) -> Self {
        Self {
            launcher,
            tracked_processes: HashMap::new(),
            installer_processes: HashSet::new(),
            dropped_image_paths: HashSet::new(),
        }
    }

    pub fn note_running_process(&mut self, process: ProcessId, image_name: &str) {
        if is_installer_image(image_name) {
            self.installer_processes.insert(process);
        }
    }

    pub fn note_process_start(&mut self, process: ProcessId, parent: ProcessId, image_path: &str) {
        let is_installer = is_installer_image(image_path);
        if is_installer {
            self.installer_processes.insert(process);
        }
        if let Some(rule) = self.rule_for_new_process(parent, image_path) {
            let is_installer_client = is_installer && rule == AttributionRule::Descent;
            let tracked_process = TrackedProcess {
                rule,
                is_installer_client,
            };
            self.tracked_processes.insert(process, tracked_process);
        }
    }

    pub fn note_process_stop(&mut self, process: ProcessId) {
        self.tracked_processes.remove(&process);
        self.installer_processes.remove(&process);
    }

    pub fn note_dropped_image(&mut self, path: &str) {
        self.dropped_image_paths.insert(path.to_lowercase());
    }

    pub fn attribution_of(&self, process: ProcessId) -> Option<AttributionRule> {
        if let Some(tracked_process) = self.tracked_processes.get(&process) {
            return Some(tracked_process.rule);
        }
        let serves_tracked_client =
            self.installer_processes.contains(&process) && self.has_live_installer_client();
        serves_tracked_client.then_some(AttributionRule::InstallerService)
    }

    pub fn has_live_tracked_process(&self) -> bool {
        !self.tracked_processes.is_empty()
    }

    fn has_live_installer_client(&self) -> bool {
        self.tracked_processes
            .values()
            .any(|tracked_process| tracked_process.is_installer_client)
    }

    fn rule_for_new_process(&self, parent: ProcessId, image_path: &str) -> Option<AttributionRule> {
        if parent == self.launcher {
            return Some(AttributionRule::Descent);
        }
        if let Some(inherited_rule) = self.attribution_of(parent) {
            return Some(inherited_rule);
        }
        self.dropped_image_paths
            .contains(&image_path.to_lowercase())
            .then_some(AttributionRule::DroppedImage)
    }
}

fn is_installer_image(image_path: &str) -> bool {
    file_name_of(image_path).eq_ignore_ascii_case(INSTALLER_IMAGE_NAME)
}

fn file_name_of(path: &str) -> &str {
    path.rsplit(PATH_SEPARATOR).next().unwrap_or(path)
}
