use btleplug::api::WriteType;
use btleplug::platform::Peripheral;
use futures::future::ok;
use futures::StreamExt; // Changed from futures_util to futures
use lazy_static::lazy_static;
use lsl::Pushable; // Add the necessary imports
use lsl::StreamOutlet;
use lsl::{ChannelFormat, StreamInfo};
use serde_json::json; // Add this import at the top
use serialport;
use std::io::{self, Read, Write};
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use tauri::{self, AppHandle, Emitter}; // Import Emitter along with AppHandle
use tokio::sync::mpsc;
use tungstenite::connect;
use tungstenite::protocol::Message;
<<<<<<< HEAD
use url::Url;
=======

use std::sync::atomic::{AtomicU8, Ordering};
use btleplug::api::WriteType;
use futures::StreamExt;  // Changed from futures_util to futures
use lazy_static::lazy_static;
use std::sync::{Arc, Mutex};
>>>>>>> 98cafaa (Implement BLE)
lazy_static! {
    static ref BAUDRATE: Arc<Mutex<u32>> = Arc::new(Mutex::new(230400)); // Default baud rate
    static ref PACKET_SIZE: Arc<Mutex<usize>> = Arc::new(Mutex::new(16)); // Default baud rate
    static ref CHANNELS: Arc<Mutex<usize>> = Arc::new(Mutex::new(6)); // Default baud rate
    static ref SAMPLE_RATE: Arc<Mutex<f64>> = Arc::new(Mutex::new(500.0)); // Default baud rate
}
use tauri::Manager;

#[tauri::command]
async fn detect_arduino() -> Result<String, String> {
    tokio::task::spawn_blocking(|| detect_arduino_internal())
        .await
        .map_err(|e| format!("Task panicked: {:?}", e))?
}

// Your original function renamed to detect_arduino_internal
fn detect_arduino_internal() -> Result<String, String> {
    loop {
        let ports = serialport::available_ports().expect("No ports found!");

        for port_info in ports {
            let port_name = port_info.port_name;
            println!("Attempting to connect to port: {}", port_name);

            if port_name.contains("BLTH")
                || port_name.contains("Bluetooth")
                || port_name.contains("console")
            {
                continue;
            }
            if let serialport::SerialPortType::UsbPort(info) = port_info.port_type {
                // Check if the VID matches your Arduino device
                if info.vid == 6790 || matches!(info.pid, 67 | 579 | 29987 | 66 | 24577) {
                    *BAUDRATE.lock().unwrap() = 115200; // Change the baud rate dynamically
                    *SAMPLE_RATE.lock().unwrap() = 250.0;
                }
            }

            match serialport::new(&port_name, *BAUDRATE.lock().unwrap())
                .timeout(Duration::from_secs(3))
                .open()
            {
                Ok(mut port) => {
                    thread::sleep(Duration::from_secs(3)); // Allow Arduino to reset
                    let command = b"WHORU\n";

                    if let Err(e) = port.write_all(command) {
                        println!("Failed to write to port: {}. Error: {:?}", port_name, e);
                        continue;
                    }
                    port.flush().expect("Failed to flush port");

                    let mut buffer: Vec<u8> = vec![0; 1024];
                    let mut response = String::new();
                    let start_time = Instant::now();
                    let timeout = Duration::from_secs(10);

                    while start_time.elapsed() < timeout {
                        match port.read(&mut buffer) {
                            Ok(size) => {
                                if size > 0 {
                                    response.push_str(&String::from_utf8_lossy(&buffer[..size]));
                                    if response.contains("UNO-R4")
                                        || response.contains("UNO-R3")
                                        || response.contains("GIGA-R1")
                                        || response.contains("RPI-PICO-RP2040")
                                        || response.contains("UNO-CLONE")
                                        || response.contains("NANO-CLONE")
                                        || response.contains("MEGA-2560-R3")
                                        || response.contains("MEGA-2560-CLONE")
                                        || response.contains("GENUINO-UNO")
                                        || response.contains("NANO-CLASSIC")
                                        || response.contains("STM32G4-CORE-BOARD")
                                        || response.contains("STM32F4-BLACK-PILL")
                                        || response.contains("NPG-LITE")
                                    {
                                        if response.contains("NANO-CLONE")
                                            || response.contains("NANO-CLASSIC")
                                            || response.contains("STM32F4-BLACK-PILL")
                                        {
                                            *PACKET_SIZE.lock().unwrap() = 20; // Change the baud rate dynamically
                                            *CHANNELS.lock().unwrap() = 8; // Change the baud rate dynamically
                                        }
                                        if response.contains("MEGA-2560-R3")
                                            || response.contains("MEGA-2560-CLONE")
                                            || response.contains("STM32G4-CORE-BOARD")
                                        {
                                            *PACKET_SIZE.lock().unwrap() = 36; // Change the baud rate dynamically
                                            *CHANNELS.lock().unwrap() = 16; // Change the baud rate dynamically
                                        }
                                        if response.contains("RPI-PICO-RP2040")
                                            || response.contains("NPG-LITE")
                                        {
                                            *PACKET_SIZE.lock().unwrap() = 10; // Change the baud rate dynamically
                                            *CHANNELS.lock().unwrap() = 3; // Change the baud rate dynamically
                                        }
                                        println!("Valid device found on port: {}", port_name);
                                        drop(port);
                                        return Ok(port_name); // Return the found port name directly
                                    }
                                }
                            }
                            Err(ref e) if e.kind() == io::ErrorKind::TimedOut => continue,
                            Err(e) => {
                                println!("Failed to read from port: {}. Error: {:?}", port_name, e);
                                break;
                            }
                        }
                    }
                    println!("Final response from port {}: {}", port_name, response);

                    drop(port);
                }
                Err(e) => {
                    println!("Failed to open port: {}. Error: {:?}", port_name, e);
                }
            }
        }

        println!("No valid device found, retrying in 5 seconds...");
        thread::sleep(Duration::from_secs(5)); // Wait before trying again
    }
}

