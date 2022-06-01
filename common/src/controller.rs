use serde::{Serialize, Deserialize};
use crate::CommunicationError;

#[derive(Serialize, Deserialize, Debug)]
pub enum DownstreamMessage {
    VelocityDataMessage(VelocityData),
    Msg
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct VelocityData {
    pub forwards_left: f32,
    pub forwards_right: f32,
    pub strafing: f32,
    pub vertical: f32,
}

impl VelocityData {
    pub fn clamp(&self) -> VelocityData {
        fn clamp(num: f32) -> f32 { num.clamp(-1.0, 1.0) }

        VelocityData {
            forwards_left: clamp(self.forwards_left),
            forwards_right: clamp(self.forwards_right),
            strafing: clamp(self.strafing),
            vertical: clamp(self.vertical)
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub enum UpstreamMessage<'a> {
    Init,
    IMUStream(&'a [u8]),
    Log(&'a str),
    Panic,
    Ack,
    BadO,
    BadP(CommunicationError)
}