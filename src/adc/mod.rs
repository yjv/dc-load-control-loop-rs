use defmt::{debug, Format};
use embedded_hal::spi::SpiBus;
use crate::adc::register::{Register, RegisterRW, WritableRegister};

// Macro to define enums with integer discriminants and implement into_bits/from_bits
#[macro_export]
macro_rules! bitfield_enum {
    (
        $(#[$meta:meta])* $vis:vis enum $name:ident : $repr:ty {
            $(
                $(#[$variant_meta:meta])* $variant:ident = $value:expr
            ),+ $(,)?
        }
    ) => {
        $(#[$meta])* #[repr($repr)] $vis enum $name {
            $( $(#[$variant_meta])* $variant = $value ),+
        }
        impl $name {
            pub const fn into_bits(self) -> $repr { self as $repr }
            pub const fn from_bits(value: $repr) -> Self {
                match value {
                    $( $value => $name::$variant, )+
                    _ => panic!(concat!("Invalid value for ", stringify!($name))),
                }
            }
        }
    };
}

pub mod register;

struct ADC<Bus: SpiBus> {
    spi: Bus,
    buf: [u8; 6],
}

impl <Bus: SpiBus> ADC<Bus> {

    pub fn new(spi: Bus) -> Self {
        Self {
            spi,
            buf: [0; 6],
        }
    }

    pub fn read<const N: usize, T: Register<N>>(&mut self) -> Result<T, Bus::Error> {
        let id = T::get_id();
        self.buf[0] = id | RegisterRW::Read as u8;

        debug!("Writing register: {:02x} {:012x}", id, self.buf);
        self.spi.transfer_in_place(&mut self.buf[..N + 1])?;

        let mut register_buf: [u8; N] = [0; N];
        register_buf.copy_from_slice(&self.buf[1..N + 1]);

        debug!("Writing register: {:06x}", self.buf);

        Ok(T::from_buffer((&self.buf[1..N + 1]).try_into().unwrap()))
    }

    pub fn write<const N: usize, T: WritableRegister<N>>(&mut self, register: &T) -> Result<(), Bus::Error> {
        let id = T::get_id();
        self.buf[0] = id | RegisterRW::Write as u8;
        self.buf[1..N + 1].copy_from_slice(&register.to_buffer());

        debug!("Writing register: {:02x} {:012x}", id, self.buf);

        self.spi.write(&self.buf[..N + 1])
    }
}

bitfield_enum! {
    /// ADC channel selection.
    ///
    /// Used in the Status Register and Channel Registers to select or indicate the active channel.
    #[derive(Format, Debug, Eq, PartialEq)]
    pub enum Channel: u8 {
        /// Channel 0
        Ch0 = 0x00,
        /// Channel 1
        Ch1 = 0x01,
        /// Channel 2
        Ch2 = 0x02,
        /// Channel 3
        Ch3 = 0x03,
    }
}

bitfield_enum! {
    /// Delay setting for the ADC conversion start.
    ///
    /// Used in the ADC Mode Register to select the delay between conversions.
    /// The delay can be used to allow external circuitry to settle before a conversion starts.
    #[derive(Format, Debug, Eq, PartialEq)]
    pub enum Delay: u8 {
        /// No delay (0 µs)
        ZeroMicroseconds = 0x00,
        /// 4 microseconds delay
        FourMicroseconds = 0x01,
        /// 16 microseconds delay
        SixteenMicroseconds = 0x02,
        /// 40 microseconds delay
        FortyMicroseconds = 0x03,
        /// 100 microseconds delay
        OneHundredMicroseconds = 0x04,
        /// 200 microseconds delay
        TwoHundredMicroseconds = 0x05,
        /// 500 microseconds delay
        FiveHundredMicroseconds = 0x06,
        /// 1 millisecond delay
        OneMillisecond = 0x07,
    }
}

bitfield_enum! {
    /// ADC operating mode.
    ///
    /// Used in the ADC Mode Register to select the conversion mode or calibration operation.
    #[derive(Format, Debug, Eq, PartialEq)]
    pub enum Mode: u8 {
        /// Continuous conversion mode. The ADC continuously converts and updates the data register.
        ContinuousConversion = 0x00,
        /// Single conversion mode. The ADC performs a single conversion and then enters standby.
        SingleConversion = 0x01,
        /// Standby mode. The ADC enters a low-power standby state.
        Standby = 0x02,
        /// Power-down mode. The ADC enters a deep power-down state.
        PowerDown = 0x03,
        /// Internal offset calibration. The ADC performs an internal offset calibration.
        InternalOffsetCalibration = 0x04,
        /// System offset calibration. The ADC performs a system offset calibration.
        SystemOffsetCalibration = 0x06,
        /// System gain calibration. The ADC performs a system gain calibration.
        SystemGainCalibration = 0x07,
    }
}

bitfield_enum! {
    /// Clock source selection for the ADC.
    ///
    /// Used in the ADC Mode Register to select the master clock source.
    #[derive(Format, Debug, Eq, PartialEq)]
    pub enum ClockSource: u8 {
        /// Internal oscillator (default, 16 MHz).
        Internal = 0x00,
        /// Internal oscillator with clock output on XTAL2/CLKIO pin.
        InternalWithOutput = 0x01,
        /// External clock supplied to XTAL2/CLKIO pin.
        External = 0x02,
        /// External crystal connected between XTAL1 and XTAL2/CLKIO.
        ExternalCrystal = 0x03,
    }
}

bitfield_enum! {
    /// CRC mode for communication error checking.
    ///
    /// Used in the Interface Mode Register to select CRC or XOR error checking.
    #[derive(Format, Debug, Eq, PartialEq)]
    pub enum Crc: u8 {
        /// CRC disabled.
        Disabled = 0x00,
        /// Enable CRC with XOR on readback only.
        EnableWithXorOnRead = 0x01,
        /// Enable full CRC on all communication.
        Enable = 0x02,
    }
}

bitfield_enum! {
    /// Data register length selection.
    ///
    /// Used in the Interface Mode Register to select the number of bits in the data register.
    #[derive(Format, Debug, Eq, PartialEq)]
    pub enum DataRegisterLength: u8 {
        /// 24-bit data register (default).
        TwentyFourBits = 0x00,
        /// 16-bit data register (truncates LSBs).
        SixteenBits = 0x01,
    }
}

bitfield_enum! {
    /// SYNC/ERROR pin mode selection.
    ///
    /// Used in the GPIO Configuration Register to select the function of the SYNC/ERROR pin.
    #[derive(Format, Debug, Eq, PartialEq)]
    pub enum SyncErrorPinMode: u8 {
        /// Pin disabled.
        Disabled = 0x00,
        /// Error input mode. Pin acts as an error input.
        ErrorInput = 0x01,
        /// Open-drain error output mode. Pin outputs error status.
        OpenDrainErrorOutput = 0x02,
        /// Error data output mode. Pin outputs error data.
        ErrDataOutput = 0x03,
    }
}

bitfield_enum! {
    /// Setup selection for channel configuration.
    ///
    /// Used in Channel Registers to select which setup configuration to use for a channel.
    #[derive(Format, Debug, Eq, PartialEq)]
    pub enum Setup: u8 {
        /// Use Setup 0
        Setup0 = 0x00,
        /// Use Setup 1
        Setup1 = 0x01,
        /// Use Setup 2
        Setup2 = 0x02,
        /// Use Setup 3
        Setup3 = 0x03,
    }
}

bitfield_enum! {
    /// Input multiplexer selection.
    ///
    /// Used in Channel Registers to select the positive or negative input for a channel.
    #[derive(Format, Debug, Eq, PartialEq)]
    pub enum Input: u8 {
        /// Analog input 0
        Analog0 = 0x00,
        /// Analog input 1
        Analog1 = 0x01,
        /// Analog input 2
        Analog2 = 0x02,
        /// Analog input 3
        Analog3 = 0x03,
        /// Analog input 4
        Analog4 = 0x04,
        /// Temperature sensor positive
        TemperatureSensorPos = 0x11,
        /// Temperature sensor negative
        TemperatureSensorNeg = 0x12,
        /// AVDD1-AVSS/5 positive
        Avdd1AvssDiffOver5Pos = 0x13,
        /// AVDD1-AVSS/5 negative
        Avdd1AvssDiffOver5Neg = 0x14,
        /// Positive reference voltage
        PositiveReferenceVoltage = 0x15,
        /// Negative reference voltage
        NegativeReferenceVoltage = 0x16,
    }
}

bitfield_enum! {
    /// Output coding mode for ADC data.
    ///
    /// Used in Setup Configuration Registers to select unipolar or bipolar output coding.
    #[derive(Format, Debug, Eq, PartialEq)]
    pub enum OutputCoding: u8 {
        /// Unipolar output coding.
        Unipolar = 0x00,
        /// Bipolar output coding.
        Bipolar = 0x01,
    }
}

bitfield_enum! {
    /// Reference source selection for ADC conversions.
    ///
    /// Used in Setup Configuration Registers to select the reference source.
    #[derive(Format, Debug, Eq, PartialEq)]
    pub enum ReferenceSource: u8 {
        /// External reference.
        External = 0x00,
        /// Internal reference.
        Internal = 0x01,
        /// AVDD1-AVSS differential reference.
        Avdd1AvssDiff = 0x02,
    }
}

bitfield_enum! {
    /// Enhanced filter rate selection.
    ///
    /// Used in Filter Configuration Registers to select the enhanced filter rate.
    #[derive(Format, Debug, Eq, PartialEq)]
    pub enum EnhancedFilterRate: u8 {
        /// 27 SPS
        Sps27 = 0x02,
        /// 25 SPS
        Sps25 = 0x03,
        /// 20 SPS
        Sps20 = 0x05,
        /// 16.67 SPS
        Sps16p67 = 0x06,
    }
}

bitfield_enum! {
    /// Digital filter order selection.
    ///
    /// Used in Filter Configuration Registers to select the filter order.
    #[derive(Format, Debug, Eq, PartialEq)]
    pub enum FilterOrder: u8 {
        /// Sinc5 + Sinc1 filter order.
        Sinc5Sinc1 = 0x00,
        /// Sinc3 filter order.
        Sinc3 = 0x03,
    }
}

bitfield_enum! {
    /// Output data rate selection for ADC conversions.
    ///
    /// Used in Filter Configuration Registers to select the output data rate.
    #[derive(Format, Debug, Eq, PartialEq)]
    pub enum OutputDataRate: u8 {
        /// 250,000 samples per second
        Sps250000 = 0x00,
        /// 125,000 samples per second
        Sps125000 = 0x01,
        /// 62,500 samples per second
        Sps62500 = 0x02,
        /// 50,000 samples per second
        Sps50000 = 0x03,
        /// 31,250 samples per second
        Sps31250 = 0x04,
        /// 25,000 samples per second
        Sps25000 = 0x05,
        /// 15,625 samples per second
        Sps15625 = 0x06,
        /// 10,000 samples per second
        Sps10000 = 0x07,
        /// 5,000 samples per second
        Sps5000 = 0x08,
        /// 2,500 samples per second
        Sps2500 = 0x09,
        /// 1,000 samples per second
        Sps1000 = 0x0a,
        /// 500 samples per second
        Sps500 = 0x0b,
        /// 397.5 samples per second
        Sps397p5 = 0x0c,
        /// 200 samples per second
        Sps200 = 0x0d,
        /// 100 samples per second
        Sps100 = 0x0e,
        /// 59.92 samples per second
        Sps59p92 = 0x0f,
        /// 49.96 samples per second
        Sps49p96 = 0x10,
        /// 20 samples per second
        Sps20 = 0x11,
        /// 16.67 samples per second
        Sps16p67 = 0x12,
        /// 10 samples per second
        Sps10 = 0x13,
        /// 5 samples per second
        Sps5 = 0x14,
    }
}