#[tauri::command]
async fn start_streaming(port_name: String, app_handle: AppHandle) {
    const START_BYTE_1: u8 = 0xC7;
    const START_BYTE_2: u8 = 0x7C;
    const END_BYTE: u8 = 0x01;

    // Create a channel for communication
    let (tx, mut rx) = mpsc::channel::<Vec<i16>>(100);

    // Create StreamInfo as before
    let info = Arc::new(
        lsl::StreamInfo::new(
            "UDL",
            "Biopotential_Signals",
            (*CHANNELS.lock().unwrap()).try_into().unwrap(),
            *SAMPLE_RATE.lock().unwrap(),
            lsl::ChannelFormat::Int16,
            "Chords",
        )
        .unwrap(),
    );

    // Create StreamOutlet in the same thread
    let (tx, rx) = std::sync::mpsc::channel::<Vec<i16>>();
    let outlet = Arc::new(Mutex::new(StreamOutlet::new(&info, 0, 360).unwrap()));

    // Use spawn_blocking to handle the task in a separate thread
    tokio::task::spawn_blocking(move || loop {
        match serialport::new(&port_name, *BAUDRATE.lock().unwrap())
            .timeout(Duration::from_secs(3))
            .open()
        {
            Ok(mut port) => {
                let start_command = b"START\r\n";

                for i in 1..=3 {
                    if let Err(e) = port.write_all(start_command) {}
                    println!("Connected to device on port: {}", port_name);
                    thread::sleep(Duration::from_millis(1000));
                }

                println!("Finished sending commands.");

                let mut buffer: Vec<u8> = vec![0; 1024];
                let mut accumulated_buffer: Vec<u8> = Vec::new();

                let mut packet_count = 0;
                let mut sample_count = 0;
                let mut byte_count = 0;
                let start_time = Instant::now();
                let mut last_print_time = Instant::now();
                packet_count += 1;

                loop {
                    match port.read(&mut buffer) {
                        Ok(size) => {
                            accumulated_buffer.extend_from_slice(&buffer[..size]);
                            byte_count += size;

                            while accumulated_buffer.len() >= *PACKET_SIZE.lock().unwrap() {
                                if accumulated_buffer[0] == START_BYTE_1
                                    && accumulated_buffer[1] == START_BYTE_2
                                {
                                    if accumulated_buffer[*PACKET_SIZE.lock().unwrap() - 1]
                                        == END_BYTE
                                    {
                                        let packet = accumulated_buffer
                                            .drain(..*PACKET_SIZE.lock().unwrap())
                                            .collect::<Vec<u8>>();
                                        sample_count += 1;
                                        let data: Vec<i16> = (0..*CHANNELS.lock().unwrap())
                                            .map(|i| {
                                                let idx = 3 + (i * 2);
                                                let high = packet[idx] as i16;
                                                let low = packet[idx + 1] as i16;
                                                (high << 8) | low
                                            })
                                            .collect();
                                        println!("Received raw data: {:?}", data);

                                        if tx.send(data).is_err() {
                                            println!("Failed to send data to the main thread.");
                                            break;
                                        }
                                    } else {
                                        accumulated_buffer.drain(..1);
                                    }
                                } else {
                                    accumulated_buffer.drain(..1);
                                }
                            }

                            if last_print_time.elapsed() >= Duration::from_secs(1) {
                                let elapsed = start_time.elapsed().as_secs_f32();
                                let refresh_rate = format!("{:.2}", packet_count as f32 / elapsed);
                                let samples_per_second =
                                    format!("{:.2}", sample_count as f32 / elapsed);
                                let bytes_per_second =
                                    format!("{:.2}", byte_count as f32 / elapsed);
                                let _ = app_handle.emit("connection", "Connected ");
                                let _ = app_handle.emit("samplerate", samples_per_second);
                                let _ = app_handle.emit("lsl", "uidserial007");
                                last_print_time = Instant::now();
                            }
                        }
                        Err(ref e) if e.kind() == io::ErrorKind::TimedOut => {
                            println!("Read timed out, retrying...");
                            continue;
                        }
                        Err(e) => {
                            println!("Error receiving data: {:?}", e);
                            break;
                        }
                    }
                }
            }
            Err(e) => {
                println!("Failed to connect to device on {}: {}", port_name, e);
                break;
            }
        }

        println!("Device disconnected, checking for new devices...");
        thread::sleep(Duration::from_secs(5));
    });
    while let Ok(data) = rx.recv() {
        if let Ok(outlet) = outlet.lock() {
            outlet.push_sample(&data).unwrap_or_else(|e| {
                println!("Failed to push data to LSL: {:?}", e);
            });
        }
    }
}

fn calculate_rate(data_size: usize, elapsed_time: f64) -> f64 {
    data_size as f64 / elapsed_time
}

