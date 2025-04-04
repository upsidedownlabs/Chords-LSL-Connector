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
        <div className="flex  relative  ">
          <div className="w-full md:w-1/2 flex flex-col">
            <div className="flex-1  grid place-items-center bg-slate-50 ">
              {(!scane) && (<>
                {activeButton ? (
                  {
                    serial: (
                      <Link
                        size={50}
                        className={`transition-colors duration-300 ${deviceConnected && activeButton === "serial"
                          ? "text-green-500"
                          : "text-gray-500"
                          }`}
                      />
                    ),
                    bluetooth: (
                      <Bluetooth
                        size={50}
                        className={`transition-colors duration-300 ${deviceConnected && activeButton === "bluetooth"
                          ? "text-green-500"
                          : "text-gray-500"
                          }`}
                      />
                    ),
                    wifi: (
                      <Wifi
                        size={50}
                        className={`transition-colors duration-300 ${deviceConnected && activeButton === "wifi"
                          ? "text-green-500"
                          : "text-gray-500"
                          }`}
                      />
                    ),
                  }[activeButton]
                ) : (
                  <div className="text-gray-400 text-sm text-center">
                    <p>Select a</p>
                    <p>connection</p>
                  </div>
                )}
              </>
              )}
              {scane && !deviceConnected && (
                <div className="max-w-md relative  rounded overflow-hidden">
                  {/* Scrollable device list */}
                  <div className="max-h-[60vh] overflow-y-auto">
                    {devices.length > 0 ? (
                      <ul className=""> {/* Extra padding for visible last item */}
                        {devices.map((device) => (
                          <li
                            key={device.id}
                            className="flex items-center pl-1  hover:bg-gray-100"
                          >

                            <label
                              htmlFor={`device-${device.id}`}
                              className="flex-1 text-gray-700 cursor-pointer"
                            >
                              {device.name || `Unknown Device (${device.id})`}
                            </label>
                            <button
                              onClick={async () => {
                                const response = await core.invoke<string>("connect_to_ble", { deviceId: device.id });
                                setStatus(response);
                                setDeviceConnected(true);
                                setScane(false);
                              }}
                              className="ml-2 bg-blue-500 hover:bg-blue-600 text-black mr-3 rounded text-sm transition-colors px-1 py-1"
                            >
                              <Link size={14

                              } />
                            </button>

                          </li>
                        ))}
                      </ul>
                    ) : (
                      <p className="
                      max-h-[60vh] text-black
                      ">Scanning for devices...</p>
                    )}
                  </div>




                </div>
              )}




            </div>

          </div>

          <div className="absolute left-1/2 top-0 transform -translate-x-1/2 w-px h-full bg-slate-200 z-10 md:static md:w-px md:h-full"></div>

          <div className="w-full md:w-1/2 flex flex-col">
            <div className="flex flex-col h-full">
              <div className="flex flex-col justify-center pl-2 border-x border-b  border-slate-200 bg-slate-50 shadow-sm min-h-[20px]  transition-all">
                <p className='text-black '>
                  <span className="text-lg  font-semibold">Status: </span>{status || "Not Connected"}
                </p>
              </div>
              <div className=" flex flex-col justify-center pl-2  border border-slate-200 bg-slate-50 shadow-sm min-h-[20px]   transition-all">
                <p className='text-black '>
                  <span className="text-lg  font-semibold">Sampling Rate:  </span>{samplerate || "No "}
                </p>
              </div>
              <div className=" flex flex-col justify-center pl-2  border border-slate-200 bg-slate-50 shadow-sm min-h-[20px]  transition-all">
                <p className='text-black '>
                  <span className="text-lg  font-semibold">Sample Lost: </span>{samplelost || "no"}
                </p>
              </div>
              <div className=" flex flex-col justify-center  pl-2 border-x border-t border-slate-200 bg-slate-50 shadow-sm min-h-[20px]  transition-all">
                <p className='text-black '>
                  <span className="text-lg  font-semibold">LSL: </span>{lsl || "No lsl yet"}
                </p>
              </div>
            </div>
          </div>
        </div>


      </div>
    </>
  );
};

export default App;