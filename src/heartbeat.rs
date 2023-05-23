use embassy_stm32::gpio::{AnyPin, Level, Output, Speed};
use embassy_time::{Duration, Ticker};

#[embassy_executor::task]
pub async fn run(pin: AnyPin) {
    let mut led = Output::new(pin, Level::High, Speed::Low);
    let mut ticker = Ticker::every(Duration::from_millis(500));
    loop {
        led.set_high();
        ticker.next().await;
        led.set_low();
        ticker.next().await;
    }
}
