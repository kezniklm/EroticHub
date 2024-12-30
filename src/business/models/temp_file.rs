use serde::Deserialize;

#[derive(Deserialize)]
pub struct GetFileInputTemplate {
    pub input_type: TempFileInput,
}

#[derive(Deserialize)]
pub enum TempFileInput {
    Video,
    Thumbnail,
}
