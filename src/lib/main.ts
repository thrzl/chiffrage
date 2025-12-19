import { WebviewWindow } from "@tauri-apps/api/webviewWindow";

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
