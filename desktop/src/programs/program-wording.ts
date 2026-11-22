import type {
  DeletionReport,
  DeletionStatus,
  LocationReference,
  LocationRole,
  ProgramOverview,
} from "./program-api";

const BYTES_PER_KILOBYTE = 1024;
const SIZE_UNITS = ["KB", "MB", "GB", "TB"] as const;
const INSTALL_DATE_PATTERN = /^(\d{4})(\d{2})(\d{2})$/;
const DETAIL_SEPARATOR = " · ";
const DEFAULT_VALUE_LABEL = "(Default)";
const PATH_SEPARATOR = "\\";

const ROLE_LABELS: Record<LocationRole, string> = {
  uninstallEntry: "Uninstall entry",
  installFolder: "Install folder",
  programFolder: "Program folder",
  dataFolder: "Data folder",
  shortcut: "Shortcut",
  settingsKey: "Settings",
  autostart: "Starts with Windows",
  service: "Service",
  appPath: "App path",
};

const STATUS_SUMMARIES: Record<DeletionStatus, readonly [string, string]> = {
  removed: ["deleted", "deleted"],
  alreadyGone: ["was already gone", "were already gone"],
  refused: ["is protected and was kept", "are protected and were kept"],
  failed: ["could not be deleted", "could not be deleted"],
};

const installDateFormat = new Intl.DateTimeFormat(undefined, { dateStyle: "medium" });
const sizeFormat = new Intl.NumberFormat(undefined, { maximumFractionDigits: 1 });

export function roleLabel(role: LocationRole): string {
  return ROLE_LABELS[role];
}

export function publisherOf(program: ProgramOverview): string {
  return program.publisher ?? "Unknown publisher";
}

export function installedSizeOf(program: ProgramOverview): number | null {
  return program.sizeInKilobytes === null ? null : program.sizeInKilobytes * BYTES_PER_KILOBYTE;
}

export function formatInstallDate(installDate: string | null): string | null {
  const dateParts = installDate === null ? null : INSTALL_DATE_PATTERN.exec(installDate);
  if (dateParts === null) {
    return null;
  }
  const [, year, month, day] = dateParts;
  return installDateFormat.format(new Date(Number(year), Number(month) - 1, Number(day)));
}

export function formatSize(sizeInBytes: number): string {
  let size = sizeInBytes / BYTES_PER_KILOBYTE;
  let unit: string = SIZE_UNITS[0];
  for (const largerUnit of SIZE_UNITS.slice(1)) {
    if (size < BYTES_PER_KILOBYTE) {
      break;
    }
    size /= BYTES_PER_KILOBYTE;
    unit = largerUnit;
  }
  return `${sizeFormat.format(size)} ${unit}`;
}

export function nameOf(location: LocationReference): string {
  if (location.valueName !== null) {
    return location.valueName === "" ? DEFAULT_VALUE_LABEL : location.valueName;
  }
  return location.path.slice(location.path.lastIndexOf(PATH_SEPARATOR) + 1);
}

export function containerOf(location: LocationReference): string {
  if (location.valueName !== null) {
    return location.path;
  }
  const separatorPosition = location.path.lastIndexOf(PATH_SEPARATOR);
  return separatorPosition < 0 ? "" : location.path.slice(0, separatorPosition);
}

export function describeLocation(location: LocationReference): string {
  return location.valueName === null ? location.path : `${location.path} [${nameOf(location)}]`;
}

export function summarizeDeletion(report: DeletionReport): string {
  const statuses: readonly DeletionStatus[] = ["removed", "alreadyGone", "refused", "failed"];
  const summaries = statuses.map((status) => {
    const count = report.outcomes.filter((outcome) => outcome.status === status).length;
    const [singular, plural] = STATUS_SUMMARIES[status];
    return count === 0 ? null : `${count} ${count === 1 ? singular : plural}`;
  });
  return joinDetails(summaries);
}

export function joinDetails(details: readonly (string | null)[]): string {
  return details.filter((detail) => detail !== null && detail !== "").join(DETAIL_SEPARATOR);
}
