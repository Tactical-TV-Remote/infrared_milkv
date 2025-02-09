// infrared.rs

use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

use wiringx::{FlowControl, Parity, Platform, Polarity, SerialConfig, WiringX};

pub struct InfraredSenderConfig {
    pub baudrate: u32,
}

pub struct InfraredSender {
    uart: wiringx::Uart,
    pwm: wiringx::PwmPin,
    config: InfraredSenderConfig,
}

impl InfraredSender {
    pub fn new(device_path: &str, baudrate: u32, carrier_pin: i32) -> Result<Self, Box<dyn std::error::Error>> {
        let wiringx = WiringX::new(Platform::MilkVDuo)?;

        let dev: PathBuf = PathBuf::from(device_path);
        let serial_config = SerialConfig { baud_rate: baudrate, data_bits: 8, parity: Parity::Even, stop_bits: 1, flow_control: FlowControl::None };
        let uart: wiringx::Uart = wiringx.setup_uart(dev, serial_config)?;

        // Generate 56kHz carrier frequency
        let pwm = wiringx.pwm_pin(
            carrier_pin,                         // pin number
            Duration::from_nanos(18_000),       // period
            0.5,                                // duty cycle
            Polarity::Inversed,
        )?;

        let config: InfraredSenderConfig = InfraredSenderConfig {
            baudrate: 4800
        };

        Ok(InfraredSender {
            uart,
            pwm,
            config
        })
    }

    pub fn run(&mut self, queue_tx: Arc<Mutex<Vec<String>>>, queue_rx: Arc<Mutex<Vec<String>>>) {
        println!("Starting infrared sender thread...");
        let mut read_buffer = [0; 128];
        let mut read_buffer_index = 0;
        loop {
            let mut data_to_send = None;
            //println!("Checking queue...");
            // Check if there's any data in the queue
            {
                let mut queue_tx_locked = queue_tx.lock().unwrap();
                if !queue_tx_locked.is_empty() {
                    data_to_send = Some(queue_tx_locked.remove(0));
                }
            }

            if self.uart.data_available() > 0 {
                //Read until buffer is empty
                while self.uart.data_available() > 0 {
                    let byte_read = self.uart.read_char();
                    //If buffer is full, reset it and read again
                    if read_buffer_index >= read_buffer.len() {
                        read_buffer_index = 0;
                        //Clear buffer
                        read_buffer = [0; 128];
                    }
                    read_buffer[read_buffer_index] = byte_read as u8;
                    read_buffer_index += 1;

                    //If a newline is received, process the buffer and reset it
                    if byte_read == '\n' {
                        let mut queue_rx_locked = queue_rx.lock().unwrap();
                        let received_data = String::from_utf8_lossy(&read_buffer[0..read_buffer_index]).to_string();
                        queue_rx_locked.push(received_data);
                        read_buffer_index = 0;
                        read_buffer = [0; 128];
                    }
                }
            }
            //println!("Data to send: {:?}", data_to_send);
            if let Some(string) = data_to_send {
                println!("Sending infrared data: {}", string);
                self.send_infrared(&string);
            } else {
                thread::sleep(Duration::from_millis(100)); // Sleep for a bit before checking the queue again
            }

        }
    }

    fn send_infrared(&mut self, string: &str) {
        self.pwm.set_duty_cycle(0.5);
        
        self.uart.put_string(string);
        self.uart.flush();
        
        // Calculate time required to send the string using baudrate
        let time_s = (string.len() as f64 * 10.0 + 2.0) / self.config.baudrate as f64; // 10 bits per char, 8 data bits, 2 stop bit
        let time_ms = (time_s * 1000.0).round() + 3.0; // Convert to milliseconds
        println!("Sending string took {} ms", time_ms);
        
        thread::sleep(Duration::from_millis(time_ms as u64));
        
        self.pwm.set_duty_cycle(0.0);
        // Infrared done
    }
}