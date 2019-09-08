//! **mem4 - Learning Rust and Wasm/WebAssembly with Virtual Dom Dodrio and WebSocket communication**  
//region: lmake_readme insert "readme.md"
//TODO: itis not working probably because of workspace
//endregion: lmake_readme insert "readme.md"

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
    //because then wasm-pack build --target web returns an error: export run not found 
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
    //Why is this bad : Doc is good. rustc has a MISSING_DOCS allowed-by-default lint for public members, but has no way to enforce documentation of private items. This lint fixes that.
    clippy::doc_markdown,
    //Why is this bad : Splitting the implementation of a type makes the code harder to navigate.
    clippy::multiple_inherent_impl,
)]
//endregion

//region: mod is used only in lib file. All the rest use use crate
mod divcardmoniker;
mod divfordebugging;
mod divgridcontainer;
mod divplayeractions;
mod divplayersandscores;
mod divrulesanddescription;
mod fetchmod;
mod fetchgameconfig;
mod gamedata;
mod javascriptimportmod;
mod logmod;
mod rootrenderingcomponent;
mod statusinviteaskbegin;
mod statusinviteasked;
mod statusinviteasking;
mod statusplayagain;
mod statusplaybefore1stcard;
mod statusplaybefore2ndcard;
mod statustaketurnbegin;
mod websocketcommunication;
mod websocketreconnect;
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

    // Get the document's container to render the virtual Dom component.
    let window = unwrap!(web_sys::window(), "error: web_sys::window");
    let document = unwrap!(window.document(), "error: window.document");
    let div_for_virtual_dom = unwrap!(
        document.get_element_by_id("div_for_virtual_dom"),
        "No #div_for_virtual_dom"
    );

    let mut rng = SmallRng::from_entropy();
    //my_ws_uid is random generated on the client and sent to
    //WebSocket server with an url param
    let my_ws_uid: usize = rng.gen_range(1, 9999);

    //find out URL
    let location_href = unwrap!(window.location().href(), "href not known");

    //WebSocket connection
    let ws = websocketcommunication::setup_ws_connection(location_href.clone(), my_ws_uid);
    //I don't know why is needed to clone the WebSocket connection
    let ws_c = ws.clone();

    // Construct a new RootRenderingComponent.
    //I added ws_c so that I can send messages on WebSocket

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
