use embassy_stm32::time::mhz;
use embassy_stm32::Config;

pub fn clock(config: &mut Config) {
    config.rcc.hse = Some(mhz(8));
    config.rcc.pll48 = true;
    config.rcc.sys_ck = Some(mhz(200));
}
