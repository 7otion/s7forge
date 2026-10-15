// MIT License
//
// Copyright (c) 2022 Gabriel Francisco Dos Santos
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.
//
// Modified by Burak Kartal on [24/06/2025]

pub mod workshop {
    use bincode::{Decode, Encode};
    use serde::Serialize;
    use steamworks::FileType;

    use crate::core::localplayer::PlayerSteamId;
    use crate::core::workshop::UgcItemVisibility;

    #[derive(Debug, Clone, Serialize, Encode, Decode)]
    pub struct WorkshopItemStatistic {
        pub num_subscriptions: Option<u64>, //   0	gets the number of subscriptions.
        pub num_favorites: Option<u64>,     //   1	gets the number of favorites.
        pub num_followers: Option<u64>,     //   2	gets the number of followers.
        pub num_unique_subscriptions: Option<u64>, // 3	gets the number of unique subscriptions.
        pub num_unique_favorites: Option<u64>, // 4	gets the number of unique favorites.
        pub num_unique_followers: Option<u64>, // 5	gets the number of unique followers.
        pub num_unique_website_views: Option<u64>, //  6	gets the number of unique views the item has on its steam workshop page.
        pub report_score: Option<u64>, //    7	gets the number of times the item has been reported.
        pub num_seconds_played: Option<u64>, //   8	gets the total number of seconds this item has been used across all players.
        pub num_playtime_sessions: Option<u64>, //    9	gets the total number of play sessions this item has been used in.
        pub num_comments: Option<u64>, //    10	gets the number of comments on the items that steam has on its steam workshop page.
        pub num_seconds_played_during_time_period: Option<u64>, //   11	gets the number of seconds this item has been used over the given time period.
        pub num_playtime_sessions_during_time_period: Option<u64>, //    12	Gets the number of sessions this item has been used in over the given time period.
    }

    impl WorkshopItemStatistic {
        fn from_query_results(results: &steamworks::QueryResults, index: u32) -> Self {
            Self {
                num_subscriptions: results
                    .statistic(index, steamworks::UGCStatisticType::Subscriptions),
                num_favorites: results.statistic(index, steamworks::UGCStatisticType::Favorites),
                num_followers: results.statistic(index, steamworks::UGCStatisticType::Followers),
                num_unique_subscriptions: results
                    .statistic(index, steamworks::UGCStatisticType::UniqueSubscriptions),
                num_unique_favorites: results
                    .statistic(index, steamworks::UGCStatisticType::UniqueFavorites),
                num_unique_followers: results
                    .statistic(index, steamworks::UGCStatisticType::UniqueFollowers),
                num_unique_website_views: results
                    .statistic(index, steamworks::UGCStatisticType::UniqueWebsiteViews),
                report_score: results.statistic(index, steamworks::UGCStatisticType::Reports),
                num_seconds_played: results
                    .statistic(index, steamworks::UGCStatisticType::SecondsPlayed),
                num_playtime_sessions: results
                    .statistic(index, steamworks::UGCStatisticType::PlaytimeSessions),
                num_comments: results.statistic(index, steamworks::UGCStatisticType::Comments),
                num_seconds_played_during_time_period: results.statistic(
                    index,
                    steamworks::UGCStatisticType::SecondsPlayedDuringTimePeriod,
                ),
                num_playtime_sessions_during_time_period: results.statistic(
                    index,
                    steamworks::UGCStatisticType::PlaytimeSessionsDuringTimePeriod,
                ),
            }
        }
    }
    #[derive(Debug, Clone, Serialize, Encode, Decode)]
    pub struct WorkshopItem {
        pub published_file_id: u64,
        pub creator_steam_game_id: Option<u32>,
        pub consumer_steam_game_id: Option<u32>,
        pub title: String,
        pub description: String,
        pub owner: PlayerSteamId,
        pub time_created: u64,
        pub time_updated: u64,
        pub visibility: UgcItemVisibility,
        pub banned: bool,
        pub accepted_for_use: bool,
        pub tags: Vec<String>,
        pub tags_truncated: bool,
        pub url: String,
        pub num_upvotes: u32,
        pub num_downvotes: u32,
        pub num_children: u32,
        pub preview_url: Option<String>,
        pub statistics: WorkshopItemStatistic,
        pub required_items: Vec<u64>,
        pub file_type: String,
        pub file_size: u32,
    }

    impl WorkshopItem {
        fn from_query_results(results: &steamworks::QueryResults, index: u32) -> Option<Self> {
            results.get(index).map(|item| {
                let required_items = results
                    .get_children(index)
                    .unwrap_or_default()
                    .into_iter()
                    .map(|file_id| file_id.0)
                    .collect();

                let published_file_id = item.published_file_id.0;

                let file_type = match item.file_type {
                    FileType::Community => "Community",
                    FileType::Microtransaction => "Microtransaction",
                    FileType::Collection => "Collection",
                    FileType::Art => "Art",
                    FileType::Video => "Video",
                    FileType::Screenshot => "Screenshot",
                    FileType::Game => "Game",
                    FileType::Software => "Software",
                    FileType::Concept => "Concept",
                    FileType::WebGuide => "WebGuide",
                    FileType::IntegratedGuide => "IntegratedGuide",
                    FileType::Merch => "Merch",
                    FileType::ControllerBinding => "ControllerBinding",
                    FileType::SteamworksAccessInvite => "SteamworksAccessInvite",
                    FileType::SteamVideo => "SteamVideo",
                    FileType::GameManagedItem => "GameManagedItem",
                };

                Self {
                    published_file_id,
                    creator_steam_game_id: item.creator_app_id.map(|id| id.0),
                    consumer_steam_game_id: item.consumer_app_id.map(|id| id.0),
                    title: item.title,
                    description: item.description,
                    owner: PlayerSteamId::from_steamid(item.owner),
                    time_created: item.time_created as u64,
                    time_updated: item.time_updated as u64,
                    visibility: item.visibility.into(),
                    banned: item.banned,
                    accepted_for_use: item.accepted_for_use,
                    tags: item.tags,
                    tags_truncated: item.tags_truncated,
                    url: item.url,
                    num_upvotes: item.num_upvotes,
                    num_downvotes: item.num_downvotes,
                    num_children: item.num_children,
                    preview_url: results.preview_url(index),
                    statistics: WorkshopItemStatistic::from_query_results(results, index),
                    required_items,
                    file_type: file_type.to_string(),
                    file_size: item.file_size,
                }
            })
        }
    }

    #[derive(Debug)]
    pub struct WorkshopItemsResult {
        pub items: Vec<Option<WorkshopItem>>,
    }

    impl WorkshopItemsResult {
        pub fn from_query_results(query_results: steamworks::QueryResults) -> Self {
            Self {
                items: (0..query_results.returned_results())
                    .map(|i| WorkshopItem::from_query_results(&query_results, i))
                    .collect(),
            }
        }
    }
}
