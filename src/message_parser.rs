use defmt::info;
use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use embassy_sync::channel::{Receiver, Sender};

use crate::items;
use crate::items::message_from_cnc::Message;
use crate::items::MessageFromCnc;
use crate::items::Position;
use crate::items::Status;

#[embassy_executor::task]
pub async fn run(
    channel_from_computer: Receiver<'static, ThreadModeRawMutex, items::MessageFromInterface, 2>,
    channel_to_computer: Sender<'static, ThreadModeRawMutex, items::MessageFromCnc, 2>,
) {
    let mut position = Position { x: 0, y: 0, z: 0 };

    loop {
        let message = channel_from_computer.recv().await;

        match message.message {
            Some(items::message_from_interface::Message::Jog(jog)) => {
                //handle_jog(jog, &mut status, &channel_to_computer).await;
                match jog.axis {
                    x if x == items::jog::Axis::X as i32 => {
                        position.x += jog.direction;
                    }
                    y if y == items::jog::Axis::Y as i32 => {
                        position.y += jog.direction;
                    }
                    z if z == items::jog::Axis::Z as i32 => {
                        position.z += jog.direction;
                    }
                    _ => {}
                }
            }
            _ => {}
        }

        /* info!(
            "Updated Position: x = {}, y = {}, z = {}",
            position.x, position.y, position.z
        );*/

        let status_message = Status {
            position: Some(position.clone()),
        };

        let cnc_message = MessageFromCnc {
            message: Some(Message::Status(status_message)),
        };

        channel_to_computer.send(cnc_message).await;
    }
}
