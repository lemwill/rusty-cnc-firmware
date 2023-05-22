use defmt::info;
use embassy_stm32::gpio::{AnyPin, Level, Output, Speed};
use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use embassy_sync::channel::Receiver;
use embassy_time::{Duration, Ticker};

use crate::items;

#[embassy_executor::task]
pub async fn run_stepper(
    pin: AnyPin,
    channel_from_computer: Receiver<'static, ThreadModeRawMutex, items::Jog, 2>,
) {
    let mut led = Output::new(pin, Level::High, Speed::Low);
    let mut ticker = Ticker::every(Duration::from_millis(500));
    loop {
        //led.set_high();
        //ticker.next().await;
        //led.set_low();
        //ticker.next().await;
        let jog = channel_from_computer.recv().await;
        info!("Axis: {:#?}", jog.axis);
        info!("Direction: {:#?}", jog.direction);
    }
}
