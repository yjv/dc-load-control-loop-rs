use core::ops::{Deref, DerefMut};
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
    ($name:ident { $($field:tt)* }, 1, $id:expr) => {
        #[bitfield_struct::bitfield(u8, repr = [u8; 1], from = u8::to_ne_bytes, into = u8::from_ne_bytes, defmt = true, order = msb)]
        pub struct $name {
            $($field)*
        }

        impl Register<1> for $name {
            fn get_id() -> u8 { $id }
            fn from_buffer(raw: &[u8; 1]) -> Self { Self::from_bits(*raw) }
        }
    };
    ($name:ident { $($field:tt)* }, 2, $id:expr) => {
        #[bitfield_struct::bitfield(u16, repr = [u8; 2], from = u16::to_ne_bytes, into = u16::from_ne_bytes, defmt = true, order = msb)]
        pub struct $name {
            $($field)*
        }

        impl Register<2> for $name {
            fn get_id() -> u8 { $id }
            fn from_buffer(raw: &[u8; 2]) -> Self { Self::from_bits(*raw) }
        }
    };
    ($name:ident { $($field:tt)* }, 3, $id:expr) => {
        #[bitfield_struct::bitfield(u32, repr = [u8; 3], from = from_u32, into = into_u32, defmt = true, order = msb)]
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
    ($name:ident { $($field:tt)* }, 4, $id:expr) => {
        #[bitfield_struct::bitfield(u32, repr = [u8; 4], from = u32::to_ne_bytes, into = u32::from_ne_bytes, defmt = true, order = msb)]
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
    ($fields:tt, 1, $(($name:ident, $id:expr)),+ $(,)?) => {
        $(
            register!($name $fields, 1, $id);
        )+
    };
    ($fields:tt, 2, $(($name:ident, $id:expr)),+ $(,)?) => {
        $(
            register!($name $fields, 2, $id);
        )+
    };
    ($fields:tt, 3, $(($name:ident, $id:expr)),+ $(,)?) => {
        $(
            register!($name $fields, 3, $id);
        )+
    };
    ($fields:tt, 4, $(($name:ident, $id:expr)),+ $(,)?) => {
        $(
            register!($name $fields, 4, $id);
        )+
    };
}

macro_rules! rw_register {
    ($name:ident { $($field:tt)* }, 1, $id:expr) => {
        register!($name { $($field)* }, 1, $id);

        impl WritableRegister<1> for $name   {
            fn to_buffer(&self) -> [u8; 1] { self.into_bits() }
        }
    };
    ($name:ident { $($field:tt)* }, 2, $id:expr) => {
        register!($name { $($field)* }, 2, $id);

        impl WritableRegister<2> for $name   {
            fn to_buffer(&self) -> [u8; 2] { self.into_bits() }
        }
    };
    ($name:ident { $($field:tt)* }, 3, $id:expr) => {
        register!($name { $($field)* }, 3, $id);

        impl WritableRegister<3> for $name   {
            fn to_buffer(&self) -> [u8; 3] {
                self.into_bits()
            }
        }
    };
    ($name:ident { $($field:tt)* }, 4, $id:expr) => {
        register!($name { $($field)* }, 4, $id);

        impl WritableRegister<4> for $name   {
            fn to_buffer(&self) -> [u8; 4] {
                self.into_bits()
            }
        }
    };
    ($fields:tt, 1, $(($name:ident, $id:expr)),+ $(,)?) => {
        $(
            rw_register!($name $fields, 1, $id);
        )+
    };
    ($fields:tt, 2, $(($name:ident, $id:expr)),+ $(,)?) => {
        $(
            rw_register!($name $fields, 2, $id);
        )+
    };
    ($fields:tt, 3, $(($name:ident, $id:expr)),+ $(,)?) => {
        $(
            rw_register!($name $fields, 3, $id);
        )+
    };
    ($fields:tt, 4, $(($name:ident, $id:expr)),+ $(,)?) => {
        $(
            rw_register!($name $fields, 4, $id);
        )+
    };
}

register!(StatusRegister {
    #[bits(1, default = true)] pub ready: bool,           // 7
    #[bits(1)] pub adc_error: bool,       // 6
    #[bits(1)] pub crc_error: bool,       // 5
    #[bits(1)] pub register_error: bool,  // 4
    #[bits(2)] __: u8,                     // 3:2 reserved
    #[bits(2)] pub channel: Channel,           // 1:0
}, 1, 0x00);

rw_register!(AdcModeRegister {
    #[bits(1, default = true)] pub ref_enable: bool,       // 15
    #[bits(1)] pub hide_delay: bool,   // 14
    #[bits(1)] pub sing_cyc: bool,     // 13
    #[bits(2)] __: u8,                  // 12 reserved
    #[bits(3)] pub delay: Delay,          // 11:8
    #[bits(1)] __: u8,                  // 7 reserved
    #[bits(3)] pub mode: Mode,           // 6:4
    #[bits(2)] pub clksel: ClockSource,         // 3:2
    #[bits(2)] __: u8,                  // 1:0 reserved
}, 2, 0x01);

rw_register!(InterfaceModeRegister {
    #[bits(3)] __: u8,                     // 15:13 reserved
    #[bits(1)] pub alt_sync: bool,        // 12
    #[bits(1)] pub iostrength: bool,      // 11
    #[bits(2)] __: u8,                     // 10 reserved
    #[bits(1)] pub dout_reset: bool,      // 9
    #[bits(1)] pub cont_read: bool,       // 8
    #[bits(1)] pub data_stat: bool,       // 7
    #[bits(1)] pub reg_check: bool,       // 6
    #[bits(1)] __: u8,                     // 3 reserved
    #[bits(2)] pub crc_en: Crc,            // 5:4
    #[bits(1)] __: u8,                     // 1:0 reserved
    #[bits(1)] pub wl16: DataRegisterLength,            // 2
}, 2, 0x02);

register!(RegisterCheck {
    #[bits(24)] pub reg_check: u32,
}, 3, 0x03);

register!(DataRegister {
    #[bits(24)] pub data: u32,
}, 3, 0x04);

register!(DataAndStatusRegister {
    #[bits(24)] pub data: u32,
    #[bits(8)] pub status: u8,
}, 4, 0x04);

rw_register!(GPIOConfigRegister {
    #[bits(3)] __: u8,                 // 15:13 reserved
    #[bits(1)] pub mux_io: bool,      // 12
    #[bits(1, default = true)] pub sync_en: bool,     // 11
    #[bits(2)] pub err_en: SyncErrorPinMode,      // 10:9
    #[bits(1)] pub err_dat: bool,     // 8
    #[bits(2)] __: u8,                 // 7:6 reserved
    #[bits(1)] pub gpio1_input_enable: bool,         // 5
    #[bits(1)] pub gpio0_input_enable: bool,         // 4
    #[bits(1)] pub gpio1_output_enable: bool,         // 3
    #[bits(1)] pub gpio0_output_enable: bool,         // 2
    #[bits(1)] pub gpio1_data: bool,                 // 1
    #[bits(1)] pub gpio0_data: bool,                 // 0
}, 2, 0x06);

// ID register (fixed value for part ID, usually read only)
register!(IdRegister {
    #[bits(16, default = 0x0cd0)] pub id: u16
}, 2, 0x07);

// Channel Registers: 0x10..0x13 (x4)
// One example; you can alias/update address for each channel as needed
rw_register!({
    #[bits(1, default = true)] pub ch_en: bool,          // 15
    #[bits(1)] __: u8,                    // 14 reserved
    #[bits(2)] pub setup_sel: Setup,        // 13:12
    #[bits(2)] __: u8,                    // 11:10 reserved
    #[bits(5)] pub ainpos: Input,           // 9:5
    #[bits(5)] pub ainneg: Input,           // 4:0
}, 2,
    (Channel0Register, 0x10), 
    (Channel1Register, 0x11), 
    (Channel2Register, 0x12), 
    (Channel3Register, 0x13)
);

// Setup Configuration Registers: 0x20..0x23 (x4)
rw_register!({
    #[bits(3)] __: u16,                    // 15:13 reserved
    #[bits(1, default = OutputCoding::Bipolar)] pub bi_unipolar: OutputCoding,    // 12
    #[bits(1)] pub refbuf_pos_enabled: bool,     // 11
    #[bits(1)] pub refbuf_neg_enabled: bool,     // 10
    #[bits(1, default = true)] pub ainbuf_pos_enabled: bool,     // 9
    #[bits(1, default = true)] pub ainbuf_neg_enabled: bool,     // 8
    #[bits(1)] pub burnout_en: bool,     // 7
    #[bits(1)] __: u8,                    // 6 reserved 
    #[bits(2, default = ReferenceSource::Internal)] pub ref_sel: ReferenceSource,          // 5:4
    #[bits(4)] __: u8,                    // 3:0 reserved
}, 2,
    (SetupConfig0Register, 0x20), 
    (SetupConfig1Register, 0x21), 
    (SetupConfig2Register, 0x22), 
    (SetupConfig3Register, 0x23)
);

// Filter Configuration Registers: 0x28..0x2B (x4)
rw_register!({
    #[bits(1, access = RO, default = false)] pub sinc3_map: bool,      // 15
    #[bits(3)] __: u8,                    // 14:12 reserved
    #[bits(1)] pub enhfilten: bool,      // 11
    #[bits(3, default = EnhancedFilterRate::Sps20)] pub enhfilt: EnhancedFilterRate,          // 10:8
    #[bits(1)] __: u8,                    // 7 reserved
    #[bits(2)] pub order: FilterOrder,            // 6:5
    #[bits(5)] pub odr: OutputDataRate,              // 4:0 (needs bits; check datasheet for split)
}, 2,
    (DefaultFilterConfig0Register, 0x28), 
    (DefaultFilterConfig1Register, 0x29), 
    (DefaultFilterConfig2Register, 0x2a), 
    (DefaultFilterConfig3Register, 0x2b)
);

// Filter Configuration Registers: 0x28..0x2B (x4)
rw_register!({
    #[bits(1, access = RO, default = true)] pub sinc3_map: bool,      // 15
    #[bits(15)] pub decimation_rate: u16,              // 4:0 (needs bits; check datasheet for split)
}, 2,
    (DirectSinc3MapFilterConfig0Register, 0x28), 
    (DirectSinc3MapFilterConfig1Register, 0x29), 
    (DirectSinc3MapFilterConfig2Register, 0x2a), 
    (DirectSinc3MapFilterConfig3Register, 0x2b)
);

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

// Offset Registers: 0x30..0x33 (x4)
register!({
    #[bits(24, default = 0x800000)] pub offset: u32,         // 23:0
}, 3, 
    (Offset0Register, 0x30),
    (Offset1Register, 0x31),
    (Offset2Register, 0x32), 
    (Offset3Register, 0x33)
);

// Gain Registers: 0x38..0x3B (x4)
register!({
    #[bits(24, default = 0x500000)] pub gain: u32,           // 23:0
}, 3, 
    (Gain0Register, 0x38),
    (Gain1Register, 0x39),
    (Gain2Register, 0x3a), 
    (Gain3Register, 0x3b)
);
