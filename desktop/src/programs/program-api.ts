import { invoke } from "@tauri-apps/api/core";

export type LocationKind = "fileSystemEntry" | "registryKey" | "registryValue";
export type LocationRole =
  | "uninstallEntry"
  | "installFolder"
  | "programFolder"
  | "dataFolder"
  | "shortcut"
  | "settingsKey"
  | "autostart"
  | "service"
  | "appPath";
export type DeletionStatus = "removed" | "alreadyGone" | "refused" | "failed";

export interface ProgramOverview {
  readonly id: string;
  readonly name: string;
  readonly publisher: string | null;
  readonly version: string | null;
  readonly installDate: string | null;
  readonly sizeInKilobytes: number | null;
  readonly iconSource: string | null;
  readonly hasUninstaller: boolean;
}

export interface LocationReference {
  readonly kind: LocationKind;
  readonly path: string;
  readonly valueName: string | null;
}

export interface ProgramLocation {
  readonly location: LocationReference;
  readonly role: LocationRole;
  readonly isCertain: boolean;
}

export interface ProgramDetails {
  readonly overview: ProgramOverview;
  readonly isInstalled: boolean;
  readonly fileLocations: readonly ProgramLocation[];
  readonly registryLocations: readonly ProgramLocation[];
  readonly unreadableLocationCount: number;
}

export interface DeletionOutcome {
  readonly location: LocationReference;
  readonly status: DeletionStatus;
  readonly reason: string | null;
}

export interface DeletionReport {
  readonly registryBackupDirectory: string | null;
  readonly outcomes: readonly DeletionOutcome[];
}

export function listInstalledPrograms(): Promise<ProgramOverview[]> {
  return invoke<ProgramOverview[]>("list_installed_programs");
}

export function listLeftovers(): Promise<ProgramOverview[]> {
  return invoke<ProgramOverview[]>("list_leftovers");
}

export function readProgram(id: string): Promise<ProgramDetails> {
  return invoke<ProgramDetails>("read_program", { id });
}

export function uninstallProgram(id: string): Promise<void> {
  return invoke<void>("uninstall_program", { id });
}

export function deleteLocations(locations: readonly LocationReference[]): Promise<DeletionReport> {
  return invoke<DeletionReport>("delete_locations", { locations });
}
