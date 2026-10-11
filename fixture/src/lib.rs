//! A plugin with fixed answers.
//!
//! It exists so a host can be tested against a module compiled from Rust
//! with the SDK, not one written by hand in wasm text. Nothing here reaches
//! a real service: the one request it makes goes to `fixture.test`, which
//! the test host answers. A test that wants what it resolves to really be
//! opened writes another address under `base` in the plugin's store.
//!
//! The tree it lets itself be browsed as:
//!
//! ```text
//! (root)
//!   search           a place to type
//!   genres/          a folder
//!     genre:jazz/    a folder with two live items
//!   long/            three pages of two items, by cursor
//!   show/            a folder of two finite items with a length and a date
//!   live:one         a live item
//!   file:one         a finite item
//!   hls:one          an item that says it cannot be played
//! hostile:loop       a `next` that never changes
//! hostile:blank      entries with no id or no title
//! hostile:huge       far more entries than fit
//! ```

use chilpa_plugin_sdk::api::{
    Browse, Column, Delivery, Entry, Fact, EntryKind, Listing, Quality, Query, RemoteTrack, Resolve, Results,
    Stream,
};
use chilpa_plugin_sdk::{host, plugin};

/// Where the pretend service lives. `fixture.test` unless the store says
/// otherwise.
fn base() -> String {
    host::store_get("base")
        .and_then(|bytes| String::from_utf8(bytes).ok())
        .map(|base| base.trim().trim_end_matches('/').to_string())
        .filter(|base| !base.is_empty())
        .unwrap_or_else(|| "https://fixture.test".to_string())
}

fn track(id: &str, title: &str) -> RemoteTrack {
    RemoteTrack {
        external_id: id.to_string(),
        title: title.to_string(),
        artist: "Fixture".to_string(),
        duration_ms: 180_000,
        ..RemoteTrack::default()
    }
}

pub fn search(query: Query) -> Result<Results, String> {
    let text = query.text.to_lowercase();
    let tracks = [track("file:one", "First Song"), track("file:two", "Second Song")]
        .into_iter()
        .filter(|t| t.title.to_lowercase().contains(&text))
        .take(query.limit)
        .collect();
    Ok(Results { tracks, ..Results::default() })
}

fn item(id: &str, title: &str, live: bool) -> Entry {
    Entry {
        id: id.to_string(),
        title: title.to_string(),
        kind: EntryKind::Item,
        live,
        duration_ms: if live { 0 } else { 180_000 },
        playable: true,
        ..Entry::default()
    }
}

fn folder(id: &str, title: &str) -> Entry {
    Entry { id: id.to_string(), title: title.to_string(), playable: true, ..Entry::default() }
}

pub fn browse(ask: Browse) -> Result<Listing, String> {
    let listing = match ask.at.as_str() {
        // Narrowed by its one column: only the live items, or only the files.
        // "All" comes as an empty pick, which is the whole root.
        "" if ask.picks.get("kind").is_some_and(|k| !k.is_empty()) => {
            let live = ask.picks.get("kind").map(String::as_str) == Some("live");
            Listing {
                title: "Fixture".into(),
                entries: vec![item("live:one", "Live One", true), item("live:two", "Live Two", true), item("file:one", "First Song", false)]
                    .into_iter()
                    .filter(|e| e.live == live)
                    .collect(),
                next: None,
                columns: Vec::new(),
            }
        }
        "" => Listing {
            title: "Fixture".into(),
            entries: vec![
                Entry { kind: EntryKind::Search, ..folder("search", "Search") },
                folder("genres", "Genres"),
                folder("long", "Long"),
                Entry { subtitle: "Somebody".into(), description: "A show about nothing.".into(), ..folder("show", "A Show") },
                item("live:one", "Live One", true),
                item("file:one", "First Song", false),
                Entry {
                    playable: false,
                    why_not: "it is only offered as HLS".into(),
                    ..item("hls:one", "Segmented", true)
                },
            ],
            next: None,
            columns: vec![Column {
                id: "kind".into(),
                title: "Kind".into(),
                entries: vec![
                    Entry { subtitle: "2".into(), ..folder("live", "Live") },
                    Entry { subtitle: "1".into(), ..folder("file", "Files") },
                ],
                preset: None,
            }],
        },
        "search" => {
            let text = ask.text.unwrap_or_default().to_lowercase();
            let entries = [item("live:one", "Live One", true), item("live:two", "Live Two", true)]
                .into_iter()
                .filter(|e| !text.is_empty() && e.title.to_lowercase().contains(&text))
                .collect();
            Listing { title: "Search".into(), entries, ..Listing::default() }
        }
        "genres" => Listing {
            title: "Genres".into(),
            entries: vec![folder("genre:jazz", "Jazz")],
            next: None,
            columns: Vec::new(),
        },
        "genre:jazz" => Listing {
            title: "Jazz".into(),
            entries: vec![
                Entry {
                    subtitle: "Somewhere".into(),
                    facts: vec![Fact::new("Country", "Somewhere"), Fact::new("Codec", "WAV")],
                    ..item("live:one", "Live One", true)
                },
                item("live:two", "Live Two", true),
            ],
            next: None,
            columns: Vec::new(),
        },
        "long" => {
            let page: usize = ask.cursor.as_deref().and_then(|c| c.parse().ok()).unwrap_or(0);
            let entries = (0..2)
                .map(|i| {
                    let n = page * 2 + i;
                    item(&format!("file:long-{n}"), &format!("Long {n}"), false)
                })
                .collect();
            Listing {
                title: "Long".into(),
                entries,
                next: (page < 2).then(|| (page + 1).to_string()),
                columns: Vec::new(),
            }
        }
        "show" => Listing {
            title: "A Show".into(),
            entries: ["one", "two"]
                .into_iter()
                .enumerate()
                .map(|(n, name)| Entry {
                    subtitle: "A Show".into(),
                    duration_ms: 2_000,
                    published_at: Some(1_700_000_000 + n as i64 * 86_400),
                    description: format!("What happens in episode {name}."),
                    ..item(&format!("ep:{name}"), &format!("Episode {name}"), false)
                })
                .collect(),
            next: None,
            columns: Vec::new(),
        },
        "hostile:loop" => Listing {
            title: "Loop".into(),
            entries: vec![item("file:one", "First Song", false)],
            next: Some(ask.cursor.unwrap_or_else(|| "same".into())),
            columns: Vec::new(),
        },
        "hostile:blank" => Listing {
            title: "Blank".into(),
            entries: vec![item("", "No id", false), item("x", "  ", false), item("ok", "Fine", false)],
            next: None,
            columns: Vec::new(),
        },
        "hostile:huge" => Listing {
            title: "Huge".into(),
            entries: (0..5000).map(|n| item(&format!("file:{n}"), "Many", false)).collect(),
            next: None,
            columns: Vec::new(),
        },
        other => return Err(format!("there is no \"{other}\" here")),
    };
    Ok(listing)
}

