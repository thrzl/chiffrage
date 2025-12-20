import { WebviewWindow } from "@tauri-apps/api/webviewWindow";

// Source - https://stackoverflow.com/a/18650828
// Posted by anon, modified by community. See post 'Timeline' for change history
// Retrieved 2025-12-19, License - CC BY-SA 4.0
export function formatBytes(bytes: number, decimals = 2) {
  if (!+bytes) return "0 bytes";

  const k = 1000;
  const dm = decimals < 0 ? 0 : decimals;
  const sizes = ["bytes", "KB", "MB", "GB", "TB", "PB", "EB", "ZB", "YB"];

  const i = Math.floor(Math.log(bytes) / Math.log(k));

  return `${parseFloat((bytes / Math.pow(k, i)).toFixed(dm))} ${sizes[i]}`;
}

export function getFileName(path: string) {
  const normalized = path.replace(/\\/g, "/");
  return normalized.split("/").pop();
}

export type Progress = {
  read_bytes: number;
  total_bytes: number;
  current_file: string;
};

export type Key = {
  id: string;
  name: string;
  key_type: "Public" | "Private";
  date_created: { secs_since_epoch: number };
  contents: {
    public: string;
    private: {
      nonce: number[];
      ciphertext: number[];
    } | null;
  };
};

export function openWindow(window: string, title: string) {
  const win = new WebviewWindow(window, {
    title,
    url: window, // or correct relative path
    width: 500,
    height: 600,
  });

  win.once("tauri://created", () => {
    console.log("New window created!");
  });
  win.once("tauri://error", (e) => {
    console.error("Failed to create window", e);
  });
}
