/*
 * Copyright © 2024 the original author or authors.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

// ----------------------------------------------------------------

use dioxus::prelude::{ReadableOptionExt, Signal, Writable};
// Define the Hackernews API
use futures::future::join_all;

use crate::api;
use crate::types::{Comment, PreviewState, StoryItem, StoryPageData};

// ----------------------------------------------------------------

pub static BASE_API_URL: &str = "https://hacker-news.firebaseio.com/v0/";
pub static ITEM_API: &str = "item/";
pub static USER_API: &str = "user/";
const COMMENT_DEPTH: i64 = 2;

// ----------------------------------------------------------------

pub async fn get_story_preview(id: i64) -> Result<StoryItem, reqwest::Error> {
    let url = format!("{}{}{}.json", BASE_API_URL, ITEM_API, id);
    reqwest::get(&url).await?.json().await
}

pub async fn get_stories(count: usize) -> Result<Vec<StoryItem>, reqwest::Error> {
    let url = format!("{}topstories.json", BASE_API_URL);
    let stories_ids = &reqwest::get(&url).await?.json::<Vec<i64>>().await?[..count];

    let story_futures = stories_ids[..usize::min(stories_ids.len(), count)]
        .iter()
        .map(|&story_id| get_story_preview(story_id));
    let stories = join_all(story_futures)
        .await
        .into_iter()
        .filter_map(|story| story.ok())
        .collect();
    Ok(stories)
}

pub async fn get_story(id: i64) -> Result<StoryPageData, reqwest::Error> {
    let url = format!("{}{}{}.json", BASE_API_URL, ITEM_API, id);
    let mut story = reqwest::get(&url).await?.json::<StoryPageData>().await?;
    let comment_futures = story.item.kids.iter().map(|&id| get_comment(id));
    let comments = join_all(comment_futures)
        .await
        .into_iter()
        .filter_map(|c| c.ok())
        .collect();

    story.comments = comments;
    Ok(story)
}

pub async fn resolve_story(
    mut full_story: Signal<Option<StoryPageData>>,
    mut preview_state: Signal<PreviewState>,
    story_id: i64,
) {
    if let Some(cached) = full_story.as_ref() {
        *preview_state.write() = PreviewState::Loaded(cached.clone());
        return;
    }

    *preview_state.write() = PreviewState::Loading;
    if let Ok(story) = api::get_story(story_id).await {
        *preview_state.write() = PreviewState::Loaded(story.clone());
        *full_story.write() = Some(story);
    }
}

#[async_recursion::async_recursion(? Send)]
pub async fn get_comment_with_depth(id: i64, depth: i64) -> Result<Comment, reqwest::Error> {
    let url = format!("{}{}{}.json", BASE_API_URL, ITEM_API, id);
    let mut comment = reqwest::get(&url).await?.json::<Comment>().await?;
    if depth > 0 {
        let sub_comments_futures = comment
            .kids
            .iter()
            .map(|story_id| get_comment_with_depth(*story_id, depth - 1));
        comment.sub_comments = join_all(sub_comments_futures)
            .await
            .into_iter()
            .filter_map(|c| c.ok())
            .collect();
    }
    Ok(comment)
}

pub async fn get_comment(comment_id: i64) -> Result<Comment, reqwest::Error> {
    let comment = get_comment_with_depth(comment_id, COMMENT_DEPTH).await?;
    Ok(comment)
}
