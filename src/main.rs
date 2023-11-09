mod measurements;

use crate::measurements::{MeasurementWindow, SensorSample, SensorSampleMeasurement};
use eframe::{egui, Frame};

use std::io::{BufRead, BufReader};
use std::sync::*;
use std::thread;
use egui::{Color32, Context};
use tracing::{error, info, warn};

use serialport;
use serde::Deserialize;
use std::time::Duration;

pub struct MonitorApp {
    include_y: Vec<f64>,
    measurements: Arc<Mutex<MeasurementWindow>>,
}

impl MonitorApp {
    fn new(look_behind: usize) -> Self {
        Self {
            measurements: Arc::new(Mutex::new(MeasurementWindow::new_with_look_behind(
                look_behind
            ))),
            include_y: Vec::new(),
        }
    }
}

impl eframe::App for MonitorApp {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let mut plot = egui_plot::Plot::new("measurements").legend(Legend::default());
            for y in self.include_y.iter() {
                plot = plot.include_y(*y);
            }

            plot.show(ui, |plot_ui| {
                if let [t_plot, x_plot, y_plot, z_plot] = self.measurements.lock().unwrap().plot_values() {
                    plot_ui.line(egui_plot::Line::new(t_plot).name("Temperature").color(Color32::DARK_RED));
                    plot_ui.line(egui_plot::Line::new(x_plot).name("X").color(Color32::RED));
                    plot_ui.line(egui_plot::Line::new(y_plot).name("Y").color(Color32::GREEN));
                    plot_ui.line(egui_plot::Line::new(z_plot).name("Z").color(Color32::BLUE));
                };
            });
        });

        ctx.request_repaint();
    }
}

use clap::Parser;
use egui_plot::Legend;

#[derive(Parser, Debug)]
#[clap(author, version, about)]
struct Args {
    /// Look-behind window size
    #[clap(short, long, default_value_t = 1000000)]
    window_size: usize,

    #[clap(short, long)]
    include_y: Vec<f64>,
}

fn main() {
    let args = Args::parse();

    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(tracing::Level::TRACE)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    let mut app = MonitorApp::new(args.window_size);

    app.include_y = args.include_y;

    let native_options = eframe::NativeOptions::default();

    let monitor_ref = app.measurements.clone();

    thread::spawn(move || {
        //Read Sensor Data Here
        let mut serial_port = serialport::new("/dev/ttyUSB0", 115200)
            .timeout(Duration::from_millis(1500))
            // .parity(Parity::Even)
            .open()
            .expect("Failed to open serial port");
        let mut reader = BufReader::new(serial_port);
        loop {
            let mut my_str = String::new();
            match reader.read_line(&mut my_str) {
                Ok(_) => (),
                _ => continue,
            };
            let slice = my_str.trim();
            // println!("{} - Size: {}", &slice, &slice.len());
            if slice.len() == 1050 {
                let mut hex_decode_vec = hex::decode(slice).expect("Could Not Decode Hex");
                let hex_decode_slice = hex_decode_vec.as_mut_slice();
                if let Ok(deserialised) = postcard::take_from_bytes_cobs::<[SensorSample; 30]>(hex_decode_slice) {
                    for sensor_sample in deserialised.0.iter() {
                        monitor_ref
                            .lock()
                            .unwrap()
                            .add(*sensor_sample)
                    }
                    // println!("{:?}", &deserialised.0);
                    // println!("{:?}", &deserialised.1);
                };
            }
        }
    });

    info!("Main thread started");
    eframe::run_native("Monitor app", native_options, Box::new(|_| Box::new(app))).expect("Could not create App");
}
