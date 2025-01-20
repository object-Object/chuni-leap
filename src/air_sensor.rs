use std::{error::Error, iter::once, sync::Arc};

use glam::Vec3;
use leaprs::{Connection, ConnectionConfig, EventRef};
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};

pub const NUM_SENSORS: usize = 6;

pub type AirSensorData = [bool; NUM_SENSORS];

pub struct AirSensor {
    state: Arc<Mutex<AirSensorState>>,
    ctx: egui::Context,
    conn: Connection,
    sensors: AirSensorData,
}

impl AirSensor {
    pub fn new(
        state: Arc<Mutex<AirSensorState>>,
        ctx: egui::Context,
    ) -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            state,
            ctx,
            conn: Connection::create(ConnectionConfig::default())?,
            sensors: [false; NUM_SENSORS],
        })
    }

    pub fn run(mut self) -> Result<Self, Box<dyn Error>> {
        self.conn.open()?;

        loop {
            if let Ok(msg) = self.conn.poll(1000) {
                match msg.event() {
                    EventRef::Tracking(e) => {
                        let mut state = self.state.lock();
                        let mut new_sensors = [false; NUM_SENSORS];

                        for hand in e.hands().iter() {
                            for pos in once(hand.palm().position().into_glam()).chain(
                                hand.digits()
                                    .iter()
                                    .flat_map(|d| d.bones())
                                    .map(|b| b.next_joint().into_glam()),
                            ) {
                                if let Some(index) = state.config.sensor_index(pos) {
                                    new_sensors[index] = true;
                                }
                                // FIXME: scuffed
                                if !state.config.check_fingers {
                                    break;
                                }
                            }
                        }

                        if self.sensors != new_sensors {
                            self.sensors = new_sensors;
                            // TODO: shared memory

                            if state.config.visualize {
                                state.visualized_sensors = new_sensors;
                                self.ctx.request_repaint();
                            }
                        }
                    }
                    // TODO
                    EventRef::Device(_) => (),
                    EventRef::DeviceLost => (),
                    _ => (),
                };
            }
        }
    }
}

#[derive(Default)]
pub struct AirSensorState {
    pub config: AirSensorConfig,
    pub visualized_sensors: AirSensorData,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct AirSensorConfig {
    pub base_height: f32,
    pub step_height: f32,
    pub check_fingers: bool,
    pub visualize: bool,
}

impl AirSensorConfig {
    fn sensor_index(&self, pos: Vec3) -> Option<usize> {
        // https://docs.ultraleap.com/api-reference/tracking-api/leapc-guide/leap-concepts.html
        let index = ((pos.y - self.base_height) / self.step_height).floor() as i32;
        if (0..(NUM_SENSORS as i32)).contains(&index) {
            Some(index as usize)
        } else {
            None
        }
    }
}

impl Default for AirSensorConfig {
    fn default() -> Self {
        Self {
            base_height: 100.,
            step_height: 25.,
            check_fingers: true,
            visualize: false,
        }
    }
}
