#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use embassy_executor::Spawner;
use embassy_stm32::gpio::Pin;
use embassy_stm32::Config;
use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use embassy_sync::channel::Channel;
use {defmt_rtt as _, panic_probe as _};
// Import modules
mod allocator;
mod heartbeat;
mod init;
mod message_parser;
mod usb;

mod items {
    include!(concat!(env!("OUT_DIR"), "/messages_proto.rs"));
}

static CHANNEL_TO_COMPUTER: Channel<ThreadModeRawMutex, items::MessageFromCnc, 2> = Channel::new();
static CHANNEL_FROM_COMPUTER: Channel<ThreadModeRawMutex, items::MessageFromInterface, 2> =
    Channel::new();

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    allocator::init(); // Initialize the allocator in allocator.rs

    let mut config = Config::default();
    init::clock(&mut config); // Initialize the clock in init.rs
    let peripherals = embassy_stm32::init(config);

    spawner
        .spawn(usb::init(
            peripherals.USB_OTG_FS,
            peripherals.PA12,
            peripherals.PA11,
            CHANNEL_TO_COMPUTER.receiver(),
            CHANNEL_FROM_COMPUTER.sender(),
        ))
        .unwrap();

    spawner
        .spawn(message_parser::run(
            CHANNEL_FROM_COMPUTER.receiver(),
            CHANNEL_TO_COMPUTER.sender(),
        ))
        .unwrap();

    // Spawn the hearbeat task
    spawner // Spawn the heartbeat task
        .spawn(heartbeat::run(peripherals.PB0.degrade()))
        .unwrap();
}
