//! A small source that needs neither network nor storage permissions.

use chilpa_plugin_sdk::api::{Browse, Entry, EntryKind, Listing, Query, RemoteTrack, Results};
use chilpa_plugin_sdk::plugin;

const ID: &str = "hello:one";

pub fn search(query: Query) -> Result<Results, String> {
    let tracks = if "Hello".to_lowercase().contains(&query.text.to_lowercase()) && query.limit > 0 {
        vec![RemoteTrack {
            external_id: ID.into(),
            title: "Hello".into(),
            artist: "Example".into(),
            ..RemoteTrack::default()
        }]
    } else {
        Vec::new()
    };
    Ok(Results {
        tracks,
        ..Results::default()
    })
}

pub fn browse(ask: Browse) -> Result<Listing, String> {
    if !ask.at.is_empty() {
        return Err(format!("unknown location: {}", ask.at));
    }
    Ok(Listing {
        title: "Hello Source".into(),
        entries: vec![Entry {
            id: ID.into(),
            title: "Hello".into(),
            kind: EntryKind::Item,
            playable: false,
            why_not: "This example has no audio stream".into(),
            ..Entry::default()
        }],
        ..Listing::default()
    })
}

plugin! {
    api_version: 1,
    chilpa_search => search,
    chilpa_browse => browse,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_item_keeps_one_identity_from_listing_to_search() {
        let listing = browse(Browse::root()).unwrap();
        let id = listing.entries[0].id.clone();
        assert_eq!(
            id,
            search(Query::new("hello", 5)).unwrap().tracks[0].external_id
        );
        assert!(!listing.entries[0].playable);
        assert!(browse(Browse::at("other")).is_err());
    }
}
