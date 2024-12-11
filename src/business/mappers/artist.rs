use crate::business::models::artist_detail::ArtistDetail;
use crate::persistence::entities::artist::Artist;

impl From<Artist> for ArtistDetail {
    fn from(artist: Artist) -> Self {
        ArtistDetail {
            id: artist.id,
            user_id: artist.user_id,
            description: artist.description.unwrap_or("".to_string()),
        }
    }
}

// impl From<ArtistDetail> for Artist {
//     fn from(artist_detail: ArtistDetail) -> Self {
//         Artist {
//             id: artist_detail.id,
//             user_id: artist_detail.user_id,
//             description: artist_detail.description,
//         }
//     }
// }
