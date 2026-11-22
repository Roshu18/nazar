import type { Attribution, ChangeEffect, RemovalAction, TraceDetails } from "./trace-api";

const MILLISECONDS_PER_SECOND = 1000;

const EFFECT_LABELS: Record<ChangeEffect, string> = {
  created: "Created",
  modified: "Changed",
  deleted: "Deleted",
};

const ATTRIBUTION_NOTES: Record<Attribution, string> = {
  descent: "",
  droppedImage: "by a program it installed",
  installerService: "through Windows Installer",
};

const REMOVAL_ACTION_LABELS: Record<RemovalAction, string> = {
  delete: "Delete",
  restore: "Restore",
  review: "Review",
};

const startTimeFormat = new Intl.DateTimeFormat(undefined, {
  dateStyle: "medium",
  timeStyle: "short",
});

export function effectLabel(effect: ChangeEffect): string {
  return EFFECT_LABELS[effect];
}

export function attributionNote(attribution: Attribution): string {
  return ATTRIBUTION_NOTES[attribution];
}

export function removalActionLabel(action: RemovalAction): string {
  return REMOVAL_ACTION_LABELS[action];
}

export function formatStartTime(startedAtUnixSeconds: number): string {
  return startTimeFormat.format(new Date(startedAtUnixSeconds * MILLISECONDS_PER_SECOND));
}

export function summarize(details: TraceDetails): string {
  const fileSummary = countOf(details.fileChanges.length, "file or folder", "files and folders");
  const registrySummary = countOf(
    details.registryChanges.length,
    "registry entry",
    "registry entries",
  );
  const durationSummary = countOf(details.durationInSeconds, "second", "seconds");
  return `${fileSummary} · ${registrySummary} · tracked for ${durationSummary}`;
}

export function countOf(count: number, singular: string, plural: string): string {
  return `${count} ${count === 1 ? singular : plural}`;
}
