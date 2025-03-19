"use client";

import React, { useRef, useState } from 'react';
import { core } from "@tauri-apps/api";
import { Link, Wifi } from 'lucide-react';

const App = () => {
  const [deviceConnected, setDeviceConnected] = useState(false);
  const portRef = useRef<unknown>(null);
  const [activeButton, setActiveButton] = useState<"serial" | "wifi" | null>(null);


  const isProcessing = useRef(false);

  const ConnectserialDevice = async () => {
    try {
      isProcessing.current = true;
      setActiveButton("serial");
      const portName = await core.invoke('detect_arduino') as string;
      console.log(`Connected to device on port: ${portName}`);
      portRef.current = portName;
      setDeviceConnected(true);
      await core.invoke('start_streaming', { portName: portRef.current, stream_name: "UDL" });


    } catch (error) {
      console.error('Failed to connect to device:', error);
    }
  };



  return (
    <div className="flex flex-col items-center justify-center h-screen bg-gray-200">
      <div className="flex space-x-10">
        {/* First Button */}
        <div
          onClick={activeButton === null ? ConnectserialDevice : undefined}
          onMouseDown={activeButton === null ? () => (isProcessing.current = true) : undefined}
          className={`
            flex items-center justify-center w-28 h-28 rounded-full cursor-pointer bg-gray-200 shadow-[8px_8px_16px_#bebebe,-8px_-8px_16px_#ffffff] 
            transition-all duration-600 relative ${activeButton && activeButton !== "wifi" ?
               'animate-[rotateShadow_1.5s_linear_infinite]' : ''}
            ${activeButton && activeButton !== "serial" ? "opacity-50 pointer-events-none" : ""}
          `}
        >
          <Link
            size={40}
            className={`transition-colors duration-300 ${
              deviceConnected && activeButton === "serial" ? "text-green-500" : "text-gray-500"
            }`}
          />
        </div>
      </div>
    </div>

  );
};

export default App;
