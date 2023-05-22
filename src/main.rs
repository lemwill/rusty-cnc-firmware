#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use embassy_executor::Spawner;
use embassy_stm32::Config;
use embassy_stm32::{gpio::Pin, time::mhz};
use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use embassy_sync::channel::Channel;
use {defmt_rtt as _, panic_probe as _};

// Import usb.rs file
mod usb;
use usb::init_usb;

mod stepper_control;
use stepper_control::run_stepper;

pub mod items {
    include!(concat!(env!("OUT_DIR"), "/protobuf_messages.rs"));
}

// Configure clock here
fn init_clock(mut config: &mut Config) {
    config.rcc.hse = Some(mhz(8));
    config.rcc.pll48 = true;
    config.rcc.sys_ck = Some(mhz(200));
}

// PROST protocol buffer library uses alloc. Therefore it requires a gobal allocator. (defined by embedded_alloc)
use embedded_alloc::Heap;
#[global_allocator]
static HEAP: Heap = Heap::empty();

static channel_to_computer: Channel<ThreadModeRawMutex, items::Jog, 2> = Channel::new();
static channel_from_computer: Channel<ThreadModeRawMutex, items::Jog, 2> = Channel::new();

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    // Initialize the allocator. This is necessary for the alloc types and the PROST protocol buffer library
    {
        use core::mem::MaybeUninit;
        const HEAP_SIZE: usize = 8192;
        static mut HEAP_MEM: [MaybeUninit<u8>; HEAP_SIZE] = [MaybeUninit::uninit(); HEAP_SIZE];
        unsafe { HEAP.init(HEAP_MEM.as_ptr() as usize, HEAP_SIZE) }
    }

    let mut config = Config::default();

    init_clock(&mut config);

    // Create embassy-usb Config
    let peripherals = embassy_stm32::init(config);

    spawner
        .spawn(init_usb(
            peripherals.USB_OTG_FS,
            peripherals.PA12,
            peripherals.PA11,
            channel_to_computer.receiver(),
            channel_from_computer.sender(),
        ))
        .unwrap();

    spawner
        .spawn(run_stepper(
            peripherals.PB0.degrade(),
            channel_from_computer.receiver(),
        ))
        .unwrap();
}
