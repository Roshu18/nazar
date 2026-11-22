import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";

export type ChangeEffect = "created" | "modified" | "deleted";
export type Attribution = "descent" | "droppedImage" | "installerService";
export type RemovalAction = "delete" | "restore" | "review";

export interface TraceOverview {
  readonly name: string;
  readonly programName: string;
  readonly programPath: string;
  readonly startedAtUnixSeconds: number;
  readonly changeCount: number;
}

export interface ChangeRow {
  readonly path: string;
  readonly valueName: string | null;
  readonly effect: ChangeEffect;
  readonly attribution: Attribution;
}

export interface RemovalStep {
  readonly action: RemovalAction;
  readonly target: string;
}

export interface TraceDetails {
  readonly overview: TraceOverview;
  readonly durationInSeconds: number;
  readonly unresolvedEventCount: number;
  readonly fileChanges: readonly ChangeRow[];
  readonly registryChanges: readonly ChangeRow[];
  readonly removalSteps: readonly RemovalStep[];
}

const TRACKABLE_EXTENSIONS = ["exe", "msi", "bat", "cmd", "ps1"];

export function listTraces(): Promise<TraceOverview[]> {
  return invoke<TraceOverview[]>("list_traces");
}

export function readTrace(name: string): Promise<TraceDetails> {
  return invoke<TraceDetails>("read_trace", { name });
}

export function trackProgram(programPath: string): Promise<string> {
  return invoke<string>("track_program", { programPath });
}

export function stopTracking(): Promise<void> {
  return invoke<void>("stop_tracking");
}

export function chooseProgram(): Promise<string | null> {
  return open({
    title: "Choose a program to track",
    multiple: false,
    directory: false,
    filters: [{ name: "Programs and scripts", extensions: TRACKABLE_EXTENSIONS }],
  });
}
