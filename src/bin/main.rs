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
use esp_hal::{dma_buffers, Blocking};
use esp_hal::dma::{DmaRxBuf, DmaTxBuf};
use esp_hal::spi::BitOrder;
use esp_hal::spi::master::{Spi, SpiDmaBus};
use esp_hal::time::Rate;
use esp_hal::timer::systimer::SystemTimer;
use esp_println as _;

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

    let (rx_buffer, rx_descriptors, tx_buffer, tx_descriptors) = dma_buffers!(32000);
    let dma_rx_buf = DmaRxBuf::new(rx_descriptors, rx_buffer).unwrap();
    let dma_tx_buf = DmaTxBuf::new(tx_descriptors, tx_buffer).unwrap();

    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config.clone());

    let dma_channel = peripherals.DMA_CH0;
    let spi_config = esp_hal::spi::master::Config::default()
        .with_frequency(Rate::from_mhz(10))
        .with_mode(esp_hal::spi::Mode::_3)
        .with_read_bit_order(BitOrder::MsbFirst)
        .with_write_bit_order(BitOrder::MsbFirst);

    let spi = Spi::new(peripherals.SPI2, spi_config).unwrap()
        .with_cs(peripherals.GPIO8) // D5
        .with_sck(peripherals.GPIO7) // D4
        .with_mosi(peripherals.GPIO6)// D3
        .with_miso(peripherals.GPIO5)// D2
        .with_dma(dma_channel)
        .with_buffers(dma_rx_buf, dma_tx_buf);
    
    let timer0 = SystemTimer::new(peripherals.SYSTIMER);
    esp_hal_embassy::init(timer0.alarm0);

    info!("Embassy initialized!");

    spawner.spawn(spi_task(spi)).unwrap();
    // spi.transfer_in_place_async(&mut []).await.unwrap();

    // TODO: Spawn some tasks
    let _ = spawner;

    loop {
        info!("Hello world!");
        Timer::after(Duration::from_secs(1)).await;
    }

    // for inspiration have a look at the examples at https://github.com/esp-rs/esp-hal/tree/esp-hal-v1.0.0-beta.1/examples/src/bin
}

#[embassy_executor::task]
async fn spi_task(mut spi: SpiDmaBus<'static, Blocking>) {
    // Example SPI task that can be spawned
    let send_buffer = [0, 1, 2, 3, 4, 5, 6, 7];
    loop {
        let mut buffer = [0; 8];
        esp_println::println!("Sending bytes");
        spi.transfer(&mut buffer, &send_buffer).unwrap();
        esp_println::println!("Bytes received: {:?}", buffer);
        Timer::after(Duration::from_millis(5_000)).await;
    }
}