#[tauri::command]
async fn start_wifistreaming(app_handle: AppHandle) {
    let stream_name = "NPG-Lite";
    let info = StreamInfo::new(
        stream_name,
        "EXG",
        3,
        500.0,
        ChannelFormat::Float32,
        "uidwifi007",
    )
    .unwrap();
    let outlet = StreamOutlet::new(&info, 0, 360).unwrap();
    let ws_url = "ws://multi-emg.local:81";
    let (mut socket, _) =
        connect(Url::parse(ws_url).unwrap()).expect("WebSocket connection failed");
    println!("{} WebSocket connected!", stream_name);

    let mut block_size = 13;
    let mut packet_size = 0;
    let mut data_size = 0;
    let mut sample_size = 0;
    let mut previous_sample_number: Option<u8> = None;
    let mut previous_data = vec![];
    let start_time = Instant::now();

    loop {
        match socket.read_message() {
            Ok(Message::Binary(data)) => {
                data_size += data.len();
                let elapsed_time = start_time.elapsed().as_secs_f64();
                let mut samplelost = 0;
                if elapsed_time >= 1.0 {
                    let samples_per_second = calculate_rate(sample_size, elapsed_time);
                    let refresh_rate = calculate_rate(packet_size, elapsed_time);
                    let bytes_per_second = calculate_rate(data_size, elapsed_time);
                    println!(
                        "{} FPS : {} SPS : {} BPS",
                        refresh_rate.ceil(),
                        samples_per_second.ceil(),
                        bytes_per_second.ceil()
                    );
                    packet_size = 0;
                    sample_size = 0;
                    data_size = 0;
                }

                packet_size += 1;
                println!("Packet size: {} Bytes", data.len());

                for block_location in (0..data.len()).step_by(block_size) {
                    sample_size += 1;
                    let block = &data[block_location..block_location + block_size];
                    let sample_number = block[0];
                    let mut channel_data = vec![];

                    for channel in 0..3 {
                        let offset = 1 + (channel * 2);
                        let sample = i16::from_be_bytes([block[offset], block[offset + 1]]);
                        channel_data.push(sample as f32);
                    }

                    if let Some(prev) = previous_sample_number {
                        if sample_number.wrapping_sub(prev) > 1 {
                            println!("Error: Sample Lost");
                            samplelost += 1;
                            break;
                        } else if sample_number == prev {
                            println!("Error: Duplicate Sample");
                            break;
                        }
                    }
                    let _ = app_handle.emit("samplelost", samplelost);

                    previous_sample_number = Some(sample_number);
                    previous_data = channel_data.clone();

                    println!("EEG Data: {} {:?}", sample_number, channel_data);
                    outlet.push_sample(&channel_data).unwrap();
                    let _ = app_handle.emit("connection", "Connected ");
                    let _ =
                        app_handle.emit("samplerate", calculate_rate(sample_size, elapsed_time));
                    let _ = app_handle.emit("lsl", "uidwifi 007");
                }
            }
            Ok(_) => {} // Ignore non-binary messages
            Err(e) => {
                eprintln!("WebSocket error: {:?}", e);
                break;
            }
        }
        thread::sleep(Duration::from_millis(1));
    }
}
use btleplug::api::{Central, Manager as _, Peripheral as _, ScanFilter};
<<<<<<< HEAD
use btleplug::platform::Manager as BtleManager;
=======
use btleplug::platform::Manager;
use tokio::time;

// #[tauri::command]
// async fn scan_bluetooth_devices(app_handle: AppHandle) -> Result<(), String> {
//     let manager = Manager::new().await.map_err(|e| e.to_string())?;
//     let adapters = manager.adapters().await.map_err(|e| e.to_string())?;

//     for adapter in adapters {
//         adapter.start_scan(ScanFilter::default()).await.map_err(|e| e.to_string())?;
//         time::sleep(Duration::from_secs(5)).await; // Wait for scan results

//         let peripherals = adapter.peripherals().await.map_err(|e| e.to_string())?;
//         let mut devices = Vec::new();

//         for peripheral in peripherals {
//             if let Some(properties) = peripheral.properties().await.unwrap() {
//                 if let Some(local_name) = properties.local_name {
//                     let device_info = serde_json::json!({
//                         "name": local_name,
//                         "id": peripheral.id().to_string()
//                     });

//                     devices.push(device_info);
//                     println!("{:#?}", devices);
//                 }
//             }
//         }

//         // Emit the list of devices to the frontend
//         app_handle.emit("bluetoothDevices", devices).map_err(|e| e.to_string())?;
//     }
//     Ok(())
// }

>>>>>>> 98cafaa (Implement BLE)

// Thread-safe wrapper for StreamOutlet
struct SafeOutlet(Option<StreamOutlet>);
unsafe impl Send for SafeOutlet {}
unsafe impl Sync for SafeOutlet {}

// Constants for BLE communication
const SINGLE_SAMPLE_LEN: usize = 7;
const BLOCK_COUNT: usize = 10;
const NEW_PACKET_LEN: usize = SINGLE_SAMPLE_LEN * BLOCK_COUNT;

// Global state for BLE
lazy_static! {
    static ref BLE_OUTLET: Arc<Mutex<SafeOutlet>> = Arc::new(Mutex::new(SafeOutlet(None)));
    static ref BLE_SAMPLE_COUNTER: AtomicU8 = AtomicU8::new(0);
    static ref BLE_CONNECTED: Arc<Mutex<bool>> = Arc::new(Mutex::new(false));
}

// Create BLE LSL outlet
fn create_ble_outlet() -> Result<(), String> {
    let info = StreamInfo::new(
        "NPG-Lite",
        "EXG",
        3,
        500.0,
        ChannelFormat::Float32,
        "uidbluetooth007",
    )
    .map_err(|e| e.to_string())?;
    let outlet = StreamOutlet::new(&info, 0, 360).map_err(|e| e.to_string())?;
    *BLE_OUTLET.lock().unwrap() = SafeOutlet(Some(outlet));
    Ok(())
}

// Close BLE LSL outlet
fn close_ble_outlet() {
    *BLE_OUTLET.lock().unwrap() = SafeOutlet(None);
    BLE_SAMPLE_COUNTER.store(0, Ordering::Relaxed);
    *BLE_CONNECTED.lock().unwrap() = false;
}

