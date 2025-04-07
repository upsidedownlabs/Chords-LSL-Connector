"use client";

import React, { useState } from "react";
import { Bluetooth, Wifi, Link, X, Minus, Pin } from "lucide-react";
import { Window } from "@tauri-apps/api/window";

export default function CustomTitleBar() {
  const [alwaysOnTop, setAlwaysOnTop] = useState(false);
  const appWindow = Window.getCurrent();

  const toggleAlwaysOnTop = async () => {
    const newValue = !alwaysOnTop;
    setAlwaysOnTop(newValue);
    await appWindow.setAlwaysOnTop(newValue);
  };

  const minimizeWindow = async () => {
    await appWindow.minimize();
  };

  const closeWindow = async () => {
    await appWindow.close();
  };

  return (
    <div
      className="flex justify-between items-center w-full h-12 px-4 bg-gray-800 text-white select-none"
      data-tauri-drag-region
    >
      {/* Left Buttons */}
      <div className="flex space-x-3">
        <button className="hover:text-blue-400" title="Serial">
          <Link size={20} />
        </button>
        <button className="hover:text-blue-400" title="Bluetooth">
          <Bluetooth size={20} />
        </button>
        <button className="hover:text-blue-400" title="WiFi">
          <Wifi size={20} />
        </button>
      </div>

      {/* Right Buttons */}
      <div className="flex space-x-3">
        <button
          onClick={toggleAlwaysOnTop}
          className={`${alwaysOnTop ? "text-green-400" : "text-white"} hover:text-green-300`}
          title="Toggle Always on Top"
        >
          <Pin size={20} />
        </button>
        <button
          onClick={minimizeWindow}
          className="hover:text-yellow-400"
          title="Minimize"
        >
          <Minus size={20} />
        </button>
        <button
          onClick={closeWindow}
          className="hover:text-red-400"
          title="Close"
        >
          <X size={20} />
        </button>
      </div>
    </div>
  );
}
