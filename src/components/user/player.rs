use crate::general::types::Song;
use leptos::{
    logging::{error, log},
    prelude::*,
    *,
};

#[component]
pub fn Player(
    #[prop(into)] position: Signal<f32>,
    #[prop(into)] current_song: Signal<Option<Song>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    create_effect(move |_| {
        current_song.with(|song| {
            if let Some(song) = song {
                set_bg_img(&song.image_url);
            }
        });
    });

    let song_length = move || current_song().map(|s| s.duration).unwrap_or_default();

    view! {
        <div class="player">
            <img
                prop:src=move || current_song().map(|s| s.image_url).unwrap_or_default()
                alt="the album cover of the current song"
            />

            <div class="info" id="info">
                <div
                    class="title"
                    id="title"
                    class:scroll=move || {
                        current_song.track();
                        if cfg!(target_arch = "wasm32") {
                            will_element_overflow("title", Some("info"))
                        } else {
                            false
                        }
                    }
                >

                    {move || {
                        let is_overflowing = if cfg!(target_arch = "wasm32") {
                            will_element_overflow("title", Some("info"))
                        } else {
                            false
                        };
                        current_song()
                            .map(|s| s.name.clone())
                            .unwrap_or_default()
                            .repeat({ if is_overflowing { 2 } else { 1 } })
                    }}

                </div>
                <div
                    class="artist"
                    id="artist"
                    class:scroll=move || {
                        current_song.track();
                        if cfg!(target_arch = "wasm32") {
                            will_element_overflow("artist", Some("info"))
                        } else {
                            false
                        }
                    }
                >

                    {move || {
                        let artists = current_song()
                            .map(|s| s.artists.join(", "))
                            .unwrap_or_default();
                        let is_overflowing = if cfg!(target_arch = "wasm32") {
                            will_element_overflow("artist", Some("info"))
                        } else {
                            false
                        };
                        artists.repeat({ if is_overflowing { 2 } else { 1 } })
                    }}

                </div>
            </div>

            <div class="progress">
                <div class="bar">
                    <div
                        class="position"
                        style:width=move || format!("{}%", position() * 100.0)
                    ></div>
                </div>
                <div class="times">
                    <div>
                        {move || millis_to_min_sec((position() * song_length() as f32) as u32)}
                    </div>
                    <div>{move || millis_to_min_sec(song_length())}</div>
                </div>
            </div>

            {if let Some(extra_elements) = children {
                extra_elements().into_view()
            } else {
                view! {}.into_view()
            }}

        </div>
    }
}

pub fn set_bg_img(url: &str) {
    let body = web_sys::window()
        .unwrap()
        .document()
        .unwrap()
        .body()
        .unwrap();
    body
    .style()
    .set_property(
        "background-image", 
        &format!("radial-gradient(50% 50% at 50% 50%, rgba(0, 0, 0, 0.60) 0%, rgba(0, 0, 0, 0.75) 100%), url({})", url)).unwrap();
}

pub fn will_element_overflow(element_id: &str, parent_id: Option<&str>) -> bool {
    use web_sys::*;
    let document = window().unwrap().document().unwrap();
    let element = document
        .get_element_by_id(element_id)
        .unwrap_or_else(|| panic!("element with id {} not found", element_id));

    let parent_width = {
        if let Some(parent_id) = parent_id {
            document
                .get_element_by_id(parent_id)
                .unwrap_or_else(|| panic!("parent element with class {} not found", parent_id))
        } else {
            element.parent_element().expect("the element has no parent")
        }
    }
    .scroll_width();

    let is_overflowing = parent_width < element.client_width();
    log!(
        "is_overflowing :{}, {} width:{}, {} width:{}",
        is_overflowing,
        element_id,
        element.scroll_width(),
        parent_id.unwrap_or("parent"),
        parent_width
    );
    is_overflowing
}

pub fn get_width_of_element(id: &str) -> i32 {
    use web_sys::*;
    let document = window().unwrap().document().unwrap();

    let width=document
        .get_element_by_id(id)
        .unwrap_or_else(|| panic!("element with id {} not found", id))
        .scroll_width();
    log!("width of element {} is {}", id, width);
    width
}

pub fn millis_to_min_sec(millis: u32) -> String {
    let seconds = millis / 1000;
    let minutes = seconds / 60;
    let seconds = seconds % 60;
    format!("{:01}:{:02}", minutes, seconds)
}
