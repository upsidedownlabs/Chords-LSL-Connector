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
      if (activeButton) return; // Prevent clicking if any button is active (including the current one)
      
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
    className={`group relative p-[2px] rounded-lg bg-gradient-to-r from-pink-500 via-purple-500 to-blue-500 transition-all duration-300 
      ${activeButton ? 'opacity-50 pointer-events-none' : ''}`}
  >
    <div
      className={`relative rounded-lg p-4 transition-all duration-200 border ${
        activeButton === option.type
          ? 'border-blue-500 ring-2 ring-blue-400/30 bg-blue-900/20 cursor-not-allowed'
          : 'border-transparent bg-gray-700 hover:bg-gray-600 cursor-pointer'
      } ${
        activeButton && activeButton !== option.type ? 'cursor-not-allowed' : ''
      }`}
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
