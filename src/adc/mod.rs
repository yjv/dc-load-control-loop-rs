use defmt::{debug, Format};
use embedded_hal::spi::SpiBus;
use crate::adc::register::{Register, RegisterRW, WritableRegister};

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

#[derive(Format, Debug, Eq, PartialEq)]
pub enum Channel {
    Ch0 = 0x00,
    Ch1 = 0x01,
    Ch2 = 0x02,
    Ch3 = 0x03,
}

impl Channel {
    pub const fn into_bits(self) -> u8 {
        self as u8
    }

    pub const fn from_bits(value: u8) -> Self {
        match value {
            0x00 => Channel::Ch0,
            0x01 => Channel::Ch1,
            0x02 => Channel::Ch2,
            0x03 => Channel::Ch3,
            _ => panic!("Invalid channel value"),
        }
    }
}

#[derive(Format, Debug, Eq, PartialEq)]
pub enum Delay {
    ZeroMicroseconds = 0x00,
    FourMicroseconds = 0x01,
    SixteenMicroseconds = 0x02,
    FortyMicroseconds = 0x03,
    OneHundredMicroseconds = 0x04,
    TwoHundredMicroseconds = 0x05,
    FiveHundredMicroseconds = 0x06,
    OneMillisecond = 0x07,
}

impl Delay {
    pub const fn into_bits(self) -> u8 {
        self as u8
    }

    pub const fn from_bits(value: u8) -> Self {
        match value {
            0x00 => Delay::ZeroMicroseconds,
            0x01 => Delay::FourMicroseconds,
            0x02 => Delay::SixteenMicroseconds,
            0x03 => Delay::FortyMicroseconds,
            0x04 => Delay::OneHundredMicroseconds,
            0x05 => Delay::TwoHundredMicroseconds,
            0x06 => Delay::FiveHundredMicroseconds,
            0x07 => Delay::OneMillisecond,
            _ => panic!("Invalid delay value"),
        }
    }
}

#[derive(Format, Debug, Eq, PartialEq)]
pub enum Mode {
    ContinuousConversion = 0x00,
    SingleConversion = 0x01,
    Standby = 0x02,
    PowerDown = 0x03,
    InternalOffsetCalibration = 0x04,
    SystemOffsetCalibration = 0x06,
    SystemGainCalibration = 0x07,
}

impl Mode {
    pub const fn into_bits(self) -> u8 {
        self as u8
    }

    pub const fn from_bits(value: u8) -> Self {
        match value {
            0x00 => Mode::ContinuousConversion,
            0x01 => Mode::SingleConversion,
            0x02 => Mode::Standby,
            0x03 => Mode::PowerDown,
            0x04 => Mode::InternalOffsetCalibration,
            0x06 => Mode::SystemOffsetCalibration,
            0x07 => Mode::SystemGainCalibration,
            _ => panic!("Invalid mode value"),
        }
    }
}

#[derive(Format, Debug, Eq, PartialEq)]
pub enum ClockSource {
    Internal = 0x00,
    InternalWithOutput = 0x01,
    External = 0x02,
    ExternalCrystal = 0x03,
}

impl ClockSource {
    pub const fn into_bits(self) -> u8 {
        self as u8
    }

    pub const fn from_bits(value: u8) -> Self {
        match value {
            0x00 => ClockSource::Internal,
            0x01 => ClockSource::InternalWithOutput,
            0x02 => ClockSource::External,
            0x03 => ClockSource::ExternalCrystal,
            _ => panic!("Invalid clock source value"),
        }
    }
}

#[derive(Format, Debug, Eq, PartialEq)]
pub enum Crc {
    Disabled = 0x00,
    EnableWithXorOnRead = 0x01,
    Enable = 0x02,
}

impl Crc {
    pub const fn into_bits(self) -> u8 {
        self as u8
    }

    pub const fn from_bits(value: u8) -> Self {
        match value {
            0x00 => Crc::Disabled,
            0x01 => Crc::EnableWithXorOnRead,
            0x02 => Crc::Enable,
            _ => panic!("Invalid CRC value"),
        }
    }
}

