import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useEffect, useState } from "react";
import "./App.css";

function App() {
  const [greetMsg, setGreetMsg] = useState("");
  const [currentWallpaper, setCurrentWallpaper] = useState("None");

  // --- Backend Event Listeners ---
  // This is how our GUI reacts to the system tray
  useEffect(() => {
    const unlistenNext = listen("next_wallpaper", (event) => {
      console.log("User wants the next wallpaper!", event);
      // Here we would call the core logic to get and set a new wallpaper
      // e.g., invoke('set_random_wallpaper').then(setCurrentWallpaper);
    });

    const unlistenApprove = listen("approve_wallpaper", (event) => {
      console.log("User approved the current wallpaper!", event);
      // invoke('approve_current_wallpaper');
    });

    const unlistenReject = listen("reject_wallpaper", (event) => {
      console.log("User rejected the current wallpaper!", event);
      // invoke('reject_current_wallpaper');
    });

    return () => {
      // Cleanup listeners when component unmounts
      unlistenNext.then((f) => f());
      unlistenApprove.then((f) => f());
      unlistenReject.then((f) => f());
    };
  }, []);

  async function greet() {
    setGreetMsg(await invoke("greet", { name: "Wallpaper Manager" }));
  }

  return (
    <div className="container">
      <h1>Welcome to Wallpaper Manager!</h1>
      <p>This window can be used for settings and viewing pending images.</p>

      <div className="row">
        <button onClick={greet}>Greet Backend</button>
      </div>
      <p>{greetMsg}</p>

      <p>Current Wallpaper: {currentWallpaper}</p>
      <p>Right-click the system tray icon to control the application.</p>
    </div>
  );
}

export default App;
