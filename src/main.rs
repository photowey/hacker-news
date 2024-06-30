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

#![allow(non_snake_case)]

use chrono::Local;
use dioxus::prelude::*;

use hackernews::api;
use hackernews::types::{Comment, PreviewState, StoryItem};

// ----------------------------------------------------------------

// @see https://dioxuslabs.com/learn/0.5/guide/your_first_component

fn main() {
    launch(App);
}

// ----------------------------------------------------------------

fn App() -> Element {
    use_context_provider(|| Signal::new(PreviewState::Unset));
    rsx! {
        div {
            display: "flex",
            flex_direction: "row",
            width: "100%",
            div { width: "50%", Stories {} }
            div { width: "50%", Preview {} }
        }
    }
}

// ----------------------------------------------------------------

fn Stories() -> Element {
    let stories = use_resource(move || api::get_stories(10));

    match &*stories.read_unchecked() {
        Some(Ok(list)) => {
            rsx! {
                div {
                    for story in list {
                        StoryListing { story: story.clone() }
                    }
                }
            }
        }
        Some(Err(err)) => {
            rsx! {"An error occurred while fetching stories {err}"}
        }
        None => {
            rsx! {"Loading items"}
        }
    }
}

// ----------------------------------------------------------------

fn Preview() -> Element {
    let preview_state = consume_context::<Signal<PreviewState>>();

    match preview_state() {
        PreviewState::Unset => rsx! { "Hover over a story to preview it here" },
        PreviewState::Loading => rsx! { "Loading..." },
        PreviewState::Loaded(story) => {
            rsx! {
                div {
                    padding: "0.5rem",
                    div {
                        font_size: "1.5rem",
                        a {
                            href: story.item.url,
                            "{story.item.title}"
                        }
                    }

                    div {
                        dangerous_inner_html: story.item.text
                    }

                    for comment in &story.comments {
                        Comment {
                            comment: comment.clone()
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn Comment(comment: Comment) -> Element {
    rsx! {
        div {
            padding: "0.5rem",
            div {
                color: "gray",
                "by {comment.by}"
            }

            div {
                dangerous_inner_html:
                "{comment.text}"
            }

            for kid in &comment.sub_comments {
                Comment {
                    comment: kid.clone()
                }
            }
        }
    }
}

// ----------------------------------------------------------------

#[component]
fn StoryListing(story: ReadOnlySignal<StoryItem>) -> Element {
    let preview_state = consume_context::<Signal<PreviewState>>();

    let StoryItem {
        title,
        url,
        by,
        score,
        time,
        kids,
        id,
        ..
    } = story();

    let full_story = use_signal(|| None);

    let url = url.as_deref().unwrap_or_default();

    let mut hostname = url
        .trim_start_matches("https://")
        .trim_start_matches("http://")
        .trim_start_matches("www.");

    if hostname.is_empty() {
        hostname = "google.com"
    }

    let score = format!("{score} {}", if score == 1 { " point" } else { " points" });

    let comments = format!(
        "{} {}",
        kids.len(),
        if kids.len() == 1 {
            " comment"
        } else {
            " comments"
        }
    );

    let local_time = time.with_timezone(&Local);
    let time = local_time.format("%D %l:%M %p");

    rsx! {
        div {
            padding: "0.5rem",
            position: "relative",
            onmouseenter: move |_event| {
                api::resolve_story(full_story, preview_state, id)
            },
            div {
                font_size: "1.5rem",
                a {
                    href: url,
                    onfocus: move |_event| {
                        api::resolve_story(full_story, preview_state, id)
                    },
                    "{title}"
                }
                a {
                    color: "gray",
                    href: "https://news.ycombinator.com/from?site={hostname}",
                    text_decoration: "none",
                    " ({hostname})"
                }
            }
            div {
                display: "flex",
                flex_direction: "row",
                color: "gray",
                div { "{score}" }
                div { padding_left: "0.5rem", "by {by}" }
                div { padding_left: "0.5rem", "{time}" }
                div { padding_left: "0.5rem", "{comments}" }
            }
        }
    }
}

// ----------------------------------------------------------------
