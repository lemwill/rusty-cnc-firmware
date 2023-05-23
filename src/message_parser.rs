use defmt::info;
use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use embassy_sync::channel::{Receiver, Sender};

use crate::items;

#[embassy_executor::task]
pub async fn run(
    channel_from_computer: Receiver<'static, ThreadModeRawMutex, items::Jog, 2>,
    channel_to_computer: Sender<'static, ThreadModeRawMutex, items::Status, 2>,
) {
    loop {
        let jog = channel_from_computer.recv().await;
        info!("Axis: {:#?}", jog.axis);
        info!("Direction: {:#?}", jog.direction);
        let status = items::Status {
            position: Some(items::Position { x: 1, y: 2, z: 3 }),
        };

        channel_to_computer.send(status).await;
    }
}
