"use client";

import React, { useRef, useState } from 'react';
import { core } from "@tauri-apps/api";
import { Link ,Wifi} from 'lucide-react';

const App = () => {
  const [deviceConnected, setDeviceConnected] = useState(false);
  const portRef = useRef<unknown>(null);

  
  const isProcessing = useRef(false);

  const ConnectserialDevice = async () => {
    try {
      isProcessing.current=true;
      const portName = await core.invoke('detect_arduino') as string;
      console.log(`Connected to device on port: ${portName}`);
      portRef.current = portName;
      setDeviceConnected(true);
      await core.invoke('start_streaming', { portName: portRef.current, stream_name: "UDL" });
  

    } catch (error) {
      console.error('Failed to connect to device:', error);
    }
  };


  const ConnectwifiDevice = async () => {
    try {
      isProcessing.current=true;
        setDeviceConnected(true);
      await core.invoke("start_wifistreaming");
  

    } catch (error) {
      console.error('Failed to connect to device:', error);
    }
  };


  const handleMouseDown = (event: React.MouseEvent<HTMLDivElement>) => {
    const target = event.currentTarget;
    if (target) {
      target.classList.add("scale-95", "shadow-active");
      setTimeout(() => {
        target.classList.remove("scale-95", "shadow-active");
      }, 100); // Adjust delay as needed
    }
  };



  return (
<div className="flex flex-col items-center justify-center h-screen bg-gray-200">
  <div className="flex space-x-10">
    {/* First Button */}
    <div
      onClick={ConnectserialDevice}
      onMouseDown={handleMouseDown}
      className={`
        flex items-center justify-center w-28 h-28 rounded-full cursor-pointer bg-gray-200 shadow-[8px_8px_16px_#bebebe,-8px_-8px_16px_#ffffff] 
        transition-all duration-600 relative ${isProcessing.current ? 'animate-[rotateShadow_1.5s_linear_infinite]' : ''}
      `}
      style={{ pointerEvents: isProcessing.current ? 'none' : 'auto' }}
    >
      <Link
        size={40}
        className={`transition-colors duration-300 ${
          deviceConnected ? 'text-green-500' : 'text-gray-500'
        }`}
      />
    </div>

    {/* Second Button */}
    <div
      onClick={ConnectwifiDevice} // Different function for second button
      onMouseDown={handleMouseDown}
      className={`
        flex items-center justify-center w-28 h-28 rounded-full cursor-pointer bg-gray-200 shadow-[8px_8px_16px_#bebebe,-8px_-8px_16px_#ffffff] 
        transition-all duration-600 relative ${isProcessing.current ? 'animate-[rotateShadow_1.5s_linear_infinite]' : ''}
      `}
      style={{ pointerEvents: isProcessing.current ? 'none' : 'auto' }}
    >
      <Wifi
        size={40}
        className={`transition-colors duration-300 ${
          deviceConnected ? 'text-red-500' : 'text-gray-500'
        }`}
      />
    </div>
  </div>
</div>

  );
};

export default App;
