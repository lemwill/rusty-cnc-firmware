use bytes::Bytes;
use defmt::{panic, *};
use embassy_stm32::gpio::AnyPin;
use embassy_stm32::usb_otg::{Driver, Instance};
use embassy_stm32::{interrupt, Peripherals};
use embassy_usb::class::cdc_acm::{CdcAcmClass, State};
use embassy_usb::driver::EndpointError;
use embassy_usb::Builder;
use futures::future::join;
extern crate alloc;
use alloc::vec::Vec;
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

pub mod items {
    include!(concat!(env!("OUT_DIR"), "/messages_proto.rs"));
}

pub async fn echo<'d, T: Instance + 'd>(
    class: &mut CdcAcmClass<'d, Driver<'d, T>>,
) -> Result<(), Disconnected> {
    let mut buf = [0; 1024];
    loop {
        match class.read_packet(&mut buf).await {
            Ok(n) => {
                let data = &buf[..n];
                info!("data: {:x}", data);

                match items::Jog::decode(data) {
                    Ok(mut jog) => {
                        info!("Axis: {:#?}", jog.axis);
                        info!("Direction: {:#?}", jog.direction);
                    }
                    Err(_e) => {
                        info!("Decode Error");
                    }
                }

                match class.write_packet(data).await {
                    Ok(_) => {
                        info!("Sending data success");
                    }
                    Err(e) => {
                        info!("Error: {:?}", e);
                        return Err(Disconnected {});
                    }
                }
            }
            Err(e) => {
                info!("Error: {:?}", e);
            }
        }
        // let n = class.read_packet(&mut buf).await?;

        /*match items::Shirt::decode(data) {
            Ok(mut shirt) => {
                info!("Color: {:#?}", shirt.color);
                info!("Color2: {:#?}", shirt.color2);

                shirt.color = shirt.color + 1;

                let mut buf2 = Vec::new();
                shirt.encode(&mut buf2).unwrap();
                class.write_packet(&buf2).await?;
            }
            Err(_e) => {
                info!("Decode Error");
            }
        }*/
    }
}
#[embassy_executor::task]
pub async fn init_usb(
    usb_otg_peripheral: embassy_stm32::peripherals::USB_OTG_FS,
    dp: embassy_stm32::peripherals::PA12,
    dn: embassy_stm32::peripherals::PA11,
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
    let mut class = CdcAcmClass::new(&mut builder, &mut state, 64);

    // Build the builder.
    let mut usb = builder.build();

    // Run the USB device.
    let usb_fut = usb.run();

    // Do stuff with the class!
    let echo_fut = async {
        loop {
            class.wait_connection().await;
            info!("Connected");
            let _ = echo(&mut class).await;
            info!("Disconnected");
        }
    };

    // Run everything concurrently.
    // If we had made everything `'static` above instead, we could do this using separate tasks instead.
    join(usb_fut, echo_fut).await;
}
