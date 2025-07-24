// // NOTE: This must match with the remapped pin sequence in pins_arduino.h
// static const int8_t TO_GPIO_NUMBER[] = {
//     44, // [ 0] D0, RX
//     43, // [ 1] D1, TX
//     5,  // [ 2] D2
//     6,  // [ 3] D3, CTS
//     7,  // [ 4] D4, DSR
//     8,  // [ 5] D5
//     9,  // [ 6] D6
//     10, // [ 7] D7
//     17, // [ 8] D8
//     18, // [ 9] D9
//     21, // [10] D10, SS
//     38, // [11] D11, MOSI
//     47, // [12] D12, MISO
//     48, // [13] D13, SCK, LED_BUILTIN
//     46, // [14] LED_RED
//     0,  // [15] LED_GREEN
//     45, // [16] LED_BLUE, RTS
//     1,  // [17] A0, DTR
//     2,  // [18] A1
//     3,  // [19] A2
//     4,  // [20] A3
//     11, // [21] A4, SDA
//     12, // [22] A5, SCL
//     13, // [23] A6
//     14, // [24] A7
// };

// to get this working needed to create a project using esp-generate with espflash as the flasher
// then ground the b1 pin. reset, flash, unground the b1 pin, unplug and replug and it worked
// https://github.com/esp-rs/espflash/issues/534#issuecomment-2307985591
#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]

use defmt::info;
use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use esp_hal::clock::CpuClock;
use esp_hal::{Blocking};
use esp_hal::spi::master::{SpiDmaBus};
use esp_hal::timer::systimer::SystemTimer;
use esp_println as _;
use dc_load_control_loop_rs::adc::ADC;
use dc_load_control_loop_rs::dac::DAC;

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

// This creates a default app-descriptor required by the esp-idf bootloader.
// For more information see: <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/app_image_format.html#application-description>
esp_bootloader_esp_idf::esp_app_desc!();

#[esp_hal_embassy::main]
async fn main(spawner: Spawner) {
    // generator version: 0.4.0
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config.clone());

    let adc = ADC::new_with_peripherals(
        peripherals.SPI2,
        peripherals.GPIO8,  // D5
        peripherals.GPIO7,  // D4
        peripherals.GPIO6,  // D3
        peripherals.GPIO5,  // D2
        peripherals.DMA_CH0,
    );

    info!("ADC initialized!");

    let dac = DAC::new_with_peripherals(
        peripherals.SPI3,
        peripherals.GPIO18, // D9
        peripherals.GPIO17, // D8
        peripherals.GPIO10, // D7
        peripherals.GPIO9,  // D6
        peripherals.DMA_CH1,
    );

    info!("DAC initialized!");

    let timer0 = SystemTimer::new(peripherals.SYSTIMER);
    esp_hal_embassy::init(timer0.alarm0);

    info!("Embassy initialized!");

    spawner.spawn(dac_task(dac)).unwrap();

    let _ = spawner;

    loop {
        info!("Hello world!");
        Timer::after(Duration::from_secs(1)).await;
    }

    // for inspiration have a look at the examples at https://github.com/esp-rs/esp-hal/tree/esp-hal-v1.0.0-beta.1/examples/src/bin
}

#[embassy_executor::task]
async fn dac_task(mut dac: DAC<'static, SpiDmaBus<'static, Blocking>>) {
    let mut value = 0u32;
    loop {
        esp_println::println!("Sending value to DAC: {}", value);
        dac.write(value).unwrap();
        value += 16;
        Timer::after(Duration::from_millis(5_00)).await;
    }
}