#[derive(Format, Debug, Eq, PartialEq)]
pub enum DataRegisterLength {
    TwentyFourBits = 0x00,
    SixteenBits = 0x01,
}

impl DataRegisterLength {
    pub const fn into_bits(self) -> u8 {
        self as u8
    }

    pub const fn from_bits(value: u8) -> Self {
        match value {
            0x00 => DataRegisterLength::TwentyFourBits,
            0x01 => DataRegisterLength::SixteenBits,
            _ => panic!("Invalid data register length value"),
        }
    }
}

#[derive(Format, Debug, Eq, PartialEq)]
pub enum SyncErrorPinMode {
    Disabled = 0x00,
    ErrorInput = 0x01,
    OpenDrainErrorOutput = 0x02,
    ErrDataOutput = 0x03,
}

impl SyncErrorPinMode {
    pub const fn into_bits(self) -> u8 {
        self as u8
    }

    pub const fn from_bits(value: u8) -> Self {
        match value {
            0x00 => SyncErrorPinMode::Disabled,
            0x01 => SyncErrorPinMode::ErrorInput,
            0x02 => SyncErrorPinMode::OpenDrainErrorOutput,
            0x03 => SyncErrorPinMode::ErrDataOutput,
            _ => panic!("Invalid sync error pin mode value"),
        }
    }
}

#[derive(Format, Debug, Eq, PartialEq)]
pub enum Setup {
    Setup0 = 0x00,
    Setup1 = 0x01,
    Setup2 = 0x02,
    Setup3 = 0x03,
}

impl Setup {
    pub const fn into_bits(self) -> u8 {
        self as u8
    }

    pub const fn from_bits(value: u8) -> Self {
        match value {
            0x00 => Setup::Setup0,
            0x01 => Setup::Setup1,
            0x02 => Setup::Setup2,
            0x03 => Setup::Setup3,
            _ => panic!("Invalid setup value"),
        }
    }
}

#[derive(Format, Debug, Eq, PartialEq)]
pub enum Input {
    Analog0 = 0x00,
    Analog1 = 0x01,
    Analog2 = 0x02,
    Analog3 = 0x03,
    Analog4 = 0x04,
    TemperatureSensorPos = 0x11,
    TemperatureSensorNeg = 0x12,
    Avdd1AvssDiffOver5Pos = 0x13,
    Avdd1AvssDiffOver5Neg = 0x14,
    PositiveReferenceVoltage = 0x15,
    NegativeReferenceVoltage = 0x16,
}

impl Input {
    pub const fn into_bits(self) -> u8 {
        self as u8
    }

    pub const fn from_bits(value: u8) -> Self {
        match value {
            0x00 => Input::Analog0,
            0x01 => Input::Analog1,
            0x02 => Input::Analog2,
            0x03 => Input::Analog3,
            0x04 => Input::Analog4,
            0x11 => Input::TemperatureSensorPos,
            0x12 => Input::TemperatureSensorNeg,
            0x13 => Input::Avdd1AvssDiffOver5Pos,
            0x14 => Input::Avdd1AvssDiffOver5Neg,
            0x15 => Input::PositiveReferenceVoltage,
            0x16 => Input::NegativeReferenceVoltage,
            _ => panic!("Invalid input value"),
        }
    }
}

#[derive(Format, Debug, Eq, PartialEq)]
pub enum OutputCoding {
    Unipolar = 0x00,
    Bipolar = 0x01,
}

impl OutputCoding {
    pub const fn into_bits(self) -> u8 {
        self as u8
    }

    pub const fn from_bits(value: u8) -> Self {
        match value {
            0x00 => OutputCoding::Unipolar,
            0x01 => OutputCoding::Bipolar,
            _ => panic!("Invalid output coding value"),
        }
    }
}

