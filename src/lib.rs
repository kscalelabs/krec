use color_eyre::Result;

// Initialize tracing and color-eyre
pub fn init() -> Result<()> {
    // Install color-eyre panic and error hooks
    color_eyre::install()?;

    // Initialize tracing subscriber with pretty console output
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    Ok(())
}

mod ffmpeg;
mod krec;
mod proto;

pub use ffmpeg::{combine_with_video, extract_from_video};
pub use krec::KRec;
pub use proto::{
    proto::Vec3, ActuatorCommand, ActuatorConfig, ActuatorState, ImuQuaternion, ImuValues,
    KRecFrame, KRecHeader,
};
