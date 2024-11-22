pub mod proto {
    include!(concat!(env!("OUT_DIR"), "/krec.proto.rs"));
}

pub use proto::{
    ActuatorCommand, ActuatorConfig, ActuatorState, ImuQuaternion, ImuValues, KRecFrame, KRecHeader,
};