// Process BLE samples
<<<<<<< HEAD
fn process_ble_sample(sample: &[u8], app_handle: AppHandle) -> Result<Vec<f32>, String> {
=======
fn process_ble_sample(sample: &[u8]) -> Result<Vec<f32>, String> {
>>>>>>> 98cafaa (Implement BLE)
    if sample.len() != SINGLE_SAMPLE_LEN {
        return Err("Invalid sample length".to_string());
    }
    let mut samplelost = 0;
    let sample_counter = sample[0];
    let prev = BLE_SAMPLE_COUNTER.load(Ordering::Relaxed);
    let expected = prev.wrapping_add(1);
    if sample_counter != expected {
        samplelost += 1;

        println!(
            "Sample counter discontinuity: expected {}, got {}",
            expected, sample_counter
        );
    }
    BLE_SAMPLE_COUNTER.store(sample_counter, Ordering::Relaxed);
    let _ = app_handle.emit("samplelost", samplelost);

    Ok(vec![
        i16::from_be_bytes([sample[1], sample[2]]) as f32,
        i16::from_be_bytes([sample[3], sample[4]]) as f32,
        i16::from_be_bytes([sample[5], sample[6]]) as f32,
    ])
}

#[tauri::command]
async fn scan_ble_devices(app_handle: AppHandle) -> Result<(), String> {
<<<<<<< HEAD
    let manager = BtleManager::new()
        .await
        .map_err(|e| format!("Manager creation failed: {}", e))?;

    let adapter = manager
        .adapters()
        .await
        .map_err(|e| format!("Failed to get adapters: {}", e))?
        .into_iter()
        .next()
        .ok_or("No Bluetooth adapters found".to_string())?;

    println!(
        "Using adapter: {}",
        adapter.adapter_info().await.map_err(|e| e.to_string())?
    );

    adapter
        .start_scan(ScanFilter::default())
        .await
        .map_err(|e| format!("Failed to start scan: {}", e))?;

    println!("Scanning for BLE devices...");
    tokio::time::sleep(Duration::from_secs(5)).await;

    let peripherals = adapter
        .peripherals()
        .await
=======
    let manager = Manager::new().await.map_err(|e| format!("Manager creation failed: {}", e))?;
    
    // Get the first adapter (you might want to handle multiple adapters differently)
    let adapter = manager.adapters().await
        .map_err(|e| format!("Failed to get adapters: {}", e))?
        .into_iter()
        .next()
        .ok_or("No Bluetooth adapters found".to_string())?;

    println!("Using adapter: {}", adapter.adapter_info().await.map_err(|e| e.to_string())?);

    // Start scan with timeout
    adapter.start_scan(ScanFilter::default()).await
        .map_err(|e| format!("Failed to start scan: {}", e))?;
    
    println!("Scanning for BLE devices...");
    tokio::time::sleep(Duration::from_secs(5)).await;

    // Get peripherals
    let peripherals = adapter.peripherals().await
>>>>>>> 98cafaa (Implement BLE)
        .map_err(|e| format!("Failed to get peripherals: {}", e))?;

    if peripherals.is_empty() {
        println!("No BLE devices found");
        return Err("No BLE devices found".to_string());
<<<<<<< HEAD
    }

    let mut devices = Vec::new();
    for peripheral in peripherals {
        match peripheral.properties().await {
            Ok(Some(props)) => {
                if let Some(name) = &props.local_name {
                    if name.to_lowercase().contains("npg") {
                        println!("Found matching device: {} ({})", name, peripheral.id());
                        devices.push(json!({
                            "name": name,
                            "id": peripheral.id().to_string(),
                            "rssi": props.rssi,
                            "connected": peripheral.is_connected().await.unwrap_or(false)
                        }));
                    }
                }
            }
            Ok(None) => println!("Device with no properties"),
            Err(e) => println!("Error getting properties: {}", e),
        }
    }

    if devices.is_empty() {
        println!("No BLE devices with name containing 'npg' found");
    }

    app_handle
        .emit("bleDevices", devices)
=======
    }

    let mut devices = Vec::new();
    for peripheral in peripherals {
        match peripheral.properties().await {
            Ok(Some(props)) => {
                let name = props.local_name.unwrap_or_else(|| "Unknown".to_string());
                println!("Found device: {} ({})", name, peripheral.id());
                devices.push(json!({
                    "name": name,
                    "id": peripheral.id().to_string(),
                    "rssi": props.rssi,
                    "connected": peripheral.is_connected().await.unwrap_or(false)
                }));
            }
            Ok(None) => println!("Device with no properties"),
            Err(e) => println!("Error getting properties: {}", e),
        }
    }

    app_handle.emit("bleDevices", devices)
>>>>>>> 98cafaa (Implement BLE)
        .map_err(|e| format!("Failed to emit devices: {}", e))?;

    Ok(())
}
#[tauri::command]
async fn connect_to_ble(device_id: String, app_handle: AppHandle) -> Result<String, String> {
<<<<<<< HEAD
<<<<<<< HEAD
    println!("[CONNECT] Starting connection to device: {}", device_id);
=======
>>>>>>> 98cafaa (Implement BLE)
    close_ble_outlet();

    // 1. Initialize Bluetooth Manager
    let manager = match BtleManager::new().await {
        Ok(m) => {
            println!("[MANAGER] Bluetooth manager initialized");
            m
        }
        Err(e) => {
            println!("[ERROR] Manager creation failed: {}", e);
            return Err(format!("Bluetooth initialization failed: {}", e));
        }
    };
=======
    println!("[DEBUG] Starting connect_to_ble for device: {}", device_id);
    close_ble_outlet();

    // Initialize manager
    let manager = match Manager::new().await {
        Ok(m) => {
            println!("[DEBUG] Bluetooth manager created successfully");
            m
        }
        Err(e) => {
            println!("[ERROR] Failed to create Bluetooth manager: {}", e);
            return Err(format!("Bluetooth initialization failed: {}", e));
        }
    };

    // Get adapters
    let adapters = match manager.adapters().await {
        Ok(a) => {
            println!("[DEBUG] Found {} Bluetooth adapter(s)", a.len());
            a
        }
        Err(e) => {
            println!("[ERROR] Failed to get adapters: {}", e);
            return Err(format!("Failed to get Bluetooth adapters: {}", e));
        }
    };
>>>>>>> 08cfc70 (WIP)

    // 2. Get Bluetooth Adapters
    let adapters = match manager.adapters().await {
        Ok(a) => {
            println!("[ADAPTERS] Found {} adapter(s)", a.len());
            a
        }
        Err(e) => {
            println!("[ERROR] Failed to get adapters: {}", e);
            return Err(format!("Adapter discovery failed: {}", e));
        }
    };

    // 3. Process each adapter
    for adapter in adapters {
<<<<<<< HEAD
        let adapter_info = match adapter.adapter_info().await {
            Ok(info) => {
                println!("[ADAPTER] Adapter info: {}", info);
                info
            }
            Err(e) => {
                println!("[WARN] Failed to get adapter info: {}", e);
                continue;
            }
        };

        // 4. Detect platform/adapter type
        let (is_windows, is_linux_hci) = {
            let info_lower = adapter_info.to_lowercase();
            (
                info_lower.contains("winrt") || info_lower.contains("windows"),
                info_lower.contains("hci"),
            )
        };

        println!(
            "[PLATFORM] Detected - Windows: {}, Linux HCI: {}",
            is_windows, is_linux_hci
        );

        // 5. Windows-specific pre-pairing
        if is_windows {
            println!("[WINDOWS] Starting Windows-specific pairing process...");
            use std::process::Command;

            let win_device_id = device_id.replace(":", "");

            // Check if device is already paired
            let check_paired = Command::new("powershell")
                .args(&[
                    "-Command",
                    &format!("Get-PnpDevice -InstanceId 'BTHENUM\\DEV_{}' | Where-Object {{ $_.Status -eq 'OK' }}", 
                        win_device_id)
                ])
                .output();

            match check_paired {
                Ok(output) => {
                    let output_str = String::from_utf8_lossy(&output.stdout);
                    if output_str.trim().is_empty() {
                        println!("[WINDOWS] Device not paired, attempting to pair...");

                        // Try modern Windows 10/11 method first
                        println!("[WINDOWS] Trying modern pairing via ms-settings...");
                        let _ = Command::new("powershell")
                            .args(&[
                                "-Command",
                                &format!(
                                    "Start-Process ms-settings:bluetooth; \
                                    Start-Sleep -Seconds 2; \
                                    Add-BluetoothDevice -DeviceId {}",
                                    device_id
                                ),
                            ])
                            .status();

                        // Then try legacy method
                        println!("[WINDOWS] Trying legacy pairing via bthprops.cpl...");
                        let _ = Command::new("powershell")
                            .args(&[
                                "-Command",
                                &format!(
                                    "Start-Process bthprops.cpl; \
                                    Start-Sleep -Seconds 2; \
                                    Add-BluetoothDevice -DeviceId {}",
                                    device_id
                                ),
                            ])
                            .status();

                        tokio::time::sleep(Duration::from_secs(5)).await;
                    }
                }
                Err(e) => println!("[WARN] Failed to check pairing status: {}", e),
            }

            // Refresh device list
            println!("[WINDOWS] Starting fresh scan...");
            if let Err(e) = adapter.start_scan(ScanFilter::default()).await {
                println!("[WARN] Scan failed: {}", e);
            }
            tokio::time::sleep(Duration::from_secs(2)).await;
        }
        if !is_windows && !is_linux_hci {
            adapter
                .start_scan(ScanFilter::default())
                .await
                .expect("Failed to start scan");
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        }
        // 6. Get list of peripherals
        let peripherals = match adapter.peripherals().await {
            Ok(p) => {
                println!("[PERIPHERALS] Found {} device(s)", p.len());

                // Debug print all found devices
                println!("[DEBUG] Listing all peripherals:");
                for (i, p) in p.iter().enumerate() {
                    let props = p.properties().await.unwrap_or_default();
                    println!(
                        "  {}. ID: {}, Connected: {}",
                        i + 1,
                        p.id(),
                        p.is_connected().await.unwrap_or(false)
                    );
                }

                p
            }
            Err(e) => {
                println!("[ERROR] Peripheral discovery failed: {}", e);
                continue;
            }
        };

        // 7. Search for matching peripheral with platform-specific comparison
        for peripheral in peripherals {
            let peripheral_id = peripheral.id().to_string();
            let peripheral_props = peripheral.properties().await.unwrap_or_default();

            println!("[CHECK] Checking peripheral: {}", peripheral_id,);

            let is_match = if is_windows {
                // Windows-specific comparison
                let clean_peripheral_id = peripheral_id
                    .replace("BTHENUM\\", "")
                    .replace("DEV_", "")
                    .replace(":", "")
                    .to_lowercase();
                let clean_target_id = device_id.replace(":", "").to_lowercase();

                println!(
                    "[COMPARE] Windows: {} contains {}? {}",
                    clean_peripheral_id,
                    clean_target_id,
                    clean_peripheral_id.contains(&clean_target_id)
                );

                clean_peripheral_id.contains(&clean_target_id)
            } else if is_linux_hci {
                // Linux HCI adapter comparison
                println!("[COMPARE] Linux HCI: {} == {}", peripheral_id, device_id);
                peripheral_id == device_id
            } else {
                // Default comparison for other platforms
                println!("[COMPARE] Default: {} == {}", peripheral_id, device_id);
                peripheral_id == device_id
            };

            if is_match {
                println!("[MATCH] Found matching device!");

                // 8. Mark as connected and create LSL outlet
                println!("[STATE] Setting BLE_CONNECTED = true");
                *BLE_CONNECTED.lock().unwrap() = true;

                println!("[LSL] Creating outlet...");
                if let Err(e) = create_ble_outlet() {
                    println!("[ERROR] Outlet creation failed: {}", e);
                    return Err(format!("LSL initialization failed: {}", e));
=======
        println!("[DEBUG] Checking adapter: {:?}", adapter.adapter_info().await);

        // Get peripherals
        let peripherals = match adapter.peripherals().await {
            Ok(p) => {
                println!("[DEBUG] Found {} peripheral(s)", p.len());
                p
            }
            Err(e) => {
                println!("[WARN] Failed to get peripherals for adapter: {}", e);
                continue;
            }
        };

        for peripheral in peripherals {
            let peripheral_id = peripheral.id().to_string();
            println!("[DEBUG] Checking peripheral: {}", peripheral_id);

            if peripheral_id == device_id {
                println!("[DEBUG] Found matching peripheral: {}", peripheral_id);

                // Windows-specific pairing workaround
                #[cfg(target_os = "windows")]
                {
                    println!("[DEBUG] Running Windows-specific pairing workaround");
                    use std::process::Command;
                    
                    // First check if device is already paired
                    let check_paired = Command::new("powershell")
                        .args(&[
                            "-Command",
                            &format!("Get-PnpDevice -InstanceId 'BTHENUM\\{}' | Where-Object {{ $_.Status -eq 'OK' }}", device_id)
                        ])
                        .output();

                    match check_paired {
                        Ok(output) => {
                            let output_str = String::from_utf8_lossy(&output.stdout);
                            if output_str.trim().is_empty() {
                                println!("[DEBUG] Device not paired, attempting to pair");
                                let pair_result = Command::new("powershell")
                                    .args(&[
                                        "-Command",
                                        &format!("Start-Process -FilePath 'ms-settings:bluetooth' -Wait; Add-BluetoothDevice -DeviceId {}", device_id)
                                    ])
                                    .status();

                                match pair_result {
                                    Ok(status) => {
                                        if status.success() {
                                            println!("[DEBUG] Pairing command executed successfully");
                                        } else {
                                            println!("[WARN] Pairing command failed with status: {:?}", status.code());
                                        }
                                    }
                                    Err(e) => println!("[WARN] Failed to execute pairing command: {}", e),
                                }
                            } else {
                                println!("[DEBUG] Device is already paired");
                            }
                        }
                        Err(e) => println!("[WARN] Failed to check pairing status: {}", e),
                    }
                }

                println!("[DEBUG] Setting BLE_CONNECTED to true");
                *BLE_CONNECTED.lock().unwrap() = true;

                println!("[DEBUG] Creating LSL outlet");
                if let Err(e) = create_ble_outlet() {
                    println!("[ERROR] Failed to create LSL outlet: {}", e);
                    return Err(format!("Failed to create LSL outlet: {}", e));
                }

                // Attempt connection with timeout
                println!("[DEBUG] Attempting to connect to peripheral...");
                let connect_result = tokio::time::timeout(
                    Duration::from_secs(10),
                    peripheral.connect()
                ).await;

                match connect_result {
                    Ok(Ok(_)) => {
                        println!("[DEBUG] Successfully connected to peripheral");
                    }
                    Ok(Err(e)) => {
                        println!("[ERROR] Connection failed: {}", e);
                        return Err(format!("Connection failed: {}", e));
                    }
                    Err(_) => {
                        println!("[ERROR] Connection timed out after 10 seconds");
                        return Err("Connection timed out".to_string());
                    }
                }

                // Discover services
                println!("[DEBUG] Discovering services...");
                match peripheral.discover_services().await {
                    Ok(_) => println!("[DEBUG] Services discovered successfully"),
                    Err(e) => {
                        println!("[ERROR] Failed to discover services: {}", e);
                        return Err(format!("Service discovery failed: {}", e));
                    }
>>>>>>> 08cfc70 (WIP)
                }

                // 9. Connect with timeout (10 seconds)
                println!("[CONNECT] Attempting connection...");
                let connect_result =
                    tokio::time::timeout(Duration::from_secs(10), peripheral.connect()).await;

                match connect_result {
                    Ok(Ok(_)) => println!("[CONNECT] Connected successfully!"),
                    Ok(Err(e)) => {
                        println!("[ERROR] Connection failed: {}", e);
                        return Err(format!("Connection failed: {}", e));
                    }
                    Err(_) => {
                        println!("[ERROR] Connection timed out");
                        return Err("Connection timed out (10s)".to_string());
                    }
                }

                // 10. Discover services
                println!("[SERVICES] Discovering services...");
                if let Err(e) = peripheral.discover_services().await {
                    println!("[ERROR] Service discovery failed: {}", e);
                    return Err(format!("Service discovery failed: {}", e));
                }

                // 11. Get characteristics
                let characteristics = peripheral.characteristics();
<<<<<<< HEAD
                println!("[CHAR] Found {} characteristics", characteristics.len());

                // 12. Find required characteristics
                let data_char = characteristics
                    .iter()
                    .find(|c| c.uuid.to_string() == "beb5483e-36e1-4688-b7f5-ea07361b26a8")
                    .ok_or_else(|| {
                        println!("[ERROR] Data characteristic not found");
                        "Data characteristic missing".to_string()
                    })?;

                let control_char = characteristics
                    .iter()
                    .find(|c| c.uuid.to_string() == "0000ff01-0000-1000-8000-00805f9b34fb")
                    .ok_or_else(|| {
                        println!("[ERROR] Control characteristic not found");
                        "Control characteristic missing".to_string()
                    })?;

                // 13. Subscribe to notifications
                println!("[SUBSCRIBE] Setting up notifications...");
                if let Err(e) = peripheral.subscribe(data_char).await {
                    println!("[ERROR] Subscription failed: {}", e);
                    return Err(format!("Notification setup failed: {}", e));
                }

                // 14. Send start command
                println!("[CONTROL] Sending start command...");
                if let Err(e) = peripheral
                    .write(control_char, b"start", WriteType::WithResponse)
                    .await
                {
                    println!("[ERROR] Start command failed: {}", e);
                    return Err(format!("Failed to start device: {}", e));
                }

                // 15. Set up notification stream
                let mut notifications = match peripheral.notifications().await {
                    Ok(n) => {
                        println!("[NOTIFICATIONS] Stream established");
                        n
                    }
                    Err(e) => {
                        println!("[ERROR] Notification stream failed: {}", e);
                        return Err(format!("Notification stream error: {}", e));
=======
                println!("[DEBUG] Found {} characteristics", characteristics.len());

                let data_char = characteristics.iter()
                    .find(|c| c.uuid.to_string() == "beb5483e-36e1-4688-b7f5-ea07361b26a8")
                    .ok_or_else(|| {
                        println!("[ERROR] Data characteristic not found");
                        "Data characteristic not found".to_string()
                    })?;

                let control_char = characteristics.iter()
                    .find(|c| c.uuid.to_string() == "0000ff01-0000-1000-8000-00805f9b34fb")
                    .ok_or_else(|| {
                        println!("[ERROR] Control characteristic not found");
                        "Control characteristic not found".to_string()
                    })?;

                println!("[DEBUG] Subscribing to notifications...");
                match peripheral.subscribe(data_char).await {
                    Ok(_) => println!("[DEBUG] Subscribed to notifications successfully"),
                    Err(e) => {
                        println!("[ERROR] Failed to subscribe: {}", e);
                        return Err(format!("Subscription failed: {}", e));
                    }
                }

                println!("[DEBUG] Sending start command...");
                match peripheral.write(control_char, b"start", WriteType::WithResponse).await {
                    Ok(_) => println!("[DEBUG] Start command sent successfully"),
                    Err(e) => {
                        println!("[ERROR] Failed to send start command: {}", e);
                        return Err(format!("Failed to send start command: {}", e));
                    }
                }

                let mut notifications = match peripheral.notifications().await {
                    Ok(n) => {
                        println!("[DEBUG] Notification stream established");
                        n
                    }
                    Err(e) => {
                        println!("[ERROR] Failed to get notification stream: {}", e);
                        return Err(format!("Notification stream failed: {}", e));
>>>>>>> 08cfc70 (WIP)
                    }
                };

                let app_handle_clone = app_handle.clone();

                // 16. Spawn processing task
                tokio::spawn(async move {
<<<<<<< HEAD
                    println!("[TASK] Starting data processing loop");
                    let mut sample_count = 0;
                    let start_time = Instant::now();

=======
                    println!("[DEBUG] Starting notification processing loop");
>>>>>>> 08cfc70 (WIP)
                    while *BLE_CONNECTED.lock().unwrap() {
                        if let Some(data) = notifications.next().await {
                            println!("[DEBUG] Received notification ({} bytes)", data.value.len());
                            
                            match data.value.len() {
                                NEW_PACKET_LEN => {
                                    println!("[DEBUG] Processing full packet ({} samples)", BLOCK_COUNT);
                                    for chunk in data.value.chunks_exact(SINGLE_SAMPLE_LEN) {
<<<<<<< HEAD
                                        if let Ok(sample) =
                                            process_ble_sample(chunk, app_handle_clone.clone())
                                        {
                                            sample_count += 1;
                                            // Push to LSL
                                            if let Some(outlet) = &BLE_OUTLET.lock().unwrap().0 {
                                                if let Err(e) = outlet.push_sample(&sample) {
                                                    println!("[LSL] Push error: {}", e);
=======
                                        match process_ble_sample(chunk) {
                                            Ok(processed) => {
                                                // Send to LSL
                                                if let Some(outlet) = &BLE_OUTLET.lock().unwrap().0 {
                                                    if let Err(e) = outlet.push_sample(&processed) {
                                                        println!("[WARN] LSL push error: {}", e);
                                                    }
                                                }
                                                // Send to frontend
                                                let _ = app_handle_clone.emit("bleData", json!(processed));
                                            }
                                            Err(e) => println!("[WARN] Sample processing error: {}", e),
                                        }
                                    }
                                },
                                SINGLE_SAMPLE_LEN => {
                                    println!("[DEBUG] Processing single sample");
                                    match process_ble_sample(&data.value) {
                                        Ok(processed) => {
                                            // Send to LSL
                                            if let Some(outlet) = &BLE_OUTLET.lock().unwrap().0 {
                                                if let Err(e) = outlet.push_sample(&processed) {
                                                    println!("[WARN] LSL push error: {}", e);
>>>>>>> 08cfc70 (WIP)
                                                }
                                            }
                                        }
                                        Err(e) => println!("[WARN] Sample processing error: {}", e),
                                    }
<<<<<<< HEAD
                                }
                                SINGLE_SAMPLE_LEN => {
                                    if let Ok(sample) =
                                        process_ble_sample(&data.value, app_handle_clone.clone())
                                    {
                                        // Push to LSL
                                        sample_count += 1;

                                        if let Some(outlet) = &BLE_OUTLET.lock().unwrap().0 {
                                            if let Err(e) = outlet.push_sample(&sample) {
                                                println!("[LSL] Push error: {}", e);
                                            }
                                        }
                                        // Send to frontend
                                    }
                                }
                                len => println!("[WARN] Unexpected packet length: {}", len),
                            }
                            let elapsed = start_time.elapsed().as_secs_f64();
                            let rate = sample_count as f64 / elapsed;
                            if sample_count % 100 == 0 {
                                println!(
                                    "[DATA] Received {} total samples ({:.2} Hz)",
                                    sample_count, rate
                                );
                                let _ = app_handle.emit("samplerate", rate);
                                let _ = app_handle.emit("lsl", "uidbluetooth007");
                            }
                        } else {
                            println!("[TASK] Notification stream ended");
                            break;
                        }
                    }

                    println!("[TASK] Cleaning up...");
                    close_ble_outlet();
                });

<<<<<<< HEAD
                return Ok(format!("Connected"));
            }
        }
    }

    println!("[ERROR] Device not found after scanning all adapters");
    Err("Device not found".to_string())
}

fn cleanup_resources() {
    println!("[CLEANUP] Performing final cleanup");
    *BLE_CONNECTED.lock().unwrap() = false;
    close_ble_outlet();
}
#[tauri::command]
fn cleanup_ble() {
    close_ble_outlet();
}
// Add this with your other lazy_static declarations
lazy_static! {
    // ... your existing static refs ...
    static ref CONNECTED_PERIPHERAL: Arc<Mutex<Option<Peripheral>>> = Arc::new(Mutex::new(None));
}

// Modify the cleanup_on_exit function
fn cleanup_on_exit() {
    println!("[CLEANUP] Application exiting - cleaning up BLE resources");

    // Disconnect the peripheral if connected
    if let Some(peripheral) = CONNECTED_PERIPHERAL.lock().unwrap().take() {
        println!("[CLEANUP] Disconnecting peripheral...");
        if let Err(e) = futures::executor::block_on(peripheral.disconnect()) {
            println!("[WARN] Failed to disconnect peripheral: {}", e);
        }
    }

    // Cleanup other resources
    cleanup_resources();
}
// Modify the main function
=======
                return Ok(format!("Connected to BLE device {}", device_id));
=======
                                },
                                len => println!("[WARN] Unexpected packet length: {}", len),
                            }
                        } else {
                            println!("[DEBUG] Notification stream ended");
                            break;
                        }
                    }
                    println!("[DEBUG] Closing BLE outlet (from notification loop)");
                    close_ble_outlet();
                });

                return Ok(format!("Successfully connected to BLE device {}", device_id));
>>>>>>> 08cfc70 (WIP)
            }
        }
    }

    println!("[ERROR] Failed to find matching peripheral for device ID: {}", device_id);
    Err("Failed to connect to BLE device".to_string())
}

