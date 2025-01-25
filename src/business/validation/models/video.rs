use crate::business::models::video::{VideoEditReq, VideoUploadReq};
use crate::business::validation::contexts::video::PatchVideoValidationContext;
use crate::business::validation::utils::extract_first_error;
use crate::business::validation::validatable::{EmptyContext, Validatable};
use validator::{Validate, ValidationError};

impl Validatable<EmptyContext> for VideoUploadReq {
    async fn validate_model(&self, _context: &EmptyContext) -> Result<(), ValidationError> {
        if let Err(validation_errors) = self.validate() {
            if let Some(first_error) = extract_first_error(&validation_errors) {
                return Err(first_error);
            }
        }

        Ok(())
    }
}

impl Validatable<PatchVideoValidationContext> for VideoEditReq {
    async fn validate_model(
        &self,
        context: &PatchVideoValidationContext,
    ) -> Result<(), ValidationError> {
        if let Err(validation_errors) = self.validate() {
            if let Some(first_error) = extract_first_error(&validation_errors) {
                return Err(first_error);
            }
        }

        if let Ok(artist) = context
            .artist_facade
            .get_artist_internal(context.user_id, None)
            .await
        {
            let video_artist_id = context
                .pg_video_repo
                .get_video_artist_id(context.video_id, None)
                .await
                .map_err(ValidationError::from)?;
            if artist.id == video_artist_id {
                return Ok(());
            }
            return Err(ValidationError::new("Artist doesn't match"));
        }

        Err(ValidationError::new("Validation failed"))
    }
}
