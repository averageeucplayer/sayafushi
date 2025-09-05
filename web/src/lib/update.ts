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
        const downloaded = formatBytes(status.chunk);
        const total = formatBytes(status.length ?? null);
        text = `Downloading newest version ${downloaded} / ${total}`;
        break;
      case "latest":
        text = "Using latest version";
        loading = false;
        break;
    }

    return { status, text, loading };
  });
}

function formatBytes(bytes: number | null): string {
  if (bytes == null) return "?";

  const sizes = ["B", "KB", "MB", "GB", "TB"];
  let i = 0;
  let value = bytes;

  while (value >= 1024 && i < sizes.length - 1) {
    value /= 1024;
    i++;
  }

  return `${value.toFixed(2)} ${sizes[i]}`;
}