pub fn resolve(ask: Resolve) -> Result<Stream, String> {
    let id = ask.external_id.as_str();
    if id.starts_with("hls:") {
        return Ok(Stream {
            url: format!("{}/segmented.m3u8", base()),
            delivery: Delivery::Hls,
            ..Stream::default()
        });
    }
    if id.starts_with("live:") {
        return Ok(Stream {
            url: format!("{}/live/{}.wav", base(), &id[5..]),
            codec: Some("wav".into()),
            quality: Quality { codec: Some("wav".into()), ..Quality::default() },
            ..Stream::default()
        });
    }
    if let Some(name) = id.strip_prefix("ep:") {
        // A whole file with a beginning and an end, which is seeked and
        // resumed. Nothing is asked: the id says where it is.
        return Ok(Stream {
            url: format!("{}/episodes/{name}.wav", base()),
            codec: Some("wav".into()),
            ..Stream::default()
        });
    }
    if let Some(name) = id.strip_prefix("file:") {
        // The address comes from the "service", so a test can hand out
        // whatever it likes, and count how many times it was asked.
        let reply = host::get(&format!("{}/where/{}", base(), host::encode(name)))?;
        let url = reply.body.trim().to_string();
        let asked = host::store_get("resolved").map(|v| v.len()).unwrap_or(0);
        host::store_set("resolved", &vec![b'.'; asked + 1]);
        return Ok(Stream {
            url,
            codec: Some("flac".into()),
            // Short lived on purpose: a queue that outlives it has to ask again.
            expires_at: Some(host::now() + 60),
            ..Stream::default()
        });
    }
    Err(format!("\"{id}\" is not something I gave out"))
}

plugin! {
    api_version: 1,
    chilpa_search => search,
    chilpa_browse => browse,
    chilpa_resolver => resolve,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chilpa_plugin_sdk::testing;

    #[test]
    fn the_root_offers_a_search_folders_and_items() {
        let root = browse(Browse::root()).unwrap();
        assert_eq!(root.entries[0].kind, EntryKind::Search);
        assert!(root.entries.iter().any(|e| e.kind == EntryKind::Folder));
        assert!(root.entries.iter().any(|e| e.kind == EntryKind::Item && e.live));
    }

    #[test]
    fn a_long_folder_is_walked_by_cursor_to_its_end() {
        let mut cursor = None;
        let mut seen = Vec::new();
        loop {
            let page = browse(Browse { cursor: cursor.clone(), ..Browse::at("long") }).unwrap();
            seen.extend(page.entries.into_iter().map(|e| e.id));
            match page.next {
                Some(next) => cursor = Some(next),
                None => break,
            }
        }
        assert_eq!(seen.len(), 6);
    }

    #[test]
    fn a_file_is_resolved_by_asking_and_expires() {
        testing::reset();
        testing::set_clock(1000);
        testing::answer("https://fixture.test/where/one", 200, "https://fixture.test/one.flac\n");
        let stream = resolve(Resolve::new("file:one")).unwrap();
        assert_eq!(stream.url, "https://fixture.test/one.flac");
        assert_eq!(stream.expires_at, Some(1060));
        assert_eq!(testing::asked().len(), 1);
    }

    #[test]
    fn a_live_item_resolves_under_the_base_the_store_names() {
        testing::reset();
        assert_eq!(resolve(Resolve::new("live:one")).unwrap().url, "https://fixture.test/live/one.wav");
        host::store_set("base", b"http://127.0.0.1:9/\n");
        assert_eq!(resolve(Resolve::new("live:one")).unwrap().url, "http://127.0.0.1:9/live/one.wav");
        assert!(testing::asked().is_empty(), "a live item asks the network for nothing");
    }
}
