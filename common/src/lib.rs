#![no_std]

use postcard::flavors::{Cobs, Slice};
use serde::{Serialize, Deserialize};
use crate::controller::VelocityData;
use crate::crc::Crc;

pub mod controller;
pub mod crc;

// other vals can have less error?
pub const BAUD_RATE_CTRL : u32 = 1000000;//1000000;//921600;//460800;//115200;
pub const BAUD_RATE_FORWARD: u32 = 1000000;
pub const BAUD_RATE_SABERTOOTH : u32 = 38400;
pub const BAUD_RATE_NANO : u32 = 57600;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum CommunicationError {
    BadData,
    BadCheckSum(u16, u16),
    EOF,
    BufferFull,
    InternalError
}

impl From<postcard::Error> for CommunicationError {
    fn from(error: postcard::Error) -> Self {
        use postcard::Error::*;

        match error {
            DeserializeBadVarint | DeserializeBadBool | DeserializeBadChar | DeserializeBadUtf8 | DeserializeBadOption | DeserializeBadEnum | DeserializeBadEncoding => {
                CommunicationError::BadData
            }
            DeserializeUnexpectedEnd => {
                CommunicationError::EOF
            }
            SerializeBufferFull => {
                CommunicationError::BufferFull
            }
            _ => {
                CommunicationError::InternalError
            }
        }
    }
}

#[cfg(test)]
mod test {
    use core::mem::MaybeUninit;
    use crate::controller::{DownstreamMessage, VelocityData};
    use crate::{read, write};
    use crate::clamp_map_val;

    #[test]
    fn test_communication() {
        let mut buffer : [u8; 200] = unsafe { MaybeUninit::uninit().assume_init() };

        let command = DownstreamMessage::VelocityUpdate(VelocityData {
            forwards_left: 4.0,
            forwards_right: 3.0,
            strafing: 2.0,
            vertical: 1.0
        });

        let buffer2 = write(&command, &mut buffer).unwrap();
        let received = read::<DownstreamMessage>(buffer2).unwrap(); //

        match received {
            DownstreamMessage::VelocityUpdate(data) => {
                assert_eq!(data.forwards_left, 4.0);
                assert_eq!(data.forwards_right, 3.0);
                assert_eq!(data.strafing, 2.0);
                assert_eq!(data.vertical, 1.0);
            }
            _ => { panic!() }
        }
    }

    #[test]
    fn test_wrap_val() {
        let cases = [
            (-1.1, -1.0),
            (-1.0, -1.0),
            (-0.95, -1.0),
            (-0.5, -0.5),
            (-0.05, 0.0),
            (0.0, 0.0),
            (0.05, 0.0),
            (0.5, 0.5),
            (0.95, 1.0),
            (1.0, 1.0),
            (1.1, 1.0),
        ];

        for (test, expected) in cases {
            let val = clamp_map_val(test, 0.05, 0.95);
            assert_eq!(val, expected);
        }
    }
}

pub fn write<'a, S: Serialize>(obj: &S, out: &'a mut [u8]) -> Result<&'a mut [u8], CommunicationError> {
    postcard::serialize_with_flavor(obj, Crc::new(Cobs::try_new(Slice::new(out)).map_err(CommunicationError::from)?)).map_err(CommunicationError::from)
}

pub fn read<'a, D: Deserialize<'a>>(buffer: &'a mut [u8]) -> Result<D, CommunicationError> {
    let read = postcard_cobs::decode_in_place(buffer).map_err(|_| CommunicationError::BadData)?;
    if read > 3 {
        let data = &buffer[..read - 3];
        let crc = u16::from_le_bytes((&buffer[read - 3..read - 1]).try_into().unwrap());

        let checksum = crate::crc::CRC.checksum(data);
        if checksum == crc {
            postcard::from_bytes(data).map_err(CommunicationError::from)
        } else {
            Err(CommunicationError::BadCheckSum(checksum, crc))
        }
    } else {
        Err(CommunicationError::EOF)
    }
}

pub fn end_of_frame(byte: &u8) -> bool {
    *byte == 0x00
}

pub fn joystick_math(lx: f32, ly: f32, rx: f32, ry: f32) -> VelocityData {
    let forwards_left = ly + lx;
    let forwards_right = ly - lx;
    let strafing = rx;
    let vertical = ry;

    VelocityData {
        forwards_left,
        forwards_right,
        strafing,
        vertical
    }.clamp()
}

pub fn clamp_map_val(val: f32, min: f32, max: f32) -> f32 {
    let v = abs(val).clamp(min, max);
    let v = (v - min) / (max - min);
    copysign(v, val)
}

pub fn abs(val: f32) -> f32 {
    if val >= 0.0 { val } else { -val }
}

pub fn copysign(val: f32, sign: f32) -> f32 {
    if sign >= 0.0 { abs(val) } else { -abs(val) }
}
