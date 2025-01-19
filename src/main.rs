use std::{error::Error, iter::once};

use glam::Vec3;
use itertools::{chain, Itertools};
use leaprs::{Connection, ConnectionConfig, DigitRef, EventRef, HandRef, HandType, LeapVectorRef};

/// https://users.rust-lang.org/t/max-and-min-of-vec-vec3/109714/5
#[derive(Debug, Clone, Copy, PartialEq)]
struct Aabb {
    pub min: Vec3,
    pub max: Vec3,
}

impl Aabb {
    pub fn from_point(point: Vec3) -> Self {
        Self {
            min: point,
            max: point,
        }
    }

    /// Returns [`None`] if there are no points.
    pub fn from_points(points: impl IntoIterator<Item = Vec3>) -> Option<Self> {
        points.into_iter().map(Self::from_point).reduce(Self::union)
    }

    pub fn from_hand(hand: HandRef) -> Self {
        Self::from_points(
            hand.digits()
                .iter()
                .flat_map(|d| d.bones())
                .map(|b| b.next_joint().into())
                .chain(once(hand.palm().position().into())),
        )
        .unwrap()
    }

    pub fn union(self, other: Self) -> Self {
        Self {
            min: self.min.min(other.min),
            max: self.max.max(other.max),
        }
    }
}

// https://docs.ultraleap.com/api-reference/tracking-api/leapc-guide/leap-concepts.html

struct SensorConfig {
    pub bottom: f32,
    pub step: f32,
}

impl SensorConfig {
    pub fn sensor_index(&self, pos: Vec3) -> Option<usize> {
        let index = ((pos.y - self.bottom) / self.step).floor() as i32;
        if (0..6).contains(&index) {
            Some(index as usize)
        } else {
            None
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut c = Connection::create(ConnectionConfig::default())?;
    c.open()?;

    // units: mm
    let sensor_config = SensorConfig {
        bottom: 100.0,
        step: 25.0,
    };

    loop {
        if let Ok(msg) = c.poll(1000) {
            if let EventRef::Tracking(e) = msg.event() {
                let mut sensors = [false; 6];
                for hand in e.hands().iter() {
                    for pos in hand
                        .digits()
                        .iter()
                        .flat_map(|d| d.bones())
                        .map(|b| b.next_joint().into_glam())
                        .chain(once(hand.palm().position().into_glam()))
                    {
                        if let Some(index) = sensor_config.sensor_index(pos) {
                            sensors[index] = true;
                        }
                    }
                }
                for sensor in sensors {
                    print!("{}", sensor as u32);
                }
                println!();
            }
        }
    }
}
