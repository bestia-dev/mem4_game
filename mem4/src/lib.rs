//! **mem4 - Learning Rust and Wasm/WebAssembly with Virtual Dom Dodrio and WebSocket communication**  
//region: documentation
//! mem4 is a simple memory game made primarily for learning the Rust programming language.  
//! ## Rust and Wasm/WebAssembly
//! TODO: description
//! ## Virtual DOM
//! Constructing a HTML page with Virtual DOM (vdom) is easier  
//! because it is rendered completely on every tick (animation frame).  
//! Sometimes is hard for the developer to think what should change in the UI when some data changes.  
//! The data can change from many different events and very chaotically (asynchronously).  
//! It is easier to think how to render the complete DOM for the given data.  
//! The dodrio library has ticks, time intervals when it do something.  
//! If a rendering is scheduled, it will be done on the next tick.  
//! If a rendering is not scheduled I believe nothing happens.  
//! This enables asynchronous changing of data and rendering. They cannot happen theoretically in the same exact moment. So, no data race here.  
//! When `GameData` change and we now it will affect the DOM, then rendering must be scheduled.  
//! The root_rendering_component is splitted easily into sub-components.  
//! [[https://github.com/LucianoBestia/mem4_game/blob/master/docsimgs/subcomponents.png|alt=subcomponents]]  
//!
//! ## WebSocket communication
//! TODO: description
//! ## Status1 - User action - Status2, Status1 - WsMessage - Status2
//! In one moment the game is in a certain Game Status. The user then makes an action. This action changes the `GameData`.  
//! Then a message is sent to other players so they can also change their local `GameData`.  
//! The rendering is scheduled and it will happen shortly (async). The result is a new `GameStatus`.  
//!  
//! | Game Status1        | Render                     | User action                                 | Condition                           | GameStatus2 t.p.  | Sends Msg        | On rcv Msg o.p.            | GameStatus2 o.p.                  |
//! | ------------------ | -------------------------- | ------------------------------------------- | ----------------------------------- | ---------------- | ---------------- | -------------------------- | -------------------------------- |
//! | WantToPlayAskBegin | div_want_to_play_ask_begin | div_want_to_play_ask_begin_on_click         | -                                   | WantToPlayAsking | WantToPlay       | on_msg_want_to_play        | WantToPlayAsked                  |
//! | WantToPlayAsked    | div_want_to_play_asked     | div_want_to_play_asked_on_click             | -                                   | PlayAccepted     | PlayAccept       | on_msg_play_accept         | -                                |
//! | WantToPlayAsking   | div_want_to_play_asking    | game_data_init                              | -                                   | PlayBefore1Card  | GameDataInit     | on_msg_game_data_init      | PlayBefore1Card                  |
//! | PlayBefore1Card    | div_grid_container | div_grid_item_on_click, rrc.card_on_click_1_card(); | -                                   | PlayBefore2Card  | PlayerClick1Card | on_msg_player_click_1_card | PlayBefore2Card                  |
//! | PlayBefore2Card    | div_grid_container | div_grid_item_on_click, rrc.card_on_click_2_Card(); | If card match and points<allpoint   | PlayBefore1Card  | PlayerClick2Card | on_msg player_click_2_card | PlayBefore1Card                  |
//! | II                 | II                         | II                                          | If card match and points=>allpoints | PlayAgain        | PlayAgain        | on_msg_play_again          | PlayAgain                        |
//! | II                 | II                         | II                                          | else                                | TakeTurnBegin    | TakeTurnBegin    | on_msg_take_turn           | TakeTurnBegin                    |
//! | TakeTurnBegin      | div_take_turn_begin (div_grid_container) | div_take_turn_begin_on_click  | -                                   | PlayBefore1Card  | TakeTurnEnd      | on_take_turn_end           | PlayBefore1Card, the next player |
//! | PlayAgain          | div_play_again (div_grid_container) | window.location().reload()         | -                                   | -                | -                | -                          | -                                |
//! |  |  |  |  |  |  |  |  |
//!  
//! t.p. = this player,   o.p. = other players,  rrc = root_rendering_component, rcv = receive
//! 1. Some actions can have different results. For example the condition card match or card don’t match.  
//! 2. one action must be only for one status1. This action changes Status for this player and sends Msg to other players.  
//! 3. on receive msg can produce only one status2.  
//! 4. in this table I ignore msgs for the server like GetConfig  
//!  
//! ## Futures and promises, Rust and JavaScript
//! JavaScript is all asynchronous. Wasm is nothing else then a shortcut to the JavaScript engine.  
//! So everything is asynchronous. This is pretty hard to grasp. Everything is Promises and futures.  
//! Still better then `CallBacks`, but very hard to understand. And there is a constant
//! jumping from thinking in Rust to thinking is JavaScript and back that is pretty confusing.  
//! JavaScript does not have a good idea of Rust datatypes. All there is is a `JSValue` type.  
//! The library wasm-bindgen has made a fantastic job of giving to Rust the ability to call
//! anything JavaScript can call, but the way of doing it is sometimes very hard to understand.  
//! ## Browser console
//! At least in modern browsers (Firefox and Chrome) we have the developer tools F12 and there is a
//! console we can output to. So we can debug what is going on with our program.  
//! ## Modules
//! ## Clippy
//!
//!
//endregion

