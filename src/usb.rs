use alloc::vec::Vec;
use defmt::{panic, *};
use embassy_stm32::interrupt;
use embassy_stm32::usb_otg::Driver;
use embassy_usb::class::cdc_acm::{CdcAcmClass, State};
use embassy_usb::driver::EndpointError;
use embassy_usb::Builder;
use futures::future::join3;
extern crate alloc;
use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use embassy_sync::channel::{Receiver, Sender};

use crate::items;

pub struct Disconnected {}

impl From<EndpointError> for Disconnected {
    fn from(val: EndpointError) -> Self {
        match val {
            EndpointError::BufferOverflow => panic!("Buffer overflow"),
            EndpointError::Disabled => {
                info!("Endpoint disabled=============================================");
                Disconnected {}
            }
        }
    }
}

use prost::Message;

#[embassy_executor::task]
pub async fn init(
    usb_otg_peripheral: embassy_stm32::peripherals::USB_OTG_FS,
    dp: embassy_stm32::peripherals::PA12,
    dn: embassy_stm32::peripherals::PA11,
    channel_to_computer: Receiver<'static, ThreadModeRawMutex, items::Status, 2>,
    channel_from_computer: Sender<'static, ThreadModeRawMutex, items::Jog, 2>,
) {
    let irq = interrupt::take!(OTG_FS);
    let mut ep_out_buffer = [0u8; 256];

    let driver = Driver::new_fs(usb_otg_peripheral, irq, dp, dn, &mut ep_out_buffer);

    let mut config = embassy_usb::Config::new(0xc0de, 0xcafe);
    config.manufacturer = Some("Embassy");
    config.product = Some("USB-serial example");
    config.serial_number = Some("12345678");

    // Required for windows compatiblity.
    // https://developer.nordicsemi.com/nRF_Connect_SDK/doc/1.9.1/kconfig/CONFIG_CDC_ACM_IAD.html#help
    config.device_class = 0xEF;
    config.device_sub_class = 0x02;
    config.device_protocol = 0x01;
    config.composite_with_iads = true;

    // Create embassy-usb DeviceBuilder using the driver and config.
    // It needs some buffers for building the descriptors.
    let mut device_descriptor = [0; 256];
    let mut config_descriptor = [0; 256];
    let mut bos_descriptor = [0; 256];
    let mut control_buf = [0; 64];

    let mut state = State::new();

    let mut builder = Builder::new(
        driver,
        config,
        &mut device_descriptor,
        &mut config_descriptor,
        &mut bos_descriptor,
        &mut control_buf,
    );

    // Create classes on the builder.
    let class = CdcAcmClass::new(&mut builder, &mut state, 64);

    // Build the builder.
    let mut usb = builder.build();

    // Run the USB device.
    let usb_fut = usb.run();

    let (mut sender, mut receiver) = class.split();

    // Do stuff with the class!
    let receive_from_usb_future = async {
        loop {
            receiver.wait_connection().await;
            info!("Receive from USB Connected");
            let mut buf = [0; 1024];
            loop {
                match receiver.read_packet(&mut buf).await {
                    Ok(n) => {
                        let data = &buf[..n];

                        match items::Jog::decode(data) {
                            Ok(jog) => match channel_from_computer.try_send(jog) {
                                Ok(_) => {}
                                Err(_e) => {
                                    info!("Error sending data to channel");
                                }
                            },
                            Err(_e) => {
                                info!("Decode Error");
                            }
                        }
                    }
                    Err(e) => {
                        info!("Error: {:?}", e);
                    }
                }
            }
            //info!("Disconnected");
        }
    };

    let write_to_usb_future = async {
        loop {
            info!("Write to USB started");

            sender.wait_connection().await;
            info!("Write to USB Connected");
            let buf = [0u8; 1024];
            loop {
                // Receive data from the channel
                let jog = channel_to_computer.recv().await;
                let mut buf = Vec::new();
                // Encode the received data
                if let Err(_e) = jog.encode(&mut buf) {
                    info!("Encode Error");
                } else {
                    // Write encoded data to USB
                    match sender.write_packet(&buf).await {
                        Ok(_) => {
                            info!("Sending data success");
                        }
                        Err(e) => {
                            info!("Error: {:?}", e);
                            return;
                        }
                    }
                }
            }
            //info!("Disconnected");
        }
    };

    // Run everything concurrently.
    // If we had made everything `'static` above instead, we could do this using separate tasks instead.
    join3(usb_fut, receive_from_usb_future, write_to_usb_future).await;

    // TODO: JOIN THE THREE TASKS
}