#[derive(Format, Debug, Eq, PartialEq)]
pub enum ReferenceSource {
    External = 0x00,
    Internal = 0x01,
    Avdd1AvssDiff = 0x02,
}

impl ReferenceSource {
    pub const fn into_bits(self) -> u8 {
        self as u8
    }

    pub const fn from_bits(value: u8) -> Self {
        match value {
            0x00 => ReferenceSource::External,
            0x01 => ReferenceSource::Internal,
            0x02 => ReferenceSource::Avdd1AvssDiff,
            _ => panic!("Invalid reference source value"),
        }
    }
}

#[derive(Format, Debug, Eq, PartialEq)]
pub enum EnhancedFilterRate {
    Sps27 = 0x02,
    Sps25 = 0x03,
    Sps20 = 0x05,
    Sps16p67 = 0x06,
}

impl EnhancedFilterRate {
    pub const fn into_bits(self) -> u8 {
        self as u8
    }

    pub const fn from_bits(value: u8) -> Self {
        match value {
            0x02 => EnhancedFilterRate::Sps27,
            0x03 => EnhancedFilterRate::Sps25,
            0x05 => EnhancedFilterRate::Sps20,
            0x06 => EnhancedFilterRate::Sps16p67,
            _ => panic!("Invalid enhanced filter rate value"),
        }
    }
}

#[derive(Format, Debug, Eq, PartialEq)]
pub enum FilterOrder {
    Sinc5Sinc1 = 0x00,
    Sinc3 = 0x03,
}

impl FilterOrder {
    pub const fn into_bits(self) -> u8 {
        self as u8
    }

    pub const fn from_bits(value: u8) -> Self {
        match value {
            0x00 => FilterOrder::Sinc5Sinc1,
            0x01 => FilterOrder::Sinc3,
            _ => panic!("Invalid filter order value"),
        }
    }
}

#[derive(Format, Debug, Eq, PartialEq)]
pub enum OutputDataRate {
    Sps250000 = 0x00,
    Sps125000 = 0x01,
    Sps62500 = 0x02,
    Sps50000 = 0x03,
    Sps31250 = 0x04,
    Sps25000 = 0x05,
    Sps15625 = 0x06,
    Sps10000 = 0x07,
    Sps5000 = 0x08,
    Sps2500 = 0x09,
    Sps1000 = 0x0a,
    Sps500 = 0x0b,
    Sps397p5 = 0x0c,
    Sps200 = 0x0d,
    Sps100 = 0x0e,
    Sps59p92 = 0x0f,
    Sps49p96 = 0x10,
    Sps20 = 0x11,
    Sps16p67 = 0x12,
    Sps10 = 0x13,
    Sps5 = 0x14,
}

impl OutputDataRate {
    pub const fn into_bits(self) -> u8 {
        self as u8
    }

    pub const fn from_bits(value: u8) -> Self {
        match value {
            0x00 => OutputDataRate::Sps250000,
            0x01 => OutputDataRate::Sps125000,
            0x02 => OutputDataRate::Sps62500,
            0x03 => OutputDataRate::Sps50000,
            0x04 => OutputDataRate::Sps31250,
            0x05 => OutputDataRate::Sps25000,
            0x06 => OutputDataRate::Sps15625,
            0x07 => OutputDataRate::Sps10000,
            0x08 => OutputDataRate::Sps5000,
            0x09 => OutputDataRate::Sps2500,
            0x0a => OutputDataRate::Sps1000,
            0x0b => OutputDataRate::Sps500,
            0x0c => OutputDataRate::Sps397p5,
            0x0d => OutputDataRate::Sps200,
            0x0e => OutputDataRate::Sps100,
            0x0f => OutputDataRate::Sps59p92,
            0x10 => OutputDataRate::Sps49p96,
            0x11 => OutputDataRate::Sps20,
            0x12 => OutputDataRate::Sps16p67,
            0x13 => OutputDataRate::Sps10,
            0x14 => OutputDataRate::Sps5,
            _ => panic!("Invalid output data rate value"),
        }
    }
}