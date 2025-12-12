import x.json2
import os

fn main() {
    config_file := os.read_file("src-tauri/tauri.conf.json")!
    tauri_config := json2.decode[json2.Any](config_file)!.as_map()
    print(tauri_config["version"]!)
}
