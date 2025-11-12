export enum SetupState {
  Pending = "pending",
  Loading = "loading",
  Ready = "ready",
  Failed = "failed",
}

export interface ModelDownloadProgress {
  status: "starting" | "downloading" | "complete" | "error";
  message: string;
}
