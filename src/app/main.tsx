"use client";
import React, { useRef, useState, useEffect } from 'react';
import { core } from "@tauri-apps/api";
import { Link, Wifi, Bluetooth, X, Usb, Github } from 'lucide-react';
import { listen } from "@tauri-apps/api/event";
import { open } from '@tauri-apps/plugin-shell';
import { SmoothieChart, TimeSeries } from 'smoothie';
import { FontAwesomeIcon } from '@fortawesome/react-fontawesome';
import { faWindows, faApple, faDebian } from '@fortawesome/free-brands-svg-icons';

const App = () => {
  const [deviceConnected, setDeviceConnected] = useState(false);
  const portRef = useRef<unknown>(null);
  const [activeButton, setActiveButton] = useState<"serial" | "wifi" | "bluetooth" | null>(null);
  const [devices, setDevices] = useState<{ name: string; id: string }[]>([]);
  const [samplerate, setSamplerate] = useState<number | undefined>(0);
  const [samplelost, setSamplelost] = useState<number | undefined>(0);
  const [connecting, setconnecting] = useState(false);
  const [totalSample, setTotalSample] = useState(0);
  const isProcessing = useRef(false);
  const [scane, setScane] = useState(false);
  const chartRef = useRef<HTMLCanvasElement>(null);
  const smoothieChartRef = useRef<SmoothieChart | null>(null);
  const timeSeriesRef = useRef<TimeSeries | null>(null);


  const ConnectserialDevice = async () => {
    try {
      setconnecting(true);
      setTotalSample(0);
      isProcessing.current = true;
      const portName = await core.invoke('detect_arduino') as string;
      portRef.current = portName;
      await core.invoke('start_streaming', { portName: portRef.current, stream_name: "UDL" });
      setDeviceConnected(true);
      setconnecting(false);
    } catch (error) {
      console.error('Failed to connect to device:', error);
    }
  };

  const ConnectwifiDevice = async () => {
    try {
      setconnecting(true);
      setTotalSample(0);
      await core.invoke("start_wifistreaming");
      isProcessing.current = true;
   
    } catch (error) {
      console.error('Failed to connect to device:', error);
    }
  };

  const ConnectbluetoothDevice = async () => {
    try {
      setScane(true);
      setTotalSample(0);
      isProcessing.current = true;
      await core.invoke("scan_ble_devices");
    } catch (error) {
      console.error("Failed to connect to device:", error);
    }
  };
  const createChart = () => {
    if (!chartRef.current) return;

    // Create a new SmoothieChart with slower scrolling
    smoothieChartRef.current = new SmoothieChart({
      limitFPS: 20,
      grid: {
        strokeStyle: 'rgb(75, 85, 99)',
        lineWidth: 1,
        millisPerLine: 8000,
        verticalSections: 6,
        borderVisible: false
      },

      labels: {
        fillStyle: '#ffffff',           // White text
        fontSize: 12,
        precision: 0,                   // No decimal places
        showIntermediateLabels: true,  // Show intermediate ticks (optional)
        disabled: false                // <-- ENABLE LABELS
      },
      tooltip: true,
      tooltipLine: {
        lineWidth: 1,
        strokeStyle: '#00FF00'
      },
      maxValue: 1000,
      minValue: 0,
      responsive: true,
      millisPerPixel: 100,
      yRangeFunction: () => ({ min: 0, max: 1000 })
    });

    // Create and add TimeSeries
    timeSeriesRef.current = new TimeSeries();
    smoothieChartRef.current.addTimeSeries(timeSeriesRef.current, {
      strokeStyle: 'rgb(0, 255, 0)',
      lineWidth: 2,
      fillStyle: 'rgba(0, 255, 0, 0.1)'
    });

    // Stream to canvas with faster updates for smoother rendering
    smoothieChartRef.current.streamTo(chartRef.current, 50);
  };
  useEffect(() => {
    if (deviceConnected) {
      createChart();

      // Cleanup function
      return () => {
        if (smoothieChartRef.current && chartRef.current) {
          smoothieChartRef.current.stop();
          smoothieChartRef.current = null;
          timeSeriesRef.current = null;
        }
      };
    }
  }, [deviceConnected]); // This effect runs when deviceConnected changes
  useEffect(() => {

    const unlistenFns: (() => void)[] = [];

    const setupListeners = async () => {


      const unlistenBleDevices = await listen('bleDevices', (event) => {
        const devices = event.payload as { name: string; id: string }[];

        setDevices(devices);
      });
      unlistenFns.push(unlistenBleDevices);

      const unlistenSamplerate = await listen('samplerate', (event) => {
        const value = Math.ceil(Number(event.payload));
        setSamplerate(value);
        setTotalSample(prev => prev + value);

        if (timeSeriesRef.current) {
          const now = Date.now();
          timeSeriesRef.current.append(now, value);
          console.log(`Appended: ${now}, ${value}`);
        }
      });

      unlistenFns.push(unlistenSamplerate);

      const unlistenconnection = await listen('connection', (event) => {
        setDeviceConnected(true);
        setconnecting(false);
      });

      unlistenFns.push(unlistenconnection);
      
      const unlistenSamplelost = await listen('samplelost', (event) => {
        setSamplelost(event.payload as number);
      });
      unlistenFns.push(unlistenSamplelost);
    };

    setupListeners();

    return () => {
      for (const unlisten of unlistenFns) {
        unlisten();
      }
    };
  }, []);

  const getFirmwareIcon = (type: string) => {
    switch (type) {
      case "bluetooth":
        return <Bluetooth className="mr-2" />;
      case "serial":
        return <Usb className="mr-2" />;
      case "wifi":
        return <Wifi className="mr-2" />;
      default:
        return <Bluetooth className="mr-2" />;
    }
  };

  const handleClick = async (url: string) => {
    await open(url);
  };

  return (
    <>
      <div className=" flex-col bg-gray-200 overflow-hidden">
        <div className="bg-gray-800   p-6 ">
          <div className="flex justify-between items-center mb-4">
            <h2 className="flex items-center text-2xl font-semibold text-white gap-2">
              <div className="font-rancho font-bold text-2xl duration-300 bg-gradient-to-r from-pink-500 via-purple-500 to-blue-500 text-transparent bg-clip-text">
                Chords
              </div>
              LSL Connector </h2>
            <button
              onClick={() => handleClick("https://github.com/upsidedownlabs/Chords-Arduino-Firmware")}
              className="flex items-center cursor-pointer gap-2 px-3 py-1 bg-gray-600 hover:bg-gray-700 dark:bg-gray-500 dark:hover:bg-gray-600 text-white rounded-md text-sm transition-colors"
            >
              <Github className="h-4 w-4" /> Get Firmware
            </button>
          </div>

          <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
            {(
              [
                { type: "bluetooth", title: "Bluetooth LE", description: "For wireless Bluetooth LE connectivity" },
                { type: "serial", title: "Serial", description: "For direct wired USB serial connections" },
                { type: "wifi", title: "WiFi", description: "For wireless WiFi network connectivity" },
              ] as const
            ).map((option) => (
              <div
                key={option.type}
                onClick={() => {
                  if (activeButton) return; // Prevent all clicks when any button is active

                  setActiveButton(option.type);
                  setDevices([]);
                  switch (option.type) {
                    case "bluetooth":
                      ConnectbluetoothDevice();
                      setScane(true);
                      break;
                    case "serial":
                      ConnectserialDevice();
                      break;
                    case "wifi":
                      ConnectwifiDevice();
                      break;
                    default:
                      console.log("Unknown type selected");
                  }
                }}
                className={`group relative p-[2px] rounded-lg transition-all duration-300 
                  ${activeButton ? 'pointer-events-none' : 'cursor-pointer'}
                  ${activeButton === option.type
                    ? 'bg-gradient-to-r from-pink-500 via-purple-500 to-blue-500 animate-gradient-move'
                    : activeButton != null ? 'bg-white' : 'bg-gradient-to-r from-pink-500 via-purple-500 to-blue-500'
                  }`}
              >
                <div
                  className={`relative rounded-lg p-4 transition-all duration-200 border border-transparent bg-gray-700`}
                >
                  <div className="flex items-center text-white">
                    {getFirmwareIcon(option.type)}
                    <span className="font-medium">{option.title}</span>
                  </div>
                  <p className="text-gray-400 text-sm mt-2">{option.description}</p>
                </div>
              </div>
            ))}
          </div>
          <div className='flex w-full items-center gap-2 p-2 bg-blue-900/20 rounded-lg border border-blue-800 mt-4 '>
            <span className="text-white text-sm pl-2">Visualise/Record your signals using</span>

            <button
              onClick={() => handleClick("https://www.brainproducts.com/downloads/more-software/")}
              className="flex items-center gap-2 px-3 py-1 bg-pink-500  text-white rounded-md text-sm transition-colors cursor-pointer "
            >
              BV LSL Viewer <FontAwesomeIcon icon={faWindows} />


            </button>

            <button
              onClick={() => handleClick("https://open-ephys.org/gui")}
              className="flex items-center gap-2 px-3 py-1 bg-purple-600  text-white rounded-md text-sm transition-colors cursor-pointer"
            >
              OE GUI  <FontAwesomeIcon icon={faWindows} />
              <FontAwesomeIcon icon={faApple} />
              <FontAwesomeIcon icon={faDebian} />

            </button>
            <button
              onClick={() => handleClick("https://github.com/labstreaminglayer/App-LabRecorder/releases/tag/v1.16.5")}
              className="flex items-center gap-2 px-3 py-1 bg-blue-500 text-white rounded-md text-sm transition-colors cursor-pointer"
            >
              LabRecorder <FontAwesomeIcon icon={faWindows} />
              <FontAwesomeIcon icon={faApple} />
              <FontAwesomeIcon icon={faDebian} />

            </button>
            <button
              onClick={() => handleClick("https://labstreaminglayer.readthedocs.io/info/viewers.html")}
              className="flex items-center gap-2 px-3 py-1 bg-green-600  text-white rounded-md text-sm transition-colors cursor-pointer"
            >
              More +
            </button>
          </div>

          {deviceConnected ? (
            <>
              <div className="bg-gray-800 rounded-lg p-2 border border-gray-700 mt-2">
                <div className="mb-2">
                  <h3 className="text-lg font-medium text-white mb-2">Connection Statistics</h3>
                </div>
                <div className="h-[12.5rem] bg-black rounded-lg border border-gray-600 relative">
                  <div className="bg-gray-900 w-full h-full rounded-lg">
                    <canvas
                      ref={chartRef}
                      className="w-full h-full rounded-lg"
                    />
                  </div>

                  {/* Overlay text remains unchanged */}
                  <div className="absolute top-2 left-2 text-white text-xs space-y-1">
                    <div className="flex gap-2">
                      <span className="flex items-center gap-2 p-2 bg-blue-900/20 rounded-lg border border-blue-800">
                        Sampling Rate: {samplerate || 0} Hz
                      </span>
                      <span className="flex items-center gap-2 p-2 bg-blue-900/20 rounded-lg border border-blue-800">
                        Samples Lost: {samplelost || 0}
                      </span>
                      <span className="flex items-center gap-2 p-2 bg-blue-900/20 rounded-lg border border-blue-800">
                        Total Samples: {totalSample}
                      </span>
                    </div>
                  </div>
                </div>
              </div>
            </>
          ) : (
            <>
              <h3 className="text-lg font-medium text-white mb-2 mt-4">Connectivity Guide</h3>
              <div className="bg-gray-800 rounded-lg p-6 border border-gray-700">
                <div className="mb-4 p-2 bg-red-900/20 rounded-lg border border-red-800">
                  <div className="flex items-center space-x-2">
                    <div className="w-2 h-2 bg-red-500 rounded-full"></div>
                    <p className="text-white text-sm font-medium">
                      Device Not connected, please follow the steps below to connect your device.
                    </p>
                  </div>
                </div>

                <div className="space-x-6 flex">
                  <div className="flex-1 bg-gray-700 rounded-lg p-4 border border-gray-600">
                    <div className="flex items-start space-x-3">
                      <div className="flex-shrink-0 w-8 h-8 bg-blue-900 rounded-full flex items-center justify-center">
                        <span className="text-blue-200 font-semibold text-sm">1</span>
                      </div>
                      <div>
                        <h5 className="font-medium text-white mb-2">
                          Upload Chords Arduino Firmware
                        </h5>
                        <p className="text-gray-400 text-sm">
                          Visit <a className="underline text-blue-400" href="https://github.com/upsidedownlabs/Chords-Arduino-Firmware" target="_blank" rel="noopener noreferrer">Chords Arduino Firmware repository</a> and upload the firmware for your development board.
                        </p>
                      </div>
                    </div>
                  </div>

                  <div className="flex-1 bg-gray-700 rounded-lg p-4 border border-gray-600">
                    <div className="flex items-start space-x-3">
                      <div className="flex-shrink-0 w-8 h-8 bg-green-900 rounded-full flex items-center justify-center">
                        <span className="text-green-400 font-semibold text-sm">2</span>
                      </div>
                      <div>
                        <h5 className="font-medium text-white mb-2">
                          Connect your development board
                        </h5>
                        <p className="text-gray-400 text-sm">
                          Connect your board via Serial/BLE/WiFi and follow steps on screen to start the LSL stream.
                        </p>
                      </div>
                    </div>
                  </div>
                </div>
              </div>
            </>
          )}
        </div>
        {scane && (
          <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
            <div className="bg-white dark:bg-gray-800 rounded-lg shadow-xl max-w-md w-full mx-4 border border-gray-200 dark:border-gray-700 max-h-[90vh] flex flex-col">
              {/* Header */}
              <div className="border-b border-gray-200 dark:border-gray-700 p-4 flex justify-between items-center flex-shrink-0">
                <div>
                  <h2 className="text-xl font-semibold text-gray-900 dark:text-white">Available Devices</h2>
                </div>
                <button
                  onClick={() => {
                    setScane(false);
                  }}
                  className="text-gray-400 hover:text-gray-600 dark:hover:text-gray-300 transition-colors"
                >
                  <X className="h-5 w-5" />
                </button>
              </div>

              {/* Scrollable device list */}
              <div className="p-6 text-gray-900 dark:text-white overflow-y-auto flex-1 min-h-0">
                {devices.length > 0 ? (
                  <div className="space-y-2">
                    {devices.map((device) => (
                      <div
                        key={device.id}
                        className="border border-gray-300 dark:border-gray-600 rounded-lg p-3 cursor-pointer hover:bg-gray-50 dark:hover:bg-gray-700 flex justify-between items-center transition-colors"
                        onClick={async () => {
                          await core.invoke<string>("connect_to_ble", { deviceId: device.id });
                          setDeviceConnected(true);
                          setScane(false);
                        }}
                      >
                        <div className="flex items-center">
                          <div className="h-5 w-5 mr-3 bg-blue-100 dark:bg-blue-900/20 rounded-full flex items-center justify-center">
                            <div className="h-2 w-2 bg-blue-600 dark:bg-blue-400 rounded-full animate-pulse"></div>
                          </div>
                          <span className="text-gray-900 dark:text-white truncate">
                            {device.name || `Unknown Device (${device.id})`}
                          </span>
                        </div>

                        <Link className="h-5 w-5 text-blue-600 dark:text-blue-400 flex-shrink-0 ml-2" />
                      </div>
                    ))}
                  </div>
                ) : (
                  <div className="text-center">
                    <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600 dark:border-blue-400 mx-auto mb-4"></div>
                    <p className="text-gray-600 dark:text-gray-400">Scanning for devices...</p>
                  </div>
                )}
              </div>
            </div>
          </div>
        )}
        {connecting && (
          <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
            <div className="bg-white dark:bg-gray-800 rounded-lg shadow-xl max-w-md w-full mx-4 border border-gray-200 dark:border-gray-700 max-h-[90vh] flex flex-col">
              {/* Scrollable device list */}
              <div className="p-6 text-gray-900 dark:text-white overflow-y-auto flex-1 min-h-0">
                <div className="text-center">
                  <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600 dark:border-blue-400 mx-auto mb-4"></div>
                  <p className="text-gray-600 dark:text-gray-400">Connecting...</p>
                </div>
              </div>
            </div>
          </div>
        )}
      </div>
    </>
  );
};

export default App;