#[tauri::command]
async fn disconnect_from_ble(device_id: String) -> Result<String, String> {
    let manager = Manager::new().await.map_err(|e| e.to_string())?;
    let adapters = manager.adapters().await.map_err(|e| e.to_string())?;

    for adapter in adapters {
        let peripherals = adapter.peripherals().await.map_err(|e| e.to_string())?;
        for peripheral in peripherals {
            if peripheral.id().to_string() == device_id {
                // Get characteristics once
                let characteristics = peripheral.characteristics();
                
                // 1. First send stop command
                if let Some(control_char) = characteristics.iter()
                    .find(|c| c.uuid.to_string() == "0000ff01-0000-1000-8000-00805f9b34fb") 
                {
                    match peripheral.write(control_char, b"stop", WriteType::WithResponse).await {
                        Ok(_) => log::info!("Stop command sent successfully"),
                        Err(e) => log::warn!("Failed to send stop command: {}", e),
                    }
                }

                // 2. Unsubscribe from notifications
                if let Some(data_char) = characteristics.iter()
                    .find(|c| c.uuid.to_string() == "beb5483e-36e1-4688-b7f5-ea07361b26a8") 
                {
                    let _ = peripheral.unsubscribe(data_char).await;
                }

                // 3. Disconnect
                let disconnect_result = peripheral.disconnect().await;

                // 4. Platform-specific unpairing
                #[cfg(target_os = "linux")]
                let unpair_result = std::process::Command::new("bluetoothctl")
                    .args(&["remove", &device_id])
                    .status();

                #[cfg(target_os = "macos")]
                let unpair_result = std::process::Command::new("blueutil")
                    .args(&["--unpair", &device_id])
                    .status();

                #[cfg(target_os = "windows")]
                let unpair_result = std::process::Command::new("powershell")
                    .args(&["-Command", &format!("Remove-BluetoothDevice -DeviceId {}", device_id)])
                    .status();

                if let Err(e) = unpair_result {
                    log::warn!("Failed to unpair device: {}", e);
                }

                // Cleanup
                *BLE_CONNECTED.lock().unwrap() = false;
                close_ble_outlet();

                match disconnect_result {
                    Ok(_) => return Ok(format!("Disconnected and unpaired BLE device {}", device_id)),
                    Err(e) => return Err(format!("Disconnect failed: {}", e)),
                }
            }
        }
    }
    
    close_ble_outlet();
    Err("BLE device not found".to_string())
}

#[tauri::command]
fn cleanup_ble() {
    close_ble_outlet();
}

>>>>>>> 98cafaa (Implement BLE)
fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            detect_arduino,
            scan_ble_devices,
            connect_to_ble,
<<<<<<< HEAD
=======
            disconnect_from_ble,
>>>>>>> 98cafaa (Implement BLE)
            start_streaming,
            start_wifistreaming,
            cleanup_ble,
        ])
        .setup(|app| {
            // Get the main window
            let window = app.get_webview_window("main").unwrap();

            // Register cleanup handler when app exits
            window.on_window_event(move |event| {
                if let tauri::WindowEvent::Destroyed = event {
                    cleanup_on_exit();
                }
            });
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("Error while running Tauri application");
}