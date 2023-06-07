#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use core::mem;
use cortex_m::peripheral::NVIC;
use defmt::{panic, *};
use embassy_executor::Spawner;
use embassy_executor::{Executor, InterruptExecutor};
use embassy_stm32::gpio::{AnyPin, Pin};
use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_stm32::interrupt;
use embassy_stm32::pac::Interrupt;
use embassy_stm32::Config;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;

use embassy_time::{Duration, Ticker, Timer};

use {defmt_rtt as _, panic_probe as _};

mod allocator;
mod heartbeat;
mod init;
mod message_parser;
mod usb;

mod items {
    include!(concat!(env!("OUT_DIR"), "/messages_proto.rs"));
}

#[embassy_executor::task]
async fn run_high() {
    loop {
        info!("High");
        Timer::after(Duration::from_millis(1000)).await;
    }
}
use static_cell::StaticCell;

static EXECUTOR_HIGH: InterruptExecutor = InterruptExecutor::new();
static EXECUTOR_MED: InterruptExecutor = InterruptExecutor::new();
static EXECUTOR_LOW: StaticCell<Executor> = StaticCell::new();

#[interrupt]
unsafe fn UART4() {
    EXECUTOR_HIGH.on_interrupt()
}

#[interrupt]
unsafe fn UART5() {
    EXECUTOR_MED.on_interrupt()
}
static CHANNEL_TO_COMPUTER: Channel<CriticalSectionRawMutex, items::MessageFromCnc, 2> =
    Channel::new();
static CHANNEL_FROM_COMPUTER: Channel<CriticalSectionRawMutex, items::MessageFromInterface, 2> =
    Channel::new();

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    allocator::init(); // Initialize the allocator in allocator.rs

    let mut config = Config::default();
    init::clock(&mut config); // Initialize the clock in init.rs
    let peripherals = embassy_stm32::init(config);

    let mut nvic: NVIC = unsafe { mem::transmute(()) };
    // High-priority executor: UART4, priority level 6
    unsafe { nvic.set_priority(Interrupt::TIM2, 6 << 4) };
    let spawner = EXECUTOR_HIGH.start(Interrupt::UART4);
    unwrap!(spawner.spawn(run_high()));

    //spawner // Spawn the heartbeat task
    //    .spawn(heartbeat::run(peripherals.PC8.degrade()))
    //    .unwrap();
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
