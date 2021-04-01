use thiserror::Error;
#[derive(Error, Debug)]
pub enum GranularError {
    #[error("Unable to load the sample from file: `{0}`")]
    FailedToLoadSampleFile(String),
}
