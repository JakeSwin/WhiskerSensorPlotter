mod measurements;

use crate::measurements::{MeasurementWindow, SensorSampleMeasurement};
use eframe::{egui, Frame};

use std::io::BufRead;
use std::sync::*;
use std::thread;
use egui::{Color32, Context};
use tracing::{error, info, warn};

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
            let mut plot = egui_plot::Plot::new("measurements");
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
    });

    info!("Main thread started");
    eframe::run_native("Monitor app", native_options, Box::new(|_| Box::new(app))).expect("Could not create App");
}
