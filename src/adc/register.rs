use crate::adc::{Channel, ClockSource, Crc, DataRegisterLength, Delay, EnhancedFilterRate, FilterOrder, Input, Mode, OutputCoding, OutputDataRate, ReferenceSource, Setup, SyncErrorPinMode};

pub trait Register<const BUFF_LEN: usize> {
    fn get_id() -> u8;
    fn from_buffer(raw: &[u8; BUFF_LEN]) -> Self;
}

pub trait WritableRegister<const BUFF_LEN: usize>: Register<BUFF_LEN> {
    fn to_buffer(&self) -> [u8; BUFF_LEN];
}

pub enum RegisterRW {
    Read = 0x40,
    Write = 0x00,
}

const fn from_u32(val: u32) -> [u8; 3] {
    [
        (val >> 16) as u8,
        (val >> 8) as u8,
        val as u8,
    ]
}

const fn into_u32(slice: [u8; 3]) -> u32 {
    ((slice[0] as u32) << 16) | ((slice[1] as u32) << 8) | (slice[2] as u32)
}

macro_rules! register {
    // Single struct with doc
    ($(#[$meta:meta])* $name:ident { $($field:tt)* }, 1, $id:expr) => {
        #[bitfield_struct::bitfield(u8, repr = [u8; 1], from = u8::to_ne_bytes, into = u8::from_ne_bytes, defmt = true, order = msb)]
        $(#[$meta])*
        pub struct $name {
            $($field)*
        }
        impl Register<1> for $name {
            fn get_id() -> u8 { $id }
            fn from_buffer(raw: &[u8; 1]) -> Self { Self::from_bits(*raw) }
        }
    };
    ($(#[$meta:meta])* $name:ident { $($field:tt)* }, 2, $id:expr) => {
        #[bitfield_struct::bitfield(u16, repr = [u8; 2], from = u16::to_ne_bytes, into = u16::from_ne_bytes, defmt = true, order = msb)]
        $(#[$meta])*
        pub struct $name {
            $($field)*
        }
        impl Register<2> for $name {
            fn get_id() -> u8 { $id }
            fn from_buffer(raw: &[u8; 2]) -> Self { Self::from_bits(*raw) }
        }
    };
    ($(#[$meta:meta])* $name:ident { $($field:tt)* }, 3, $id:expr) => {
        #[bitfield_struct::bitfield(u32, repr = [u8; 3], from = from_u32, into = into_u32, defmt = true, order = msb)]
        $(#[$meta])*
        pub struct $name {
            $($field)*
            #[bits(8)]
            ___: u8, // Padding to ensure 32 bits total
        }
        impl Register<3> for $name {
            fn get_id() -> u8 { $id }
            fn from_buffer(raw: &[u8; 3]) -> Self {
                Self::from_bits(*raw)
            }
        }
    };
    ($(#[$meta:meta])* $name:ident { $($field:tt)* }, 4, $id:expr) => {
        #[bitfield_struct::bitfield(u32, repr = [u8; 4], from = u32::to_ne_bytes, into = u32::from_ne_bytes, defmt = true, order = msb)]
        $(#[$meta])*
        pub struct $name {
            $($field)*
        }
        impl Register<4> for $name {
            fn get_id() -> u8 { $id }
            fn from_buffer(raw: &[u8; 4]) -> Self {
                Self::from_bits(*raw)
            }
        }
    };
    // Multi-register: doc comment and field block applied to all
    ($meta:tt ; $fields:tt, $len:expr, $(($name:ident, $id:expr)),+ $(,)?) => {
        $(
            register! { $meta $name $fields, $len, $id }
        )+
    };
}

macro_rules! rw_register {
    // Single struct with doc
    ($(#[$meta:meta])* $name:ident { $($field:tt)* }, 1, $id:expr) => {
        register!($(#[$meta])* $name { $($field)* }, 1, $id);
        impl WritableRegister<1> for $name   {
            fn to_buffer(&self) -> [u8; 1] { self.into_bits() }
        }
    };
    ($(#[$meta:meta])* $name:ident { $($field:tt)* }, 2, $id:expr) => {
        register!($(#[$meta])* $name { $($field)* }, 2, $id);
        impl WritableRegister<2> for $name   {
            fn to_buffer(&self) -> [u8; 2] { self.into_bits() }
        }
    };
    ($(#[$meta:meta])* $name:ident { $($field:tt)* }, 3, $id:expr) => {
        register!($(#[$meta])* $name { $($field)* }, 3, $id);
        impl WritableRegister<3> for $name   {
            fn to_buffer(&self) -> [u8; 3] {
                self.into_bits()
            }
        }
    };
    ($(#[$meta:meta])* $name:ident { $($field:tt)* }, 4, $id:expr) => {
        register!($(#[$meta])* $name { $($field)* }, 4, $id);
        impl WritableRegister<4> for $name   {
            fn to_buffer(&self) -> [u8; 4] {
                self.into_bits()
            }
        }
    };
    // Multi-register: doc comment and field block applied to all
    ($meta:tt ; $fields:tt, $len:expr, $(($name:ident, $id:expr)),+ $(,)?) => {
        $(
            rw_register! { $meta $name $fields, $len, $id }
        )+
    };
}

// Macro to generate multiple registers from a single doc+struct block and a list of names/ids
macro_rules! multi_rw_register {
    ($docs_and_struct:tt, 1, $(($name:ident, $id:expr)),+ $(,)?) => {
        $(
            multi_rw_register!(@emit $docs_and_struct, $name, 1, $id);
        )+
    };
    (@emit
        {
            $(#[$meta:meta])*
            $vis:vis struct $base:ident { $($fields:tt)* }
        },
        $name:ident,
        1, 
        $id:expr
    ) => {
        register!($(#[$meta])* $name { $($fields)* }, 1, $id);
        // Optionally, you can add a const or impl block for $id here
        // pub const $name _ID: u8 = $id;
    };
    ($docs_and_struct:tt, 2, $(($name:ident, $id:expr)),+ $(,)?) => {
        $(
            multi_rw_register!(@emit $docs_and_struct, $name, 2, $id);
        )+
    };
    (@emit
        {
            $(#[$meta:meta])*
            $vis:vis struct $base:ident { $($fields:tt)* }
        },
        $name:ident,
        2, 
        $id:expr
    ) => {
        register!($(#[$meta])* $name { $($fields)* }, 2, $id);
        // Optionally, you can add a const or impl block for $id here
        // pub const $name _ID: u8 = $id;
    };
    ($docs_and_struct:tt, 3, $(($name:ident, $id:expr)),+ $(,)?) => {
        $(
            multi_rw_register!(@emit $docs_and_struct, $name, 3, $id);
        )+
    };
    (@emit
        {
            $(#[$meta:meta])*
            $vis:vis struct $base:ident { $($fields:tt)* }
        },
        $name:ident,
        3, 
        $id:expr
    ) => {
        register!($(#[$meta])* $name { $($fields)* }, 3, $id);
        // Optionally, you can add a const or impl block for $id here
        // pub const $name _ID: u8 = $id;
    };
    ($docs_and_struct:tt, 4, $(($name:ident, $id:expr)),+ $(,)?) => {
        $(
            multi_rw_register!(@emit $docs_and_struct, $name, 4, $id);
        )+
    };
    (@emit
        {
            $(#[$meta:meta])*
            $vis:vis struct $base:ident { $($fields:tt)* }
        },
        $name:ident,
        4, 
        $id:expr
    ) => {
        register!($(#[$meta])* $name { $($fields)* }, 4, $id);
    };
}

register!(
    /// Status Register (0x00)
    /// Indicates ADC status, error flags, and current channel.
    ///
    /// | Bit | Name            | Description                       |
    /// |-----|-----------------|-----------------------------------|
    /// | 7   | READY           | Data ready flag. Set to true when new conversion data is available. |
    /// | 6   | ADC_ERROR       | ADC error flag. Set to true if an error is detected in the ADC core. |
    /// | 5   | CRC_ERROR       | CRC error flag. Set to true if a CRC error is detected on a register read. |
    /// | 4   | REGISTER_ERROR  | Register error flag. Set to true if a register parity error is detected. |
    /// | 3:2 | Reserved        | -                                 |
    /// | 1:0 | CHANNEL         | Current channel (see [`Channel`]).|
    ///
    /// Reset: 0x80, Access: Read-only
    StatusRegister {
        /// Data ready flag. Set to true when new conversion data is available.
        #[bits(1, default = true)] pub ready: bool,
        /// ADC error flag. Set to true if an error is detected in the ADC core.
        #[bits(1)] pub adc_error: bool,
        /// CRC error flag. Set to true if a CRC error is detected on a register read.
        #[bits(1)] pub crc_error: bool,
        /// Register error flag. Set to true if a register parity error is detected.
        #[bits(1)] pub register_error: bool,
        #[bits(2)] __: u8,
        /// Current channel. Indicates which channel's data is present (see [`Channel`]).
        #[bits(2)] pub channel: Channel,
    }, 1, 0x00);

rw_register!(
    /// ADC Mode Register (0x01)
    /// Configures the ADC's operating mode, reference, clock source, and delay settings.
    ///
    /// | Bit   | Name         | Description                                                                 |
    /// |-------|--------------|-----------------------------------------------------------------------------|
    /// | 15    | REF_EN       | Internal reference enable. Set to true to enable the internal reference.      |
    /// | 14    | HIDE_DELAY   | Hide delay bit. Set to true to hide the delay between conversions.            |
    /// | 13    | SING_CYC     | Single cycle mode. Set to true to enable single cycle settling mode.          |
    /// | 12    | Reserved     | -                                                                            |
    /// |11:8   | DELAY        | Delay setting (see [`Delay`]).                                               |
    /// | 7     | Reserved     | -                                                                            |
    /// | 6:4   | MODE         | ADC operating mode (see [`Mode`]).                                          |
    /// | 3:2   | CLKSEL       | Clock source selection (see [`ClockSource`]).                                |
    /// | 1:0   | Reserved     | -                                                                            |
    ///
    /// Reset: 0x8000, Access: Read/Write
    AdcModeRegister {
        /// Internal reference enable. Set to true to enable the internal reference.
        #[bits(1, default = true)] pub ref_enable: bool,
        /// Hide delay bit. Set to true to hide the delay between conversions.
        #[bits(1)] pub hide_delay: bool,
        /// Single cycle mode. Set to true to enable single cycle settling mode.
        #[bits(1)] pub sing_cyc: bool,
        #[bits(2)] __: u8,
        /// Delay setting. Selects the delay between conversions (see [`Delay`]).
        #[bits(3)] pub delay: Delay,
        #[bits(1)] __: u8,
        /// ADC operating mode (see [`Mode`]).
        #[bits(3)] pub mode: Mode,
        /// Clock source selection (see [`ClockSource`]).
        #[bits(2)] pub clksel: ClockSource,
        #[bits(2)] __: u8,
    }, 2, 0x01);

rw_register!(
    /// Interface Mode Register (0x02)
    /// Configures the digital interface operation, including CRC, data word length, and continuous read mode.
    ///
    /// | Bit   | Name         | Description                                                                 |
    /// |-------|--------------|-----------------------------------------------------------------------------|
    /// | 15:13 | Reserved     | -                                                                            |
    /// | 12    | ALT_SYNC     | Alternate sync pin function. Set to true to enable alternate sync.            |
    /// | 11    | IOSTRENGTH   | Output driver strength. Set to true for strong drive.                         |
    /// | 10    | Reserved     | -                                                                            |
    /// | 9     | DOUT_RESET   | Data output reset. Set to true to reset DOUT/RDY.                             |
    /// | 8     | CONTREAD     | Continuous read mode. Set to true to enable continuous read.                  |
    /// | 7     | DATA_STAT    | Data + status read. Set to true to append status byte to data.                |
    /// | 6     | REG_CHECK    | Register check enable. Set to true to enable register check.                  |
    /// | 5:4   | CRC_EN       | CRC mode (see [`Crc`]).                                                      |
    /// | 3     | Reserved     | -                                                                            |
    /// | 2     | WL16         | Data register length (see [`DataRegisterLength`]).                            |
    /// | 1:0   | Reserved     | -                                                                            |
    ///
    /// Reset: 0x0000, Access: Read/Write
    InterfaceModeRegister {
        #[bits(3)] __: u8,
        /// Alternate sync pin function. Set to true to enable alternate sync.
        #[bits(1)] pub alt_sync: bool,
        /// Output driver strength. Set to true for strong drive.
        #[bits(1)] pub iostrength: bool,
        #[bits(2)] __: u8,
        /// Data output reset. Set to true to reset DOUT/RDY.
        #[bits(1)] pub dout_reset: bool,
        /// Continuous read mode. Set to true to enable continuous read.
        #[bits(1)] pub cont_read: bool,
        /// Data + status read. Set to true to append status byte to data.
        #[bits(1)] pub data_stat: bool,
        /// Register check enable. Set to true to enable register check.
        #[bits(1)] pub reg_check: bool,
        #[bits(1)] __: u8,
        /// CRC mode (see [`Crc`]).
        #[bits(2)] pub crc_en: Crc,
        #[bits(1)] __: u8,
        /// Data register length (see [`DataRegisterLength`]).
        #[bits(1)] pub wl16: DataRegisterLength,
    }, 2, 0x02);

register!(
    /// Register Check Register (0x03)
    /// Holds the register check value for integrity verification.
    ///
    /// | Bit   | Name         | Description                                                                 |
    /// |-------|--------------|-----------------------------------------------------------------------------|
    /// | 23:0  | REG_CHECK    | Register check value.                                                       |
    ///
    /// Reset: 0x000000, Access: Read-only
    RegisterCheck {
        /// Register check value for integrity verification.
        #[bits(24)] pub reg_check: u32,
    }, 3, 0x03);

register!(
    /// Data Register (0x04)
    /// Holds the latest conversion result.
    ///
    /// | Bit   | Name         | Description                                                                 |
    /// |-------|--------------|-----------------------------------------------------------------------------|
    /// | 23:0  | DATA         | Latest conversion result.                                                    |
    ///
    /// Reset: 0x000000, Access: Read-only
    DataRegister {
        /// Latest conversion result.
        #[bits(24)] pub data: u32,
    }, 3, 0x04);

register!(
    /// Data and Status Register (0x04)
    /// Holds the latest conversion result and status byte.
    ///
    /// | Bit   | Name         | Description                                                                 |
    /// |-------|--------------|-----------------------------------------------------------------------------|
    /// | 31:8  | DATA         | Latest conversion result.                                                    |
    /// | 7:0   | STATUS       | Status byte.                                                                |
    ///
    /// Reset: 0x00000000, Access: Read-only
    DataAndStatusRegister {
        /// Latest conversion result.
        #[bits(24)] pub data: u32,
        /// Status byte.
        #[bits(8)] pub status: u8,
    }, 4, 0x04);

rw_register!(
    /// GPIO Configuration Register (0x06)
    /// Configures the GPIO and SYNC/ERROR pin functions.
    ///
    /// | Bit   | Name                | Description                                                        |
    /// |-------|---------------------|--------------------------------------------------------------------|
    /// | 15:13 | Reserved            | -                                                                  |
    /// | 12    | MUX_IO              | GPIO multiplexer enable. Set to true to enable mux.                |
    /// | 11    | SYNC_EN             | SYNC enable. Set to true to enable SYNC function.                  |
    /// | 10:9  | ERR_EN              | SYNC/ERROR pin mode (see [`SyncErrorPinMode`]).                   |
    /// | 8     | ERR_DAT             | Error data output. Set to true to output error data.               |
    /// | 7:6   | Reserved            | -                                                                  |
    /// | 5     | GPIO1_INPUT_ENABLE  | Enable GPIO1 as input. Set to true to enable.                      |
    /// | 4     | GPIO0_INPUT_ENABLE  | Enable GPIO0 as input. Set to true to enable.                      |
    /// | 3     | GPIO1_OUTPUT_ENABLE | Enable GPIO1 as output. Set to true to enable.                     |
    /// | 2     | GPIO0_OUTPUT_ENABLE | Enable GPIO0 as output. Set to true to enable.                     |
    /// | 1     | GPIO1_DATA          | GPIO1 data value. Set to true for high.                            |
    /// | 0     | GPIO0_DATA          | GPIO0 data value. Set to true for high.                            |
    ///
    /// Reset: 0x1800, Access: Read/Write
    GPIOConfigRegister {
        #[bits(3)] __: u8,
        /// GPIO multiplexer enable. Set to true to enable mux.
        #[bits(1)] pub mux_io: bool,
        /// SYNC enable. Set to true to enable SYNC function.
        #[bits(1, default = true)] pub sync_en: bool,
        /// SYNC/ERROR pin mode (see [`SyncErrorPinMode`]).
        #[bits(2)] pub err_en: SyncErrorPinMode,
        /// Error data output. Set to true to output error data.
        #[bits(1)] pub err_dat: bool,
        #[bits(2)] __: u8,
        /// Enable GPIO1 as input. Set to true to enable.
        #[bits(1)] pub gpio1_input_enable: bool,
        /// Enable GPIO0 as input. Set to true to enable.
        #[bits(1)] pub gpio0_input_enable: bool,
        /// Enable GPIO1 as output. Set to true to enable.
        #[bits(1)] pub gpio1_output_enable: bool,
        /// Enable GPIO0 as output. Set to true to enable.
        #[bits(1)] pub gpio0_output_enable: bool,
        /// GPIO1 data value. Set to true for high.
        #[bits(1)] pub gpio1_data: bool,
        /// GPIO0 data value. Set to true for high.
        #[bits(1)] pub gpio0_data: bool,
    }, 2, 0x06);

register!(
    /// ID Register (0x07)
    /// Contains the fixed device ID value for the AD7175-2.
    ///
    /// | Bit   | Name | Description                |
    /// |-------|------|----------------------------|
    /// | 15:0  | ID   | Device ID (fixed value).   |
    ///
    /// Reset: 0x0cd0, Access: Read-only
    IdRegister {
        /// Device ID (fixed value for AD7175-2).
        #[bits(16, default = 0x0cd0)] pub id: u16
    }, 2, 0x07);

multi_rw_register! {
    {
        /// Channel Registers (0x10..0x13)
        /// Configure channel enable, setup selection, and input mux for each channel.
        ///
        /// | Bit   | Name      | Description                                      |
        /// |-------|-----------|--------------------------------------------------|
        /// | 15    | CH_EN     | Channel enable. Set to true to enable channel.   |
        /// | 14    | Reserved  | -                                                |
        /// | 13:12 | SETUP_SEL | Setup selection (see [`Setup`]).                 |
        /// | 11:10 | Reserved  | -                                                |
        /// | 9:5   | AINPOS    | Positive input selection (see [`Input`]).        |
        /// | 4:0   | AINNEG    | Negative input selection (see [`Input`]).        |
        /// |
        /// Reset: 0x8000, Access: Read/Write
        pub struct ChannelRegister {
            #[bits(1, default = true)] pub ch_en: bool,
            #[bits(1)] __: u8,
            #[bits(2)] pub setup_sel: Setup,
            #[bits(2)] __: u8,
            #[bits(5)] pub ainpos: Input,
            #[bits(5)] pub ainneg: Input,
        }
    },
    2,
    (Channel0Register, 0x10),
    (Channel1Register, 0x11),
    (Channel2Register, 0x12),
    (Channel3Register, 0x13)
}

multi_rw_register! {
    {
        /// Setup Configuration Registers (0x20..0x23)
        /// Configure input/output coding, reference, and buffer settings for each setup.
        ///
        /// | Bit   | Name                | Description                                         |
        /// |-------|---------------------|-----------------------------------------------------|
        /// | 15:13 | Reserved            | -                                                   |
        /// | 12    | BI_UNIPOLAR         | Output coding (see [`OutputCoding`]).               |
        /// | 11    | REFBUF_POS_ENABLED  | Enable positive reference buffer. Set to true to enable. |
        /// | 10    | REFBUF_NEG_ENABLED  | Enable negative reference buffer. Set to true to enable. |
        /// | 9     | AINBUF_POS_ENABLED  | Enable positive input buffer. Set to true to enable.    |
        /// | 8     | AINBUF_NEG_ENABLED  | Enable negative input buffer. Set to true to enable.    |
        /// | 7     | BURNOUT_EN          | Burnout current enable. Set to true to enable.          |
        /// | 6     | Reserved            | -                                                   |
        /// | 5:4   | REF_SEL             | Reference selection (see [`ReferenceSource`]).      |
        /// | 3:0   | Reserved            | -                                                   |
        /// |
        /// Reset: 0x0580, Access: Read/Write
        pub struct SetupConfigRegister {
            #[bits(3)] __: u16,
            /// Output coding (see [`OutputCoding`]).
            #[bits(1, default = OutputCoding::Bipolar)] pub bi_unipolar: OutputCoding,
            /// Enable positive reference buffer. Set to true to enable.
            #[bits(1)] pub refbuf_pos_enabled: bool,
            /// Enable negative reference buffer. Set to true to enable.
            #[bits(1)] pub refbuf_neg_enabled: bool,
            /// Enable positive input buffer. Set to true to enable.
            #[bits(1, default = true)] pub ainbuf_pos_enabled: bool,
            /// Enable negative input buffer. Set to true to enable.
            #[bits(1, default = true)] pub ainbuf_neg_enabled: bool,
            /// Burnout current enable. Set to true to enable.
            #[bits(1)] pub burnout_en: bool,
            #[bits(1)] __: u8,
            /// Reference selection (see [`ReferenceSource`]).
            #[bits(2, default = ReferenceSource::Internal)] pub ref_sel: ReferenceSource,
            #[bits(4)] __: u8,
        }
    },
    2,
    (SetupConfig0Register, 0x20),
    (SetupConfig1Register, 0x21),
    (SetupConfig2Register, 0x22),
    (SetupConfig3Register, 0x23)
}

// Filter Configuration Registers (0x28..0x2B)
multi_rw_register! {
    {
        /// Filter Configuration Registers (0x28..0x2B)
        /// Configure digital filter type, enhanced filter, order, and output data rate for each setup.
        ///
        /// | Bit   | Name        | Description                                         |
        /// |-------|-------------|-----------------------------------------------------|
        /// | 15    | SINC3_MAP   | SINC3 direct map enable. Set to true for direct map.|
        /// | 14:12 | Reserved    | -                                                   |
        /// | 11    | ENHFILTEN   | Enhanced filter enable. Set to true to enable.      |
        /// | 10:8  | ENHFILT     | Enhanced filter rate (see [`EnhancedFilterRate`]).  |"]
        /// | 7     | Reserved    | -                                                   |
        /// | 6:5   | ORDER       | Filter order (see [`FilterOrder`]).                 |"]
        /// | 4:0   | ODR         | Output data rate (see [`OutputDataRate`]).         |"]
        /// |
        /// Reset: 0x0500, Access: Read/Write
        pub struct FilterConfigRegister {
            #[bits(1, access = RO, default = false)] pub sinc3_map: bool,
            #[bits(3)] __: u8,
            #[bits(1)] pub enhfilten: bool,
            #[bits(3, default = EnhancedFilterRate::Sps20)] pub enhfilt: EnhancedFilterRate,
            #[bits(1)] __: u8,
            #[bits(2)] pub order: FilterOrder,
            #[bits(5)] pub odr: OutputDataRate,
        }
    },
    2,
    (DefaultFilterConfig0Register, 0x28),
    (DefaultFilterConfig1Register, 0x29),
    (DefaultFilterConfig2Register, 0x2a),
    (DefaultFilterConfig3Register, 0x2b)
}

// Direct SINC3 Map Filter Configuration Registers (0x28..0x2B)
multi_rw_register! {
    {
        /// Direct SINC3 Map Filter Configuration Registers (0x28..0x2B)
        /// Configure direct SINC3 decimation rate for each setup.
        ///
        /// | Bit   | Name            | Description                                         |
        /// |-------|-----------------|-----------------------------------------------------|
        /// | 15    | SINC3_MAP       | SINC3 direct map enable. Set to true for direct map.|
        /// | 14:0  | DECIMATION_RATE | SINC3 decimation rate value.                        |
        /// |
        /// Reset: 0x8000, Access: Read/Write
        pub struct DirectSinc3MapFilterConfigRegister {
            #[bits(1, access = RO, default = true)] pub sinc3_map: bool,
            #[bits(15)] pub decimation_rate: u16,
        }
    },
    2,
    (DirectSinc3MapFilterConfig0Register, 0x28),
    (DirectSinc3MapFilterConfig1Register, 0x29),
    (DirectSinc3MapFilterConfig2Register, 0x2a),
    (DirectSinc3MapFilterConfig3Register, 0x2b)
}

pub struct FilterConfig0Register;

impl FilterConfig0Register {
    pub fn new_default() -> DefaultFilterConfig0Register {
        DefaultFilterConfig0Register::new()
    }
    
    pub fn new_direct_sinc3_map() -> DirectSinc3MapFilterConfig0Register {
        DirectSinc3MapFilterConfig0Register::new()
    }
}

pub struct FilterConfig1Register;

impl FilterConfig1Register {
    pub fn new_default() -> DefaultFilterConfig1Register {
        DefaultFilterConfig1Register::new()
    }
    
    pub fn new_direct_sinc3_map() -> DirectSinc3MapFilterConfig1Register {
        DirectSinc3MapFilterConfig1Register::new()
    }
}

pub struct FilterConfig2Register;

impl FilterConfig2Register {
    pub fn new_default() -> DefaultFilterConfig2Register {
        DefaultFilterConfig2Register::new()
    }
    
    pub fn new_direct_sinc3_map() -> DirectSinc3MapFilterConfig2Register {
        DirectSinc3MapFilterConfig2Register::new()
    }
}

pub struct FilterConfig3Register;

impl FilterConfig3Register {
    pub fn new_default() -> DefaultFilterConfig3Register {
        DefaultFilterConfig3Register::new()
    }
    
    pub fn new_direct_sinc3_map() -> DirectSinc3MapFilterConfig3Register {
        DirectSinc3MapFilterConfig3Register::new()
    }
}

// Offset Registers (0x30..0x33)
multi_rw_register! {
    {
        /// Offset Registers (0x30..0x33)
        /// Store offset calibration coefficients for each setup.
        ///
        /// | Bit   | Name   | Description                                 |
        /// |-------|--------|---------------------------------------------|
        /// | 23:0  | OFFSET | Offset calibration coefficient for the setup.|
        /// |
        /// Reset: 0x800000, Access: Read/Write
        pub struct OffsetRegister {
            #[bits(24, default = 0x800000)] pub offset: u32,
        }
    },
    3,
    (Offset0Register, 0x30),
    (Offset1Register, 0x31),
    (Offset2Register, 0x32),
    (Offset3Register, 0x33)
}

// Gain Registers (0x38..0x3B)
multi_rw_register! {
    {
        /// Gain Registers (0x38..0x3B)
        /// Store gain calibration coefficients for each setup.
        ///
        /// | Bit   | Name | Description                               |
        /// |-------|------|-------------------------------------------|
        /// | 23:0  | GAIN | Gain calibration coefficient for the setup.|
        /// |
        /// Reset: 0x5XXXX0, Access: Read/Write
        pub struct GainRegister {
            #[bits(24, default = 0x500000)] pub gain: u32,
        }
    },
    3,
    (Gain0Register, 0x38),
    (Gain1Register, 0x39),
    (Gain2Register, 0x3a),
    (Gain3Register, 0x3b)
}
