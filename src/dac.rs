use embedded_hal::spi::SpiBus;
use esp_hal::Blocking;
use esp_hal::dma::DmaChannelFor;
use esp_hal::gpio::{NoPin, Output, OutputConfig, OutputPin};
use esp_hal::spi::{AnySpi, BitOrder};
use esp_hal::spi::master::{Config, Instance, Spi, SpiDmaBus};
use esp_hal::time::Rate;
use crate::initialize_dma_buffers;

#[derive(Debug)]
pub struct DAC<'d, Bus: SpiBus> {
    spi: Bus,
    ldac_pin: Output<'d>,
}

impl <'d> DAC<'d, SpiDmaBus<'d, Blocking>> {
    
    pub fn get_spi_config() -> Config {
        Config::default()
            .with_frequency(Rate::from_khz(500))
            .with_mode(esp_hal::spi::Mode::_0)
            .with_read_bit_order(BitOrder::MsbFirst)
            .with_write_bit_order(BitOrder::MsbFirst)
    }
    
    pub fn new_with_peripherals<SpiInstance: Instance + 'static, CS: OutputPin + 'static, SCK: OutputPin + 'static, MOSI: OutputPin + 'static, LDAC: OutputPin + 'static, DmaChannel: DmaChannelFor<AnySpi<'d>>>(spi: SpiInstance, cs: CS, sck: SCK, mosi: MOSI, ldac: LDAC, dma_channel: DmaChannel) -> Self {
        let (dma_rx_buf, dma_tx_buf) = initialize_dma_buffers();

        let dac_spi = Spi::new(spi, Self::get_spi_config()).unwrap()
            .with_cs(cs)
            .with_sck(sck)
            .with_mosi(mosi)
            .with_miso(NoPin) // no need for MISO in DAC
            .with_dma(dma_channel)
            .with_buffers(dma_rx_buf, dma_tx_buf);

        let ldac_pin = Output::new(ldac, esp_hal::gpio::Level::High, OutputConfig::default()); // D6

        Self::new(dac_spi, ldac_pin)
    }
}

impl<'d, Bus: SpiBus> DAC<'d, Bus> {
    pub fn new(spi: Bus, ldac_pin: Output<'d>) -> Self {
        DAC {
            spi,
            ldac_pin,
        }
    }

    pub fn write(&mut self, value: u32) -> Result<(), Bus::Error> {
        self.spi.write(&value.to_be_bytes())?;
        self.ldac_pin.set_low();
        self.ldac_pin.set_high();
        Ok(())
    }
}

