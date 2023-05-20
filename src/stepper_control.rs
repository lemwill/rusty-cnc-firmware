use embassy_stm32::gpio::{AnyPin, Level, Output, Speed};
use embassy_stm32::time::Hertz;
use embassy_stm32::usb_otg::{Driver, Instance};
use embassy_stm32::{interrupt, Peripherals};
use embassy_time::{Duration, Ticker, Timer};

#[embassy_executor::task]
pub async fn run_stepper(pin: AnyPin) {
    let mut led = Output::new(pin, Level::High, Speed::Low);
    let mut ticker = Ticker::every(Duration::from_millis(500));
    loop {
        led.set_high();
        ticker.next().await;
        led.set_low();
        ticker.next().await;
    }
}
