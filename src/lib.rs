#![no_std]

use esp_hal::dma::{DmaRxBuf, DmaTxBuf};
use esp_hal::dma_buffers;

pub mod adc;
pub mod dac;

pub fn initialize_dma_buffers() -> (DmaRxBuf, DmaTxBuf) {
    let (rx_buffer, rx_descriptors, tx_buffer, tx_descriptors) = dma_buffers!(32000);
    let dma_rx_buf = DmaRxBuf::new(rx_descriptors, rx_buffer).unwrap();
    let dma_tx_buf = DmaTxBuf::new(tx_descriptors, tx_buffer).unwrap();
    (dma_rx_buf, dma_tx_buf)
}
