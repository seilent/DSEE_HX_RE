use thiserror::Error;

#[derive(Debug, Error)]
pub enum DseeError {
    #[error("{0}")]
    Msg(String),
    #[error("IO: {0}")]
    Io(#[from] std::io::Error),
    #[error("open_source failed: rc={0}")]
    OpenSource(i32),
    #[error("DSEE HX inactive for these parameters")]
    Inactive,
    #[error("unsupported bit depth: {0}")]
    Bits(String),
    #[error("ffmpeg/ffprobe: {0}")]
    Ffmpeg(String),
}

pub type Result<T> = std::result::Result<T, DseeError>;
