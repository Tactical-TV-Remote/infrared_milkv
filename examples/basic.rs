use std::{
    sync::{Arc, Mutex},
    thread, time::Duration,
};

use ttr_infrared_milkv::InfraredSender;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Define your device path and other parameters
    let device_path = "/dev/ttyS4"; // Example UART device path
    let baudrate = 4800;
    let carrier_pin = 13; // Example pin number

    // Create instances of the queues
    let queue_tx: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let queue_rx: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));

    // Clone the Arc references for the thread
    let queue_tx_clone = Arc::clone(&queue_tx);
    let queue_rx_clone = Arc::clone(&queue_rx);

    // Create and start the InfraredSender
    let mut infrared_sender = InfraredSender::new(device_path, baudrate, carrier_pin)?;

    // Spawn a new thread to run the `run` method of InfraredSender
    let driver_handle = thread::spawn(move || {
        infrared_sender.run(queue_tx_clone, queue_rx_clone);
    });

    // You can add code here to interact with `queue_tx` and `queue_rx`
    // For example, push data to `queue_tx` or read from `queue_rx`

    //Create a thread to send data every second
    let sender_handle = thread::spawn(move || {
        loop {
            std::thread::sleep(Duration::from_secs(1));
            let mut tx_queue_locked = queue_tx.lock().unwrap();
            tx_queue_locked.push("Sending data".to_string());
        }
    });

    //Create a thread to receive the data and print it
    let receiver_handle = thread::spawn(move || {
        loop {
            let mut rx_queue_locked = queue_rx.lock().unwrap();
            if let Some(data) = rx_queue_locked.pop() {
                println!("Received data: {}", data);
            }
        }
    });


    // Wait until any key is pressed
    let mut buffer = String::new();

    std::io::stdin()
        .read_line(&mut buffer)
        .expect("Failed to read line");

    // Wait for the thread to complete (if needed)
    driver_handle.join().expect("Couldn't join on the associated thread");
    sender_handle.join().expect("Couldn't join on the associated thread");
    receiver_handle.join().expect("Couldn't join on the associated thread");

    Ok(())
}