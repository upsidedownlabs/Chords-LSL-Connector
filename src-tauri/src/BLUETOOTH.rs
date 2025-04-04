use serialport;
use std::io::{self, Read, Write};
use std::thread;
use std::time::{Duration, Instant};
use tauri::{self, AppHandle, Emitter}; // Import Emitter along with AppHandle
use serde_json::json; // Add this import at the top
use tokio::sync::mpsc;
use lsl::Pushable;  // Add the necessary imports
use lsl::StreamOutlet;
use lsl::{StreamInfo, ChannelFormat};
use tungstenite::connect;
use url::Url;
use tungstenite::protocol::Message;


use lazy_static::lazy_static;
use std::sync::{Arc, Mutex};
lazy_static! {
    static ref BAUDRATE: Arc<Mutex<u32>> = Arc::new(Mutex::new(230400)); // Default baud rate
    static ref PACKET_SIZE: Arc<Mutex<usize>> = Arc::new(Mutex::new(16)); // Default baud rate
    static ref CHANNELS: Arc<Mutex<usize>> = Arc::new(Mutex::new(6)); // Default baud rate
    static ref SAMPLE_RATE: Arc<Mutex<f64>> = Arc::new(Mutex::new(500.0)); // Default baud rate
}

#[tauri::command]
fn detect_arduino() -> Result<String, String> {
    loop {
        let ports = serialport::available_ports().expect("No ports found!");


        for port_info in ports {
            let port_name = port_info.port_name;
            println!("Attempting to connect to port: {}", port_name);

            if port_name.contains("BLTH") || port_name.contains("Bluetooth")||port_name.contains("console"){
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
                                    if response.contains("NANO-CLONE")|| response.contains("NANO-CLASSIC")|| response.contains("STM32F4-BLACK-PILL")
                                    {
                                        *PACKET_SIZE.lock().unwrap() = 20; // Change the baud rate dynamically
                                        *CHANNELS.lock().unwrap() = 8; // Change the baud rate dynamically
                                    }
                                    if response.contains("MEGA-2560-R3")|| response.contains("MEGA-2560-CLONE")|| response.contains("STM32G4-CORE-BOARD")
                                    {
                                        *PACKET_SIZE.lock().unwrap() = 36; // Change the baud rate dynamically
                                        *CHANNELS.lock().unwrap() = 16; // Change the baud rate dynamically
                                    }
                                    if response.contains("RPI-PICO-RP2040")|| response.contains("NPG-LITE")
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
    let info = Arc::new(lsl::StreamInfo::new(
        "UDL",
        "Biopotential_Signals",
        (*CHANNELS.lock().unwrap()).try_into().unwrap(),
        *SAMPLE_RATE.lock().unwrap(),
        lsl::ChannelFormat::Int16,
        "Chords",
    )
    .unwrap());

    // Create StreamOutlet in the same thread
    let (tx, rx) = std::sync::mpsc::channel::<Vec<i16>>();
    let outlet = Arc::new(Mutex::new(StreamOutlet::new(&info, 0, 360).unwrap()));



    // Use spawn_blocking to handle the task in a separate thread
    tokio::task::spawn_blocking(move || {
        loop {
            match serialport::new(&port_name, *BAUDRATE.lock().unwrap())
                .timeout(Duration::from_secs(3))
                .open()
            {
                Ok(mut port) => {
                    let start_command = b"START\r\n";

                    for i in 1..=3 {
                        if let Err(e) = port.write_all(start_command) {
                        } 
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
                                        if accumulated_buffer[*PACKET_SIZE.lock().unwrap() - 1] == END_BYTE {
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
                                    let samples_per_second = format!("{:.2}", sample_count as f32 / elapsed);
                                    let bytes_per_second = format!("{:.2}", byte_count as f32 / elapsed);
                                    

                                    app_handle
                                        .emit(
                                            "updatePerformance",
                                            json!({
                                                "refreshRate": refresh_rate,
                                                "samplesPerSecond": samples_per_second,
                                                "bytesPerSecond": bytes_per_second
                                            }),
                                        )
                                        .unwrap_or_else(|e| {
                                            println!("Failed to emit performance metrics: {:?}", e);
                                        });

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
        }
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
async fn start_wifistreaming() {
    let stream_name = "NPG-Lite";
    let info = StreamInfo::new(stream_name, "EXG", 3, 500.0, ChannelFormat::Float32, "uid007").unwrap();
    let outlet = StreamOutlet::new(&info, 0, 360).unwrap();
    
    let ws_url = "ws://multi-emg.local:81";
    let (mut socket, _) = connect(Url::parse(ws_url).unwrap()).expect("WebSocket connection failed");
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
                            break;
                        } else if sample_number == prev {
                            println!("Error: Duplicate Sample");
                            break;
                        }
                    }
                    previous_sample_number = Some(sample_number);
                    previous_data = channel_data.clone();
    
                    println!("EEG Data: {} {:?}", sample_number, channel_data);
                    outlet.push_sample(&channel_data).unwrap();
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


use btleplug::api::WriteType;
use futures::stream::StreamExt;  // Make sure this is imported
use std::sync::atomic::{AtomicU8, Ordering};

// Thread-safe wrapper for StreamOutlet
struct SafeOutlet(Option<StreamOutlet>);
unsafe impl Send for SafeOutlet {}
unsafe impl Sync for SafeOutlet {}

// Constants for Bluetooth communication
const SINGLE_SAMPLE_LEN: usize = 7;
const BLOCK_COUNT: usize = 10;
const NEW_PACKET_LEN: usize = SINGLE_SAMPLE_LEN * BLOCK_COUNT;

// Global state for Bluetooth
lazy_static! {
    static ref BLE_OUTLET: Arc<Mutex<SafeOutlet>> = Arc::new(Mutex::new(SafeOutlet(None)));
    static ref BLE_SAMPLE_COUNTER: AtomicU8 = AtomicU8::new(0);
    static ref BLE_CONNECTED: Arc<Mutex<bool>> = Arc::new(Mutex::new(false));
}

// Create Bluetooth LSL outlet
fn create_ble_outlet() -> Result<(), String> {
    let info = StreamInfo::new("NPG-Lite", "EXG", 3, 500.0, ChannelFormat::Float32, "uid007")
        .map_err(|e| e.to_string())?;
    let outlet = StreamOutlet::new(&info, 0, 360).map_err(|e| e.to_string())?;
    *BLE_OUTLET.lock().unwrap() = SafeOutlet(Some(outlet));
    Ok(())
}

// Close Bluetooth LSL outlet
fn close_ble_outlet() {
    *BLE_OUTLET.lock().unwrap() = SafeOutlet(None);
    BLE_SAMPLE_COUNTER.store(0, Ordering::Relaxed);
    *BLE_CONNECTED.lock().unwrap() = false;
}

// Process Bluetooth samples
fn process_ble_sample(sample: &[u8]) -> Result<Vec<f32>, String> {
    if sample.len() != SINGLE_SAMPLE_LEN {
        return Err("Invalid sample length".to_string());
    }

    let sample_counter = sample[0];
    let prev = BLE_SAMPLE_COUNTER.load(Ordering::Relaxed);
    let expected = prev.wrapping_add(1);
    if sample_counter != expected {
        println!("Sample counter discontinuity: expected {}, got {}", expected, sample_counter);
    }
    BLE_SAMPLE_COUNTER.store(sample_counter, Ordering::Relaxed);

    Ok(vec![
        i16::from_be_bytes([sample[1], sample[2]]) as f32,
        i16::from_be_bytes([sample[3], sample[4]]) as f32,
        i16::from_be_bytes([sample[5], sample[6]]) as f32,
    ])
}

#[tauri::command]
async fn scan_bluetooth_devices(app_handle: AppHandle) -> Result<(), String> {
    let manager = Manager::new().await.map_err(|e| e.to_string())?;
    let adapters = manager.adapters().await.map_err(|e| e.to_string())?;

    for adapter in adapters {
        adapter.start_scan(ScanFilter::default()).await.map_err(|e| e.to_string())?;
        tokio::time::sleep(Duration::from_secs(5)).await;

        let peripherals = adapter.peripherals().await.map_err(|e| e.to_string())?;
        let mut devices = Vec::new();

        for peripheral in peripherals {
            match peripheral.properties().await {
                Ok(Some(properties)) => {
                    if let Some(local_name) = properties.local_name {
                        devices.push(json!({
                            "name": local_name,
                            "id": peripheral.id().to_string()
                        }));
                    }
                }
                Ok(None) => {
                    println!("No properties available for peripheral");
                }
                Err(e) => {
                    println!("Error getting properties: {}", e);
                }
            }
        }

        app_handle.emit("bluetoothDevices", devices).map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
async fn connect_to_bluetooth(device_id: String, app_handle: AppHandle) -> Result<String, String> {
    close_ble_outlet();

    let manager = Manager::new().await.map_err(|e| e.to_string())?;
    let adapters = manager.adapters().await.map_err(|e| e.to_string())?;

    for adapter in adapters {
        let peripherals = adapter.peripherals().await.map_err(|e| e.to_string())?;
        for peripheral in peripherals {
            if peripheral.id().to_string() == device_id {
                *BLE_CONNECTED.lock().unwrap() = true;
                create_ble_outlet().map_err(|e| e.to_string())?;

                peripheral.connect().await.map_err(|e| e.to_string())?;
                peripheral.discover_services().await.map_err(|e| e.to_string())?;

                let characteristics = peripheral.characteristics();
                
                let data_char = characteristics.iter()
                    .find(|c| c.uuid.to_string() == "beb5483e-36e1-4688-b7f5-ea07361b26a8")
                    .ok_or("Data characteristic not found")?;
                
                let control_char = characteristics.iter()
                    .find(|c| c.uuid.to_string() == "0000ff01-0000-1000-8000-00805f9b34fb")
                    .ok_or("Control characteristic not found")?;

                peripheral.subscribe(data_char).await.map_err(|e| e.to_string())?;
                peripheral.write(control_char, b"start", WriteType::WithResponse)
                    .await
                    .map_err(|e| e.to_string())?;

                let mut notifications = peripheral.notifications().await.map_err(|e| e.to_string())?;
                let app_handle_clone = app_handle.clone();

                tokio::spawn(async move {
                    while *BLE_CONNECTED.lock().unwrap() {
                        if let Some(data) = notifications.next().await {
                            match data.value.len() {
                                NEW_PACKET_LEN => {
                                    for chunk in data.value.chunks_exact(SINGLE_SAMPLE_LEN) {
                                        if let Ok(processed) = process_ble_sample(chunk) {
                                            // Send to LSL
                                            if let Some(outlet) = &BLE_OUTLET.lock().unwrap().0 {
                                                if let Err(e) = outlet.push_sample(&processed) {
                                                    println!("LSL push error: {}", e);
                                                }
                                            }
                                            // Send to frontend
                                            let _ = app_handle_clone.emit("bleData", json!(processed));
                                        }
                                    }
                                },
                                SINGLE_SAMPLE_LEN => {
                                    if let Ok(processed) = process_ble_sample(&data.value) {
                                        // Send to LSL
                                        if let Some(outlet) = &BLE_OUTLET.lock().unwrap().0 {
                                            if let Err(e) = outlet.push_sample(&processed) {
                                                println!("LSL push error: {}", e);
                                            }
                                        }
                                        // Send to frontend
                                        let _ = app_handle_clone.emit("bleData", json!(processed));
                                    }
                                },
                                len => println!("Unexpected packet length: {}", len),
                            }
                        } else {
                            break;
                        }
                    }
                    close_ble_outlet();
                });

                return Ok(format!("Connected to device {}", device_id));
            }
        }
    }
    Err("Failed to connect to device".to_string())
}

#[tauri::command]
async fn disconnect_from_bluetooth(device_id: String) -> Result<String, String> {
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
                    Ok(_) => return Ok(format!("Disconnected and unpaired device {}", device_id)),
                    Err(e) => return Err(format!("Disconnect failed: {}", e)),
                }
            }
        }
    }
    
    close_ble_outlet();
    Err("Device not found".to_string())
}
#[tauri::command]
fn cleanup_bluetooth() {
    close_ble_outlet();
}fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            detect_arduino,
            scan_bluetooth_devices,
            connect_to_bluetooth,
            disconnect_from_bluetooth,  // Add this here
            start_streaming,
            start_wifistreaming,
            cleanup_bluetooth,
        ])
        .run(tauri::generate_context!())
        .expect("Error while running Tauri application");
}
