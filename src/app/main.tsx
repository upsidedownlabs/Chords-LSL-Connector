"use client";

import React, { useRef, useState, useEffect } from 'react';
import { core } from "@tauri-apps/api";
import { Link, Wifi, Bluetooth, X, Minus, ChevronsUp, Usb } from 'lucide-react';
import { listen } from "@tauri-apps/api/event";
import { Window } from "@tauri-apps/api/window";

const App = () => {
  const [deviceConnected, setDeviceConnected] = useState(false);
  const portRef = useRef<unknown>(null);
  const [activeButton, setActiveButton] = useState<"serial" | "wifi" | "bluetooth" | null>(null);
  const [devices, setDevices] = useState<{ name: string; id: string }[]>([]);
  const [status, setStatus] = useState("");
  const [samplerate, setSamplerate] = useState<number | undefined>(0);
  const [samplelost, setSamplelost] = useState<number | undefined>(0);
  const [lsl, setLSL] = useState("");

  const isProcessing = useRef(false);
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
  const appWindow = Window.getCurrent();
  const [scane, setScane] = useState(false);
  const [alwaysOnTop, setAlwaysOnTop] = useState(true);

  useEffect(() => {
    appWindow.setAlwaysOnTop(true);
  }, [appWindow]);

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

  const ConnectserialDevice = async () => {
    try {
      isProcessing.current = true;
      const portName = await core.invoke('detect_arduino') as string;
      portRef.current = portName;
      setDeviceConnected(true);
      await core.invoke('start_streaming', { portName: portRef.current, stream_name: "UDL" });
    } catch (error) {
      console.error('Failed to connect to device:', error);
    }
  };

  const ConnectwifiDevice = async () => {
    try {
      isProcessing.current = true;
      setDeviceConnected(true);
      await core.invoke("start_wifistreaming");
    } catch (error) {
      console.error('Failed to connect to device:', error);
    }
  };

  const ConnectbluetoothDevice = async () => {
    try {
      setScane(true);
      isProcessing.current = true;
      await core.invoke("scan_ble_devices");
    } catch (error) {
      console.error("Failed to connect to device:", error);
    }
  };

  useEffect(() => {
    listen('connection', (event) => {
      setStatus(event.payload as string);
    });
    listen('bleDevices', (event) => {
      const devices = event.payload as { name: string; id: string }[];
      if (devices.length === 0) {
        setStatus("No NPG device");
      }
      setDevices(devices);
    });
    
    listen('samplerate', (event) => {
      setSamplerate(Math.ceil(Number(event.payload)));
    });
    listen('samplelost', (event) => {
      setSamplelost(event.payload as number);
    });

    listen('lsl', (event) => {
      setLSL(event.payload as string);
    });
  }, []);

  

  return (
    <>
      <div className=" flex-col bg-gray-200 overflow-hidden">

        <div className="w-full">
          <div
            className="flex justify-between items-center w-full h-12 px-4 bg-gray-800 text-white select-none"
            data-tauri-drag-region
          >
            {/* Left Buttons */}
            <div className="flex space-x-3">
              <button
                onClick={() => {
                  setActiveButton("serial");
                  setScane(false);
                  setDevices([]);
                  ConnectserialDevice();
                }}
                className={`transition-colors duration-300 hover:text-blue-400 ${activeButton === "serial"
                  ? "text-green-500"
                  : ""
                  }`}
                title="Serial"
                disabled={deviceConnected}
              >
                <Usb size={20} />
              </button>

              <button
                onClick={() => {
                  setActiveButton("bluetooth");
                  setDevices([]);
                  ConnectbluetoothDevice();
                }}
                className={`transition-colors duration-300 hover:text-blue-400 ${activeButton === "bluetooth"
                  ? "text-green-500"
                  : ""
                  }`}
                title="Bluetooth"
                disabled={deviceConnected}

              >
                <Bluetooth size={20} />
              </button>

              <button
                onClick={() => {
                  setActiveButton("wifi");
                  setScane(false);
                  setDevices([]);
                  ConnectwifiDevice();
                }}
                className={`transition-colors duration-300 hover:text-blue-400 ${activeButton === "wifi"
                  ? "text-green-500"
                  : ""
                  }`}
                title="WiFi"
                disabled={deviceConnected}

              >
                <Wifi size={20} />
              </button>
            </div>
            <div
              className="flex items-center px-2 font-semibold text-sm tracking-wide text-shadow-sm select-none"
              data-tauri-drag-region
            >
              Chords LSL Connector
            </div>
            {/* Right Buttons */}
            <div className="flex space-x-3">
              <button onClick={toggleAlwaysOnTop} className={`${alwaysOnTop ? "text-green-400" : "text-white"} hover:text-green-300`} title="Toggle Always on Top">
                <ChevronsUp size={20} />
              </button>
              <button onClick={minimizeWindow} className="hover:text-yellow-400" title="Minimize">
                <Minus size={20} />
              </button>
              <button onClick={closeWindow} className="hover:text-red-400" title="Close">
                <X size={20} />
              </button>
            </div>
          </div>
        </div>
      )}
      {deviceConnected && (
        <button
          onClick={disconnectFromDevice}
          className="mt-4 px-6 py-2 bg-red-500 text-black rounded-lg shadow-lg hover:bg-red-600 transition"
        >
          Disconnect
        </button>
      )}
    </div>
  );
};

export default App;