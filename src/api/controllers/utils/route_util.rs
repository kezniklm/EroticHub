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
