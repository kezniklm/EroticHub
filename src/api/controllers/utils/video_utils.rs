use crate::api::controllers::utils::route_util::build_get_video_path;
use crate::business::facades::artist::{ArtistFacade, ArtistFacadeTrait};
use crate::business::models::video::VideoList;
use crate::business::Result;
use crate::persistence::entities::video::Video;
use actix_web::web::Data;
use anyhow::Error;

pub fn parse_option_string(input: Option<String>) -> Result<Option<Vec<i32>>, Error> {
    if let Some(s) = input {
        if s.is_empty() {
            return Ok(None);
        }
        let result: Vec<i32> = s
            .split(',')
            .map(|item| item.trim().parse::<i32>())
            .collect::<Result<Vec<i32>, _>>()?;

        Ok(Some(result))
    } else {
        Ok(None)
    }
}

pub async fn from_video_to_video_list(
    videos: Vec<Video>,
    artist_facade: Data<ArtistFacade>,
) -> Result<Vec<VideoList>> {
    let mut artists_ids = Vec::new();
    videos.iter().for_each(|v| {
        artists_ids.push(v.artist_id);
    });

    let artists = artist_facade.get_artists_names_by_id(artists_ids).await?;

    let mut serialized_videos = Vec::new();
    for video in &videos {
        for artist in &artists {
            if video.artist_id == artist.id {
                let (_video_path, thumbnail_path) = build_get_video_path(video.id);
                serialized_videos.push(VideoList {
                    id: video.id,
                    artist_id: video.artist_id,
                    artist_name: artist.name.clone(),
                    thumbnail_path,
                    name: video.name.clone(),
                })
            }
        }
    }

    Ok(serialized_videos)
}
