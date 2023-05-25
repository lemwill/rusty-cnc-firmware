use defmt::info;
use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use embassy_sync::channel::{Receiver, Sender};

use crate::items;

#[embassy_executor::task]
pub async fn run(
    channel_from_computer: Receiver<'static, ThreadModeRawMutex, items::Jog, 2>,
    channel_to_computer: Sender<'static, ThreadModeRawMutex, items::Status, 2>,
) {
    let mut status = items::Status {
        position: Some(items::Position { x: 0, y: 0, z: 0 }),
    };
    loop {
        let jog = channel_from_computer.recv().await;
        info!("Axis: {:#?}", jog.axis);
        info!("Direction: {:#?}", jog.direction);

        match jog.axis {
            x if x == items::jog::Axis::X as i32 => {
                if let Some(position) = &mut status.position {
                    position.x += jog.direction;
                }
            }
            y if y == items::jog::Axis::Y as i32 => {
                if let Some(position) = &mut status.position {
                    position.y += jog.direction;
                }
            }
            z if z == items::jog::Axis::Z as i32 => {
                if let Some(position) = &mut status.position {
                    position.z += jog.direction;
                }
            }
            _ => {}
        }

        channel_to_computer.send(status.clone()).await;
    }
}
