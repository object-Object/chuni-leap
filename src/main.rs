mod air_sensor;

use std::{error::Error, sync::Arc, thread};

use air_sensor::{AirSensor, AirSensorState};
use eframe::egui;
use egui::{Color32, Sense, Vec2};
use log::error;
use parking_lot::Mutex;
use serde::{de::DeserializeOwned, Serialize};

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    eframe::run_native(
        "chuni-leap",
        eframe::NativeOptions::default(),
        Box::new(|cc| Ok(Box::new(ChuniLeapApp::new(cc)))),
    )?;

    Ok(())
}

#[derive(Default)]
struct ChuniLeapApp {
    air_sensor_state: Arc<Mutex<AirSensorState>>,
}

impl ChuniLeapApp {
    fn new(cc: &eframe::CreationContext) -> Self {
        let app = match cc.storage {
            Some(storage) => Self {
                air_sensor_state: Arc::new(Mutex::new(AirSensorState {
                    config: deserialize_storage(storage, storage_keys::AIR_SENSOR_CONFIG),
                    ..Default::default()
                })),
            },
            None => Self::default(),
        };

        let air_sensor_state = app.air_sensor_state.clone();
        let ctx = cc.egui_ctx.clone();
        thread::spawn(move || {
            // FIXME: sane error handling idk it's midnight right now
            AirSensor::new(air_sensor_state, ctx)
                .unwrap()
                .run()
                .unwrap();
        });

        app
    }
}

impl eframe::App for ChuniLeapApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let mut state = self.air_sensor_state.lock();

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.checkbox(&mut state.config.visualize, "Visualize Slider");
            ui.add_enabled_ui(state.config.visualize, |ui| {
                ui.vertical(|ui| {
                    for &enabled in state.visualized_sensors.iter().rev() {
                        let (rect, _) = ui.allocate_at_least(
                            Vec2::new(ui.available_width(), 10.),
                            Sense::hover(),
                        );
                        if enabled {
                            ui.painter().rect_filled(rect, 0., Color32::WHITE);
                        } else {
                            ui.painter().rect_stroke(rect, 0., (1., Color32::WHITE));
                        }
                    }
                });
            });
        });
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        serialize_storage(
            storage,
            storage_keys::AIR_SENSOR_CONFIG,
            &self.air_sensor_state.lock().config,
        );
    }
}

fn deserialize_storage<T>(storage: &dyn eframe::Storage, key: &str) -> T
where
    T: DeserializeOwned + Default,
{
    storage
        .get_string(key)
        .and_then(|v| {
            serde_json::from_str(&v)
                .map_err(|e| error!("Failed to deserialize {key}: {e}"))
                .ok()
        })
        .unwrap_or_default()
}

fn serialize_storage<T>(storage: &mut dyn eframe::Storage, key: &str, value: &T)
where
    T: Sized + Serialize,
{
    match serde_json::to_string(value) {
        Ok(value) => storage.set_string(key, value),
        Err(e) => error!("Failed to serialize {key}: {e}"),
    };
}

mod storage_keys {
    pub const AIR_SENSOR_CONFIG: &str = "air_sensor_config";
}
