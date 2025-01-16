use actix_web::http::header::{HeaderName, HeaderValue};
use actix_web::HttpResponse;
use std::str::FromStr;

pub fn build_get_temp_path(file_id: i32) -> String {
    format!("/temp/{file_id}")
}

pub fn build_get_video_path(video_id: i32) -> (String, String) {
    (
        format!("/video/{video_id}"),
        format!("/thumbnail/{video_id}"),
    )
}

pub fn build_watch_path(video_id: i32) -> String {
    format!("/video/{}/watch", video_id)
}
pub fn build_stream_watch_path(stream_id: i32) -> String {
    format!("/stream/{stream_id}/watch")
}

pub fn add_redirect_header(path: &str, response: &mut HttpResponse) -> actix_web::Result<()> {
    response.head_mut().headers.append(
        HeaderName::from_str("HX-Redirect").unwrap(),
        HeaderValue::from_str(path)?,
    );

    Ok(())
}
