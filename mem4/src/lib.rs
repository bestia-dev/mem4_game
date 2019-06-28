//! **mem4 game - Wasm/WebAssembly written in Rust with Virtual Dom Dodrio and `WebSocket` communication**

//! mem4 is a simple memory game made primarily for learning Rust.  
//! ## Virtual DOM
//! Constructing a HTML page with Virtual DOM (vdom) is simple because it is rendered completely on every tick (animation frame).  
//! Sometimes is hard for the developer to think what should change in the UI when some data changes.  
//! The data can change from many different events and very chaotically (asyncronous).  
//! It is easier to think how to render the complete DOM for the given data.  
//! The dodrio library has ticks, time intervals when it do something.  
//! If a rendering is scheduled, it will be done on the next tick.  
//! If a rendering is not scheduled I believe nothing happens.  
//! This enables asyncronous changing of data and rendering. They cannot happen in the same moment.  
//! When something changes that can change the DOM, the rendering must be scheduled.  
//! ## State - action - message - state
//! The game is in a cetrain GameState. The user then makes an action. This action changes the GameData.  
//! Then a message is sent to other players so they can also chenge their local GameData.  
//! The result is a new GameState.  
//!

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
