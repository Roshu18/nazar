use nazar_core::change_manifest::{Change, ChangeManifest, EffectCounts};
use nazar_core::net_effect::NetEffect;
use nazar_core::process_lineage::AttributionRule;
use nazar_core::removal_plan::{RemovalStep, plan_removal};
use nazar_core::tracked_object::TrackedObject;

const NOTHING_RECORDED: &str = "The program made no lasting changes.\n";

pub fn render_summary(manifest: &ChangeManifest) -> String {
    let summary = manifest.summarize();
    let duration_in_seconds = manifest
        .finished_at_unix_seconds
        .saturating_sub(manifest.started_at_unix_seconds);
    let mut report = String::new();
    push_line(&mut report, &format!("Program   {}", manifest.program));
    push_line(
        &mut report,
        &format!("Duration  {duration_in_seconds} seconds"),
    );
    push_line(&mut report, "");
    push_line(
        &mut report,
        &count_row("", "created", "modified", "deleted"),
    );
    push_line(
        &mut report,
        &counts_row("Files and folders", summary.file_system_entries),
    );
    push_line(
        &mut report,
        &counts_row("Registry keys", summary.registry_keys),
    );
    push_line(
        &mut report,
        &counts_row("Registry values", summary.registry_values),
    );
    report
}

pub fn render_changes(manifest: &ChangeManifest) -> String {
    if manifest.changes.is_empty() {
        return NOTHING_RECORDED.to_owned();
    }
    let mut report = String::new();
    for change in &manifest.changes {
        push_line(&mut report, &change_line(change));
    }
    report
}

pub fn render_removal_plan(manifest: &ChangeManifest) -> String {
    let steps = plan_removal(&manifest.changes);
    if steps.is_empty() {
        return NOTHING_RECORDED.to_owned();
    }
    let mut report = String::new();
    for step in &steps {
        push_line(&mut report, &removal_step_line(step));
    }
    report
}

fn counts_row(label: &str, counts: EffectCounts) -> String {
    count_row(
        label,
        &counts.created.to_string(),
        &counts.modified.to_string(),
        &counts.deleted.to_string(),
    )
}

fn count_row(label: &str, created: &str, modified: &str, deleted: &str) -> String {
    format!("{label:<18}{created:>9}{modified:>10}{deleted:>9}")
}

fn change_line(change: &Change) -> String {
    let effect = match change.effect {
        NetEffect::Created => "created",
        NetEffect::Modified { .. } => "modified",
        NetEffect::Deleted { .. } => "deleted",
    };
    let attribution_note = match change.attribution {
        AttributionRule::Descent => "",
        AttributionRule::DroppedImage => "  (by a program it installed)",
        AttributionRule::InstallerService => "  (through Windows Installer)",
    };
    format!("{effect:<10}{}{attribution_note}", change.object)
}

fn removal_step_line(step: &RemovalStep) -> String {
    match step {
        RemovalStep::DeleteFileSystemEntry { path } | RemovalStep::DeleteRegistryKey { path } => {
            format!("delete    {path}")
        }
        RemovalStep::DeleteRegistryValue { key_path, name } => {
            let value = TrackedObject::registry_value(key_path, name);
            format!("delete    {value}")
        }
        RemovalStep::RestoreRegistryValue { key_path, name, .. } => {
            let value = TrackedObject::registry_value(key_path, name);
            format!("restore   {value}")
        }
        RemovalStep::ReviewManually { object, .. } => {
            format!("review    {object}  (its original content was not captured)")
        }
    }
}

fn push_line(report: &mut String, line: &str) {
    report.push_str(line.trim_end());
    report.push('\n');
}
