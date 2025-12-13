import { WebviewWindow } from "@tauri-apps/api/webviewWindow";

export type Key = {
  name: string;
  key_type: "public" | "private";
  date_created: { secs_since_epoch: number };
  contents: {
    public: String;
    private: String | null;
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