//needed for dodrio! macro (typed-html)
#![recursion_limit = "512"]
//region: Clippy
#![warn(
    clippy::all,
    clippy::restriction,
    clippy::pedantic,
    clippy::nursery,
    clippy::cargo,
    //variable shadowing is idiomatic to Rust, but unnatural to me.
    clippy::shadow_reuse,
    clippy::shadow_same,
    clippy::shadow_unrelated,

)]
#![allow(
    //library from dependencies have this clippy warnings. Not my code.
    //Why is this bad: It will be more difficult for users to discover the purpose of the crate, 
    //and key information related to it.
    clippy::cargo_common_metadata,
    //Why is this bad : This bloats the size of targets, and can lead to confusing error messages when 
    //structs or traits are used interchangeably between different versions of a crate.
    clippy::multiple_crate_versions,
    //Why is this bad : As the edition guide says, it is highly unlikely that you work with any possible 
    //version of your dependency, and wildcard dependencies would cause unnecessary 
    //breakage in the ecosystem.
    clippy::wildcard_dependencies,
    //Rust is more idiomatic without return statement
    //Why is this bad : Actually omitting the return keyword is idiomatic Rust code. 
    //Programmers coming from other languages might prefer the expressiveness of return. 
    //It’s possible to miss the last returning statement because the only difference 
    //is a missing ;. Especially in bigger code with multiple return paths having a 
    //return keyword makes it easier to find the corresponding statements.
    clippy::implicit_return,
    //I have private function inside a function. Self does not work there.
    //Why is this bad: Unnecessary repetition. Mixed use of Self and struct name feels inconsistent.
    clippy::use_self,
    //Cannot add #[inline] to the start function with #[wasm_bindgen(start)]
    //because then wasm-pack build --target web returns an error: export `run` not found 
    //Why is this bad: In general, it is not. Functions can be inlined across crates when that’s profitable 
    //as long as any form of LTO is used. When LTO is disabled, functions that are not #[inline] 
    //cannot be inlined across crates. Certain types of crates might intend for most of the 
    //methods in their public API to be able to be inlined across crates even when LTO is disabled. 
    //For these types of crates, enabling this lint might make sense. It allows the crate to 
    //require all exported methods to be #[inline] by default, and then opt out for specific 
    //methods where this might not make sense.
    clippy::missing_inline_in_public_items,
    //Why is this bad: This is only checked against overflow in debug builds. In some applications one wants explicitly checked, wrapping or saturating arithmetic.
    //clippy::integer_arithmetic,
    //Why is this bad: For some embedded systems or kernel development, it can be useful to rule out floating-point numbers.
    clippy::float_arithmetic,
    //Why is this bad
    //Doc is good. rustc has a MISSING_DOCS allowed-by-default lint for public members, but has no way to enforce documentation of private items. This lint fixes that.
    clippy::doc_markdown,
)]
//endregion

//region: mod is used only in lib file. All the rest use `use crate`
mod cardmoniker;
mod gamedata;
mod gridcontainer;
mod playeractions;
mod playersandscores;
mod rootrenderingcomponent;
mod rulesanddescription;
mod websocketcommunication;
//endregion

//region: extern and use statements
//Strum is a set of macros and traits for working with enums and strings easier in Rust.
extern crate console_error_panic_hook;
extern crate log;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate mem4_common;
extern crate serde_json;
extern crate strum;
extern crate strum_macros;
extern crate web_sys;
#[macro_use]
extern crate unwrap;
extern crate conv;

use rand::rngs::SmallRng;
use rand::FromEntropy;
use rand::Rng;
use wasm_bindgen::prelude::*;
//endregion

//region: wasm_bindgen(start) is where everything starts
#[wasm_bindgen(start)]
///To start the Wasm application, wasm_bindgen runs this functions
pub fn run() -> Result<(), JsValue> {
    // Initialize debugging for when/if something goes wrong.
    console_error_panic_hook::set_once();

    // Get the document's container to render the virtual dom component.
    let window = unwrap!(web_sys::window(), "error: web_sys::window");
    let document = unwrap!(window.document(), "error: window.document");
    let div_for_virtual_dom = unwrap!(
        document.get_element_by_id("div_for_virtual_dom"),
        "No #div_for_virtual_dom"
    );

    let mut rng = SmallRng::from_entropy();
    //my_ws_uid is random generated on the client and sent to
    //websocket server with an url param
    let my_ws_uid: usize = rng.gen_range(1, 9999);

    //find out URL
    let location_href = unwrap!(window.location().href(), "href not known");

    //websocket connection
    let ws = websocketcommunication::setup_ws_connection(location_href.clone(), my_ws_uid);
    //I don't know why is needed to clone the websocket connection
    let ws_c = ws.clone();

    // Construct a new `RootRenderingComponent`.
    //I added ws_c so that I can send messages on websocket

    let mut root_rendering_component =
        rootrenderingcomponent::RootRenderingComponent::new(ws_c, my_ws_uid);
    root_rendering_component.game_data.href = location_href;

    // Mount the component to the `<div id="div_for_virtual_dom">`.
    let vdom = dodrio::Vdom::new(&div_for_virtual_dom, root_rendering_component);

    websocketcommunication::setup_all_ws_events(&ws, vdom.weak());

    // Run the component forever. Forget to drop the memory.
    vdom.forget();

    Ok(())
}
//endregion

/// Get the top-level window's session storage.
/// TODO: to save user preferences maybe?
pub fn session_storage() -> web_sys::Storage {
    let window = unwrap!(web_sys::window(), "error: web_sys::window");
    window.session_storage().unwrap_throw().unwrap_throw()
}
//endregion
