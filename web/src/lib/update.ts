import { writable } from "svelte/store";

export type UpdateStatusType =
  | { type: "checking" }
  | { type: "latest" }
  | { type: "finished" }
  | { type: "downloading"; chunk: number; length: number | null };

export interface UpdateState {
  status: UpdateStatusType;
  text: string;
  loading: boolean;
}

export const updateState = writable<UpdateState>({
  status: { type: "checking" },
  text: "Loading assets...",
  loading: true,
});

export function setUpdateStatus(status: UpdateStatusType) {
  updateState.update(() => {
    let text = "";
    let loading = true;

    switch (status.type) {
      case "checking":
        text = "Checking updates...";
        break;
      case "downloading":
        text = `Downloading newest version ${status.chunk} / ${status.length ?? "?"}`;
        break;
      case "latest":
      case "finished":
        text = status.type === "latest" ? "Using latest version" : "Finished";
        loading = false;
        break;
    }

    return { status, text, loading };
  });
}