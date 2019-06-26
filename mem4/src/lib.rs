//region: App Description for docs
//! Learning Rust Wasm/WebAssembly with Virtual Dom Dodrio with `WebSocket` communication.
//! mem4 is a simple game for kids.
//! Constructing a HTML page with Virtual DOM (vdom) is simple because it is rendered completely every tick (animation frame).
//! For the developer it is hard to think what should change in the UI when some data changes.
//! It is easier to think how to render the complete DOM for the given data.
//! The dodrio library has ticks, time intervals when it do something.
//! If a rendering is scheduled it will be done on the next tick.
//! If a rendering is not scheduled I believe nothing happens.
//! read Readme.md
//! read StructModel.md
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
)]
//endregion

//region: extern and use statements
mod gamedata;
mod playersandscores;
mod rulesanddescription;
mod websocketcommunication;
use crate::gamedata::{CardStatusCardFace, GameData, GameState, Size2d};
use crate::playersandscores::PlayersAndScores;
use crate::rulesanddescription::RulesAndDescription;

use crate::websocketcommunication::setup_ws_connection;
use crate::websocketcommunication::setup_ws_msg_recv;
use crate::websocketcommunication::setup_ws_onclose;
use crate::websocketcommunication::setup_ws_onerror;
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

use conv::*;
use dodrio::builder::*;
use dodrio::bumpalo::{self, Bump};
use dodrio::{Cached, Node, Render};
use mem4_common::{Player, WsMessage};
use rand::rngs::SmallRng;
use rand::FromEntropy;
use rand::Rng;
use typed_html::dodrio;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast; //don't remove this. It is needed for dyn_into.
use web_sys::{console, WebSocket};
//endregion

//region: enum, structs, const,...
///game title
const GAME_TITLE: &str = "mem4";
///fixed filename for card face down
const SRC_FOR_CARD_FACE_DOWN: &str = "img/mem_image_00_cardfacedown.png";

///Root Render Component: the card grid struct has all the needed data for play logic and rendering
struct RootRenderingComponent {
    ///game data will be inside of Root, but reference for all other RenderingComponents
    game_data: GameData,
    ///subComponent: score
    players_and_scores: Cached<PlayersAndScores>,
    ///subComponent: the static parts can be cached.
    /// I am not sure if a field in this struct is the best place to put it.
    cached_rules_and_description: Cached<RulesAndDescription>,
}
//endregion

//region: wasm_bindgen(start) is where everything starts
#[wasm_bindgen(start)]
///wasm_bindgen runs this functions at start
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
    let ws = setup_ws_connection(location_href.clone(), my_ws_uid);
    //I don't know why is needed to clone the websocket connection
    let ws_c = ws.clone();

    // Construct a new `RootRenderingComponent`.
    //I added ws_c so that I can send messages on websocket

    let mut root_rendering_component = RootRenderingComponent::new(ws_c, my_ws_uid);
    root_rendering_component.game_data.href = location_href;

    // Mount the component to the `<div id="div_for_virtual_dom">`.
    let vdom = dodrio::Vdom::new(&div_for_virtual_dom, root_rendering_component);

    setup_all_ws_events(&ws, vdom.weak());

    // Run the component forever. Forget to drop the memory.
    vdom.forget();

    Ok(())
}
//endregion

///setup all ws events
pub fn setup_all_ws_events(ws: &WebSocket, weak: dodrio::VdomWeak) {
    //websocket on receive message callback
    setup_ws_msg_recv(ws, weak.clone());

    //websocket on error message callback
    setup_ws_onerror(ws, weak.clone());

    //websocket on close message callback
    setup_ws_onclose(ws, weak);
}

//region: Helper functions

///change the newline lines ending into <br> node
fn text_with_br_newline<'a>(txt: &'a str, bump: &'a Bump) -> Vec<Node<'a>> {
    let mut vec_text_node = Vec::new();
    let spl = txt.lines();
    for part in spl {
        vec_text_node.push(text(part));
        vec_text_node.push(br(bump).finish());
    }
    vec_text_node
}
/// Get the top-level window's session storage.
pub fn session_storage() -> web_sys::Storage {
    let window = unwrap!(web_sys::window(), "error: web_sys::window");
    window.session_storage().unwrap_throw().unwrap_throw()
}
//endregion

//region:RootRenderingComponent struct is the only persistant data we have in Rust Virtual Dom.dodrio
//in the constructor we initialize that data.
//Later onclick we change this data.
//at every animation frame we use only this data to render the virtual Dom.
//It knows nothing about HTML and Virtual dom.
impl RootRenderingComponent {
    /// Construct a new `RootRenderingComponent` component. Only once at the begining.
    pub fn new(ws: WebSocket, my_ws_uid: usize) -> Self {
        let game_data = GameData::new(ws, my_ws_uid);

        let game_rule_01 = RulesAndDescription {};
        let cached_rules_and_description = Cached::new(game_rule_01);
        let players_and_scores = Cached::new(PlayersAndScores::new());

        RootRenderingComponent {
            game_data,
            players_and_scores,
            cached_rules_and_description,
        }
    }
    ///check invalidate render cache for all sub components
    fn check_invalidate_for_all_components(&mut self) {
        if self.players_and_scores.update_intern_cache(&self.game_data) {
            Cached::invalidate(&mut self.players_and_scores);
        }
    }
    ///The onclick event passed by javascript executes all the logic
    ///and changes only the fields of the Card Grid struct.
    ///That stuct is the only permanent data storage for later render the virtual dom.
    fn card_on_click(&mut self) {
        //get this_click_card_index from game_data
        let this_click_card_index = if self.game_data.count_click_inside_one_turn == 1 {
            self.game_data.card_index_of_first_click
        } else {
            self.game_data.card_index_of_second_click
        };

        if self.game_data.count_click_inside_one_turn == 1
            || self.game_data.count_click_inside_one_turn == 2
        {
            //flip the card up
            unwrap!(
                self.game_data.vec_cards.get_mut(this_click_card_index),
                "error this_click_card_index"
            )
            .status = CardStatusCardFace::UpTemporary;

            if self.game_data.count_click_inside_one_turn == 2 {
                //if is the second click, flip the card and then check for card match

                //if the cards match, player get one point and continues another turn
                if unwrap!(
                    self.game_data
                        .vec_cards
                        .get(self.game_data.card_index_of_first_click),
                    "error game_data.card_index_of_first_click"
                )
                .card_number_and_img_src
                    == unwrap!(
                        self.game_data
                            .vec_cards
                            .get(self.game_data.card_index_of_second_click),
                        "error game_data.card_index_of_second_click"
                    )
                    .card_number_and_img_src
                {
                    //give points
                    unwrap!(
                        self.game_data
                            .players
                            .get_mut(unwrap!(self.game_data.player_turn.checked_sub(1))),
                        "self.game_data.players.get_mu(self.game_data.player_turn - 1)"
                    )
                    .points += 1;

                    // the two cards matches. make them permanent FaceUp
                    let x1 = self.game_data.card_index_of_first_click;
                    let x2 = self.game_data.card_index_of_second_click;
                    unwrap!(
                        self.game_data.vec_cards.get_mut(x1),
                        "error game_data.card_index_of_first_click"
                    )
                    .status = CardStatusCardFace::UpPermanently;
                    unwrap!(
                        self.game_data.vec_cards.get_mut(x2),
                        "error game_data.card_index_of_second_click"
                    )
                    .status = CardStatusCardFace::UpPermanently;
                    self.game_data.count_click_inside_one_turn = 0;
                    //if the sum of points is number of card/2, the game is over
                    let mut point_sum = 0;
                    for x in &self.game_data.players {
                        point_sum += x.points;
                    }
                    if unwrap!(self.game_data.vec_cards.len().checked_div(2)) == point_sum {
                        self.game_data.game_state = GameState::EndGame;
                        //send message
                        unwrap!(
                            self.game_data.ws.send_with_str(
                                &serde_json::to_string(&WsMessage::EndGame {
                                    my_ws_uid: self.game_data.my_ws_uid,
                                    players: unwrap!(
                                        serde_json::to_string(&self.game_data.players),
                                        "serde_json::to_string(&self.game_data.players)"
                                    ),
                                })
                                .expect("error sending EndGame"),
                            ),
                            "Failed to send EndGame"
                        );
                    }
                }
            }
        }
        self.check_invalidate_for_all_components();
    }
    ///fn on change for both click and we msg.
    fn take_turn(&mut self) {
        self.game_data.player_turn = if self.game_data.player_turn < self.game_data.players.len() {
            unwrap!(self.game_data.player_turn.checked_add(1))
        } else {
            1
        };

        //click on Change button closes first and second card
        let x1 = self.game_data.card_index_of_first_click;
        let x2 = self.game_data.card_index_of_second_click;
        unwrap!(
            self.game_data.vec_cards.get_mut(x1),
            "error game_data.card_index_of_first_click "
        )
        .status = CardStatusCardFace::Down;
        unwrap!(
            self.game_data.vec_cards.get_mut(x2),
            "error game_data.card_index_of_second_click"
        )
        .status = CardStatusCardFace::Down;
        self.game_data.card_index_of_first_click = 0;
        self.game_data.card_index_of_second_click = 0;
        self.game_data.count_click_inside_one_turn = 0;
        self.check_invalidate_for_all_components();
    }
    ///prepares the game data
    fn game_data_init(&mut self) {
        self.game_data.content_folder_name = self.game_data.asked_folder_name.clone();
        self.game_data.prepare_random_data();
        self.game_data.game_state = GameState::Play;
        self.game_data.player_turn = 1;
    }
    ///reset the data to replay the game
    fn reset(&mut self) {
        self.game_data.vec_cards = GameData::prepare_for_empty();
        self.game_data.count_click_inside_one_turn = 0;
        self.game_data.card_index_of_first_click = 0;
        self.game_data.card_index_of_second_click = 0;
        self.game_data.players.clear();
        self.game_data.game_state = GameState::Start;
        self.game_data.content_folder_name = "alphabet".to_string();
        self.game_data.asked_folder_name = "".to_string();
        self.game_data.my_player_number = 1;
        self.game_data.player_turn = 0;
        self.game_data.game_config = None;

        self.check_invalidate_for_all_components();
    }
    //region: all functions for receive message (like events)
    // I separate the code into functions to avoid looking at all that boilerplate in the big match around futures and components.
    // All the data changing must be encapsulated inside these functions.
    ///msg response with uid, just to check. because the websocket server
    ///gets the uid from the client in the url_param. The client generates a random number.
    fn on_response_ws_uid(&mut self, your_ws_uid: usize) {
        if self.game_data.my_ws_uid != your_ws_uid {
            self.game_data.error_text = "my_ws_uid is incorrect!".to_string();
        }
    }
    ///msg want to play
    fn on_want_to_play(&mut self, my_ws_uid: usize, asked_folder_name: String) {
        console::log_1(&"rcv wanttoplay".into());
        self.reset();
        self.game_data.game_state = GameState::Asked;
        //the first player is the initiator
        self.game_data.players.push(Player {
            ws_uid: my_ws_uid,
            points: 0,
        });
        self.game_data.players.push(Player {
            ws_uid: self.game_data.my_ws_uid,
            points: 0,
        });
        self.game_data.my_player_number = 2; //temporary number
        self.game_data.asked_folder_name = asked_folder_name;
    }
    ///msg accept play
    fn on_accept_play(&mut self, my_ws_uid: usize) {
        if self.game_data.my_player_number == 1 {
            self.game_data.players.push(Player {
                ws_uid: my_ws_uid,
                points: 0,
            });
            self.check_invalidate_for_all_components();
        }
    }
    ///on game data init
    fn on_game_data_init(&mut self, card_grid_data: &str, game_config: &str, players: &str) {
        self.game_data.content_folder_name = self.game_data.asked_folder_name.clone();
        self.game_data.game_state = GameState::Play;
        self.game_data.player_turn = 1;
        self.game_data.vec_cards = unwrap!(
            serde_json::from_str(card_grid_data),
            "error serde_json::from_str(card_grid_data)"
        );

        self.game_data.game_config = unwrap!(
            serde_json::from_str(game_config),
            "error serde_json::from_str(game_config)"
        );

        self.game_data.players = unwrap!(
            serde_json::from_str(players),
            "error serde_json::from_str(players)"
        );

        self.game_data.game_state = GameState::Play;
        //find my player number
        for index in 0..self.game_data.players.len() {
            if unwrap!(
                self.game_data.players.get_mut(index),
                "self.game_data.players.get_mut(index)"
            )
            .ws_uid
                == self.game_data.my_ws_uid
            {
                self.game_data.my_player_number = unwrap!(index.checked_add(1));
            }
        }
        self.check_invalidate_for_all_components();
    }
    ///msg end game
    fn on_end_game(&mut self) {
        self.game_data.game_state = GameState::EndGame;
    }
    ///msg response game_config json
    fn on_response_game_config_json(&mut self, json: &str) {
        self.game_data.game_config = unwrap!(
            serde_json::from_str(json),
            "error root_rendering_component.game_data.game_config = serde_json::from_str(&json)",
        );
    }
    ///msg player change
    fn on_player_change(&mut self) {
        self.take_turn();
    }
    ///msg player click
    fn on_player_click(&mut self, count_click_inside_one_turn: usize, card_index: usize) {
        self.game_data.count_click_inside_one_turn = count_click_inside_one_turn;
        if count_click_inside_one_turn == 1 {
            self.game_data.card_index_of_first_click = card_index;
        } else if count_click_inside_one_turn == 2 {
            self.game_data.card_index_of_second_click = card_index;
        } else {
            //nothing
        }
        self.card_on_click();
    }
    //endregion
}
//endregion

//region: `Render` trait implementation on RootRenderingComponent struct
///It is called for every Dodrio animation frame to render the vdom.
///Probably only when something changes. Here it is a click on the cards.
///Not sure about that, but I don't see a reason to make execute it otherwise.
///It is the only place where I create HTML elements in Virtual Dom.
impl Render for RootRenderingComponent {
    #[inline]
    fn render<'a, 'bump>(&'a self, bump: &'bump Bump) -> Node<'bump>
    where
        'a: 'bump,
    {
        //the card grid is a html css grid object (like a table) with <img> inside
        //other html elements are pretty simple.

        //region: private helper fn for Render()
        //here I use private functions for readability only, to avoid deep code nesting.
        //I don't understand closures enought to use them properly.
        //These private functions are not in the "impl Render forRootRenderingComponent" because of the error
        //method `from_card_number_to_img_src` is not a member of trait `Render`
        //there is not possible to write private and public methods in one impl block there are only pub methods.
        //`pub` not permitted there because it's implied
        //so I have to write functions outside of the impl block but inside my "module"

        ///grid width
        fn grid_width() -> usize {
            //the size of  the visible part of the window
            let window = unwrap!(web_sys::window(), "error: web_sys::window");

            let jsvalue_inner_width = unwrap!(window.inner_width(), "window.inner_width");

            let f64_inner_width = unwrap!(
                jsvalue_inner_width.as_f64(),
                "jsValue_inner_width.as_string()"
            );
            let usize_inner_width: usize = unwrap!(f64_inner_width.approx());
            //width min: 300px, max: 600 px in between width=visible width
            //3 columnsdelimiter 5px wide
            let grid_width: usize;
            if usize_inner_width < 300 {
                grid_width = 300;
            } else if usize_inner_width > 600 {
                grid_width = 600;
            } else {
                grid_width = usize_inner_width;
            }
            grid_width
        }
        ///grid height
        fn grid_height() -> usize {
            //the size of  the visible part of the window
            let window = unwrap!(web_sys::window(), "error: web_sys::window");
            let jsvalue_inner_height = unwrap!(window.inner_height(), "window.inner_height");

            let f64_inner_height = unwrap!(
                jsvalue_inner_height.as_f64(),
                "jsValue_inner_height.as_f64()"
            );
            let usize_inner_height: usize = unwrap!(f64_inner_height.approx());

            //height minimum 300, max 1000, else 0.8*visible height
            //3 row separetors 5px wide
            let grid_height: usize;
            if usize_inner_height < 300 {
                grid_height = 300;
            } else if usize_inner_height > 1000 {
                grid_height = 1000;
            } else {
                grid_height =
                    unwrap!((0.8 * (unwrap!(usize_inner_height.approx_as::<f64>())))
                        .approx_as::<usize>());
            }
            grid_height
        }
        ///prepare a vector<Node> for the Virtual Dom for 'css grid' item with <img>
        ///the grid container needs only grid items. There is no need for rows and columns in 'css grid'.
        fn div_grid_items<'a, 'bump>(
            root_rendering_component: &'a RootRenderingComponent,
            bump: &'bump Bump,
        ) -> Vec<Node<'bump>> {
            //this game_data mutable reference is dropped on the end of the function
            let game_data = &root_rendering_component.game_data;

            let mut vec_grid_item_bump: Vec<Node<'bump>> = Vec::new();
            if game_data.game_config.is_some() {
                //The format 4x4 was too small for the game with multiple smartphones on the table.
                //Now I can choose different sizes gx x gy
                //grid_width x grid_height is wh cards. index goes from PlayerNUmber-1*wh+1 to Player
                console::log_1(&JsValue::from_str(&format!(
                    "my_player_number {}",
                    &root_rendering_component.game_data.my_player_number
                )));

                //((game_data.my_player_number - 1) * grid_width*grid_height) + 1
                let start_index = unwrap!(unwrap!((unwrap!(game_data
                    .my_player_number
                    .checked_sub(1)))
                .checked_mul(unwrap!(unwrap!(game_data.game_config.as_ref())
                    .grid_items_hor
                    .checked_mul(unwrap!(game_data.game_config.as_ref()).grid_items_ver))))
                .checked_add(1));
                let end_index =
                    unwrap!(game_data
                        .my_player_number
                        .checked_mul(unwrap!(unwrap!(game_data.game_config.as_ref())
                            .grid_items_hor
                            .checked_mul(unwrap!(game_data.game_config.as_ref()).grid_items_ver))));
                for x in start_index..=end_index {
                    let index: usize = x;
                    //region: prepare variables and closures for inserting into vdom
                    let img_src = match unwrap!(
                    game_data.vec_cards.get(index),
                    "match game_data.vec_cards.get(index) {} startindex {} endindex {} vec_card.len {}",index,start_index, end_index,game_data.vec_cards.len()
                )
                .status
                {
                    CardStatusCardFace::Down => bumpalo::format!(in bump, "content/{}/{}",
                                                game_data.content_folder_name,
                                                SRC_FOR_CARD_FACE_DOWN)
                    .into_bump_str(),
                    CardStatusCardFace::UpTemporary | CardStatusCardFace::UpPermanently => {
                        bumpalo::format!(in bump, "content/{}/img/mem_image_{:02}.png",
                        game_data.content_folder_name,
                                unwrap!(game_data
                                    .vec_cards
                                    .get(index)
                                    ,"error index")
                                    .card_number_and_img_src
                        )
                        .into_bump_str()
                    }
                };

                    let img_id =
                    bumpalo::format!(in bump, "img{:02}",unwrap!(game_data.vec_cards.get(index),"game_data.vec_cards.get(index)").card_index_and_id)
                        .into_bump_str();

                    let opacity = if img_src
                        == format!(
                            "content/{}/{}",
                            game_data.content_folder_name, SRC_FOR_CARD_FACE_DOWN
                        ) {
                        bumpalo::format!(in bump, "opacity:{}", 0.2).into_bump_str()
                    } else {
                        bumpalo::format!(in bump, "opacity:{}", 1).into_bump_str()
                    };
                    //endregion

                    //creating grid_width*grid_height <div> in loop
                    let grid_item_bump = dodrio!(bump,
                    <div class= "grid_item">
                        <img src={img_src} id={img_id} style={opacity} onclick={move |root, vdom, event| {
                                //on click needs a code Closure in Rust. Dodrio and wasm-bindgen
                                //generate the javascript code to call it properly.
                                //we need our Struct RootRenderingComponent for Rust to write any data.
                                //It comes in the parameter root.
                                //All we can change is inside the struct RootRenderingComponent fields.
                                //The method render will later use that for rendering the new html.
                                let root_rendering_component =
                                    root.unwrap_mut::<RootRenderingComponent>();
                                //this game_data mutable reference is dropped on the end of the function
                                let mut game_data = &mut root_rendering_component.game_data;
                                if game_data.game_state.as_ref() == GameState::Play.as_ref() {
                                    // If the event's target is our image...
                                    let img = match event
                                        .target()
                                        .and_then(|t| t.dyn_into::<web_sys::HtmlImageElement>().ok())
                                    {
                                        None => return,
                                        //?? Don't understand what this does. The original was written for Input element.
                                        Some(input) => input,
                                    };

                                    //id attribute of image html element is prefixed with img ex. "img12"
                                    let this_click_card_index = unwrap!(
                                        (unwrap!(img.id().get(3..), "error slicing")).parse::<usize>(),
                                        "error parse img id to usize"
                                    );

                                    //click is usefull only od facedown cards
                                    if let CardStatusCardFace::Down = unwrap!(
                                        game_data.vec_cards.get(this_click_card_index),
                                        "error this_click_card_index"
                                    )
                                    .status
                                    {
                                        //the begining of the turn is count_click_inside_one_turn=0
                                        //on click imediately increase that. So first click is 1 and second click is 2.
                                        //all other clicks on the grid are not usable.
                                        game_data.count_click_inside_one_turn += 1;

                                        if game_data.count_click_inside_one_turn == 1 {
                                            game_data.card_index_of_first_click = this_click_card_index;
                                            game_data.card_index_of_second_click = 0;
                                        } else if game_data.count_click_inside_one_turn == 2 {
                                            game_data.card_index_of_second_click =
                                                this_click_card_index;
                                        } else {
                                            //nothing
                                        }

                                        //region: send WsMessage over websocket
                                        unwrap!(
                                            game_data.ws.send_with_str(
                                                &serde_json::to_string(&WsMessage::PlayerClick {
                                                    my_ws_uid: game_data.my_ws_uid,
                                                    players: unwrap!(
                                                        serde_json::to_string(&game_data.players),
                                                        "serde_json::to_string(&game_data.players)",
                                                    ),
                                                    card_index: this_click_card_index,
                                                    count_click_inside_one_turn: game_data
                                                        .count_click_inside_one_turn,
                                                })
                                                .expect("error sending PlayerClick"),
                                            ),
                                            "Failed to send PlayerClick"
                                        );
                                        //endregion

                                        //region: audio play
                                        if game_data.count_click_inside_one_turn == 1
                                            || game_data.count_click_inside_one_turn == 2
                                        {
                                            //prepare the audio element with src filename of mp3
                                            let audio_element = web_sys::HtmlAudioElement::new_with_src(
                                                format!(
                                                    "content/{}/sound/mem_sound_{:02}.mp3",
                                                    game_data.content_folder_name,
                                                    unwrap!(
                                                        game_data.vec_cards.get(this_click_card_index),
                                                        "error this_click_card_index"
                                                    )
                                                    .card_number_and_img_src
                                                )
                                                .as_str(),
                                            );

                                            //play() return a Promise in JSValue. That is too hard for me to deal with now.
                                            unwrap!(
                                                unwrap!(audio_element, "Error: HtmlAudioElement new.")
                                                    .play(),
                                                "Error: HtmlAudioElement.play() "
                                            );
                                        }
                                        //endregion

                                        root_rendering_component.card_on_click();
                                    }
                                    // Finally, re-render the component on the next animation frame.
                                    vdom.schedule_render();
                                }
                            }}>
                        </img>
                    </div>
                    );
                    vec_grid_item_bump.push(grid_item_bump);
                }
            }

            //return
            vec_grid_item_bump
        }

        ///the header can show only the game title or two spellings. Not everything together.
        fn div_grid_header<'a>(
            root_rendering_component: &'a RootRenderingComponent,
            bump: &'a Bump,
        ) -> Node<'a> {
            //this game_data mutable reference is dropped on the end of the function
            let game_data = &root_rendering_component.game_data;
            //if the Spellings are visible, than don't show GameTitle, because there is not
            //enought space on smartphones
            if game_data.card_index_of_first_click != 0 || game_data.card_index_of_second_click != 0
            {
                //if the two opened card match use green else use red color
                let color; //haha variable does not need to be mutable. Great !

                if unwrap!(
                    game_data.vec_cards.get(game_data.card_index_of_first_click),
                    "error index"
                )
                .card_number_and_img_src
                    == unwrap!(
                        game_data
                            .vec_cards
                            .get(game_data.card_index_of_second_click),
                        "error index"
                    )
                    .card_number_and_img_src
                {
                    color = "green";
                } else if game_data.card_index_of_first_click == 0
                    || game_data.card_index_of_second_click == 0
                {
                    color = "yellow";
                } else {
                    color = "red";
                }

                {
                    //return
                    dodrio!(bump,
                    <div class= "grid_container_header" style={bumpalo::format!(in bump, "grid-template-columns: auto auto; color:{}",color).into_bump_str()}>
                        <div class= "grid_item" style= "text-align: left;">
                            {vec![text(
                            bumpalo::format!(in bump, "{}",
                            unwrap!(unwrap!(root_rendering_component.game_data.game_config.clone(),"root_rendering_component.game_data.game_config.clone()")
                            .spelling.get(unwrap!(game_data.vec_cards.get(game_data.card_index_of_first_click),"game_data.vec_cards.get(game_data.card_index_of_first_click")
                                                    .card_number_and_img_src),".card_number_and_img_src")
                                                    )
                            .into_bump_str(),
                            )]}
                            </div>
                            <div class= "grid_item" style= "text-align: right;">
                                {vec![text(
                                bumpalo::format!(in bump, "{}",
                                unwrap!(unwrap!(root_rendering_component.game_data.game_config.clone(),"root_rendering_component.game_data.game_config.clone()")
                                .spelling.get(unwrap!(game_data.vec_cards.get(game_data.card_index_of_second_click)
                                ,"game_data.card_index_of_second_click)")
                                    .card_number_and_img_src),".card_number_and_img_src)")
                                    )
                            .into_bump_str(),
                            )]}
                            </div>
                        </div>
                        )
                }
            } else {
                {
                    dodrio!(bump,
                    <div class= "grid_container_header" style= "grid-template-columns: auto;">
                        <div class= "grid_item" style= "text-align: center;">
                            {vec![text(GAME_TITLE)]}
                        </div>
                    </div>
                    )
                }
            }
        }
        ///render ask to play for multiple contents/folders
        fn ask_to_play<'a, 'bump>(
            root_rendering_component: &'a RootRenderingComponent,
            bump: &'bump Bump,
            invite_string: &str,
        ) -> Node<'bump>
        where
            'a: 'bump,
        {
            let mut vec_of_nodes = Vec::new();
            //I don't know how to solve the lifetime problems. So I just clone the small data.
            let ff = root_rendering_component.game_data.content_folders.clone();
            for folder_name in ff {
                let folder_name_clone2 = folder_name.clone();
                vec_of_nodes.push(dodrio!(bump,
                <div class="div_clickable" onclick={move |root, vdom, _event| {
                        let root_rendering_component =
                            root.unwrap_mut::<RootRenderingComponent>();
                        //region: send WsMessage over websocket
                        root_rendering_component.game_data.my_player_number = 1;
                        root_rendering_component.game_data.players.clear();
                        root_rendering_component.game_data.players.push(Player {
                            ws_uid: root_rendering_component.game_data.my_ws_uid,
                            points: 0,
                        });
                        root_rendering_component.game_data.game_state = GameState::Asking;
                        root_rendering_component.game_data.asked_folder_name =
                            folder_name.clone();

                        //send request to Websocket server for game_configs (async over websocket messages)
                        unwrap!(
                            root_rendering_component.game_data.ws.send_with_str(
                                &serde_json::to_string(&WsMessage::RequestGameConfig {
                                    filename: format!(
                                        "content/{}/game_config.json",
                                        root_rendering_component.game_data.asked_folder_name
                                    ),
                                })
                                .expect("error sending RequestGameConfig"),
                            ),
                            "Failed to send RequestGameConfig"
                        );

                        unwrap!(
                            root_rendering_component.game_data.ws.send_with_str(
                                &serde_json::to_string(&WsMessage::WantToPlay {
                                    my_ws_uid: root_rendering_component.game_data.my_ws_uid,
                                    asked_folder_name: folder_name.clone(),
                                })
                                .expect("error sending WantToPlay"),
                            ),
                            "Failed to send WantToPlay"
                        );

                        //endregion
                        vdom.schedule_render();
                        }}>
                <h3 id= "ws_elem" style= "color:green;">
                        {vec![text(
                        //show Ask Player2 to Play!
                        bumpalo::format!(in bump, "{} for {} !", invite_string, folder_name_clone2)
                            .into_bump_str(),
                        )]}
                </h3>
                </div>
                ));
            }
            dodrio!(bump,
            <div>
                {vec_of_nodes}
            </div>
            )
        }

        ///html element to inform player what to do and get a click action from user
        fn div_game_status_and_player_actions<'a, 'bump>(
            root_rendering_component: &'a RootRenderingComponent,
            bump: &'bump Bump,
        ) -> Node<'bump>
        where
            'a: 'bump,
        {
            //0	CONNECTING, 1	OPEN, 2	CLOSING, 3	CLOSED
            #![allow(clippy::cognitive_complexity)]
            if !root_rendering_component.game_data.is_state_start()
                && (root_rendering_component.game_data.is_reconnect
                    || root_rendering_component.game_data.ws.ready_state() != 1)
            {
                //connection lost. Reconnect?
                dodrio!(bump,
                <div>
                    <div class="div_clickable" onclick={
                        move |root, vdom, _event| {
                        let root_rendering_component = root.unwrap_mut::<RootRenderingComponent>();
                        //the old ws and closures are now a memory leak, but small
                        let window = unwrap!(web_sys::window(), "error: web_sys::window");
                        let href = root_rendering_component.game_data.href.clone();
                        //usize is Copy(), so I don't need clone()
                        let my_ws_uid = root_rendering_component.game_data.my_ws_uid;
                        console::log_1(&JsValue::from_str(&format!(
                            "href {}  my_ws_uid {}",
                            href,
                            my_ws_uid,
                        )));
                        console::log_1(&"before reconnect".into());
                        let ws = setup_ws_connection(href, my_ws_uid);
                        setup_all_ws_events(&ws,vdom.clone());

                        root_rendering_component.game_data.ws=ws;
                        console::log_1(&"before game_data.is_reconnect = false and schedule_render".into());
                        root_rendering_component.game_data.is_reconnect = false;
                        vdom.schedule_render();
                    }}>
                        <h3 id= "ws_elem" style= "color:green;">
                            {vec![text(
                            //Reconnect?
                            bumpalo::format!(in bump, "Reconnect?{}", "").into_bump_str(),
                            )]}
                        </h3>
                    </div>
                    <h3 style= "color:red;">
                        {vec![text(
                            //connection lost
                            bumpalo::format!(in bump, "Connection lost.{}", "").into_bump_str(),
                        )]}
                    </h3>
                </div>
                )
            } else if let GameState::Start = root_rendering_component.game_data.game_state {
                // 1S Ask Player2 to play!
                console::log_1(&"GameState::Start".into());
                //return Ask Player2 to play!
                ask_to_play(root_rendering_component, bump, "Invite")
            } else if let GameState::EndGame = root_rendering_component.game_data.game_state {
                //end game ,Play again?  reload webpage
                dodrio!(bump,
                <div class="div_clickable" onclick={
                            move |root, vdom, _event| {
                            //reload the webpage
                            let window = unwrap!(web_sys::window(), "error: web_sys::window");
                            let x = window.location().reload();
                        }}>
                <h3 class= "m_container" id= "ws_elem" style= "color:green;">
                        {vec![text(
                            //Play again?
                            bumpalo::format!(in bump, "Play again{}?", "").into_bump_str(),
                        )]}
                </h3>
                </div>
                )
            } else if let GameState::Asking = root_rendering_component.game_data.game_state {
                //return wait for the other player
                dodrio!(bump,
                <div>
                    <div class="div_clickable" onclick={move |root, vdom, _event| {
                                let root_rendering_component =
                                    root.unwrap_mut::<RootRenderingComponent>();
                                //region: send WsMessage over websocket
                                root_rendering_component.game_data_init();

                                unwrap!(root_rendering_component
                                    .game_data
                                    .ws
                                    .send_with_str(
                                        &serde_json::to_string(&WsMessage::GameDataInit {
                    card_grid_data: unwrap!(serde_json::to_string(&root_rendering_component.game_data.vec_cards)
                                ,"serde_json::to_string(&self.game_data.vec_cards)"),
                    players: unwrap!(serde_json::to_string(&root_rendering_component.game_data.players)
                                ,"serde_json::to_string(&self.game_data.players)"),
                    game_config: unwrap!(serde_json::to_string(&root_rendering_component.game_data.game_config)
                                ,"serde_json::to_string(&self.game_data.game_config)"),
                            })
                            .expect("error sending WantToPlay"),
                        )
                        ,"Failed to send WantToPlay");

                    //endregion
                    vdom.schedule_render();
                    }}>
                        <h3 id="ws_elem" style= "color:green;">
                            {vec![
                                text(bumpalo::format!(in bump, "Start Game?{}", "").into_bump_str()),
                            ]}
                        </h3>
                    </div>
                    <div>
                        <h3 style= "color:red;">
                            {vec![
                                text(bumpalo::format!(in bump, "Players accepted: {}.", root_rendering_component.game_data.players.len()-1).into_bump_str()),
                            ]}
                        </h3>
                    </div>
                </div>
                )
            } else if let GameState::Accepted = root_rendering_component.game_data.game_state {
                console::log_1(&"GameState::Accepted".into());
                dodrio!(bump,
                <h3 id= "ws_elem" style= "color:red;">
                    {vec![text(bumpalo::format!(in bump, "Game {} accepted.", root_rendering_component.game_data.asked_folder_name).into_bump_str(),)]}
                </h3>
                )
            } else if let GameState::Asked = root_rendering_component.game_data.game_state {
                // 2S Click here to Accept play!
                console::log_1(&"GameState::Asked".into());
                //return Click here to Accept play
                dodrio!(bump,
                <div class="div_clickable" onclick={move |root, vdom, _event| {
                            let root_rendering_component =
                                root.unwrap_mut::<RootRenderingComponent>();
                            root_rendering_component.game_data.game_state=GameState::Accepted;

                            unwrap!(root_rendering_component
                                .game_data
                                .ws
                                .send_with_str(
                                    &serde_json::to_string(&WsMessage::AcceptPlay {
                                        my_ws_uid: root_rendering_component.game_data.my_ws_uid,
                                        players: unwrap!(serde_json::to_string(&root_rendering_component.game_data.players)
                                        ,"serde_json::to_string(&game_data.players)"),
                                    })
                                    .expect("error sending test"),
                                )
                                ,"Failed to send");
                            vdom.schedule_render();
                        }}>
                    <h3 id= "ws_elem" style= "color:green;">
                            {vec![text(
                                //show Ask Player2 to Play!
                                bumpalo::format!(in bump, "Click here to Accept {}!", root_rendering_component.game_data.asked_folder_name)
                                    .into_bump_str(),
                            )]}
                    </h3>
                </div>
                )
            } else if root_rendering_component
                .game_data
                .count_click_inside_one_turn
                >= 2
            {
                console::log_1(&JsValue::from_str(&format!(
                    "my_player_number {}",
                    &root_rendering_component.game_data.my_player_number
                )));
                console::log_1(&JsValue::from_str(&format!(
                    "player_turn {}",
                    &root_rendering_component.game_data.player_turn
                )));
                let next_player = if root_rendering_component.game_data.player_turn
                    < root_rendering_component.game_data.players.len()
                {
                    unwrap!(root_rendering_component
                        .game_data
                        .player_turn
                        .checked_add(1))
                } else {
                    1
                };
                if root_rendering_component.game_data.my_player_number == next_player {
                    dodrio!(bump,
                    <div class="div_clickable" onclick={move |root, vdom, _event| {
                                let root_rendering_component =
                                    root.unwrap_mut::<RootRenderingComponent>();
                                //this game_data mutable reference is dropped on the end of the function
                                //region: send WsMessage over websocket
                                unwrap!(root_rendering_component
                                    .game_data
                                    .ws
                                    .send_with_str(
                                        &serde_json::to_string(&WsMessage::PlayerChange {
                                            my_ws_uid: root_rendering_component.game_data.my_ws_uid,
                                            players: unwrap!(serde_json::to_string(
                                                &root_rendering_component.game_data.players,
                                            )
                                            ,"serde_json::to_string(&root_rendering_component.game_data.players)"),
                                        })
                                        .expect("error sending PlayerChange"),
                                    )
                                    ,"Failed to send PlayerChange");
                                //endregion
                                root_rendering_component.take_turn();
                                // Finally, re-render the component on the next animation frame.
                                vdom.schedule_render();
                            }}>
                        <h3 id= "ws_elem" style= "color:green;">
                            {vec![text(
                                bumpalo::format!(in bump, "Click here to take your turn !{}", "")
                                    .into_bump_str(),
                            )]}
                        </h3>
                    </div>
                    )
                } else {
                    //return wait for the other player
                    div_wait_for_other_player(bump, next_player)
                }
            } else if root_rendering_component
                .game_data
                .count_click_inside_one_turn
                < 2
            {
                if root_rendering_component.game_data.my_player_number
                    == root_rendering_component.game_data.player_turn
                {
                    dodrio!(bump,
                    <div class="div_clickable">
                        <h3 id= "ws_elem" style= "color:orange;">
                            {vec![text(bumpalo::format!(in bump, "Play player{} !", root_rendering_component.game_data.player_turn).into_bump_str())]}
                        </h3>
                    </div>
                    )
                } else {
                    //return wait for the other player
                    div_wait_for_other_player(bump, root_rendering_component.game_data.player_turn)
                }
            } else {
                //unpredictable situation
                //return
                dodrio!(bump,
                <h3 id= "ws_elem">
                    {vec![text(bumpalo::format!(in bump, "gamestate: {} player {}", root_rendering_component.game_data.game_state.as_ref(),root_rendering_component.game_data.my_player_number).into_bump_str())]}
                </h3>
                )
            }
        }
        ///the text 'wait for other player' is used multiple times
        fn div_wait_for_other_player(bump: &Bump, other_player: usize) -> Node {
            dodrio!(bump,
            <h3 id="ws_elem" style= "color:red;">
                {vec![text(bumpalo::format!(in bump, "Wait for player{} !", other_player).into_bump_str())]}
            </h3>
            )
        }
        ///calculate max with and height for a grid
        fn max_grid_size(root_rendering_component: &RootRenderingComponent) -> Size2d {
            //if the game_config is None, then return default
            if root_rendering_component.game_data.game_config.is_none() {
                Size2d { hor: 500, ver: 500 }
            } else {
                //grid_container width and height
                let mut max_grid_width = grid_width();
                let mut max_grid_height = grid_height();
                console::log_1(&JsValue::from_str(&format!(
                    "inner_width {} inner_height {}",
                    max_grid_width, max_grid_height
                )));
                //default if not choosen
                let mut card_width = 115;
                let mut card_height = 115;
                match &root_rendering_component.game_data.game_config {
                    None => (),
                    Some(_x) => {
                        card_width =
                            unwrap!(root_rendering_component.game_data.game_config.clone())
                                .card_width;
                        card_height =
                            unwrap!(root_rendering_component.game_data.game_config.clone())
                                .card_height;
                    }
                }
                console::log_1(&JsValue::from_str(&format!(
                    "card_width {} card_height {}",
                    card_width, card_height
                )));
                //ratio between width and height must stay the same
                let ratio = (unwrap!(card_height.approx_as::<f64>())
                    * unwrap!(
                        unwrap!(root_rendering_component.game_data.game_config.as_ref())
                            .grid_items_ver
                            .approx_as::<f64>()
                    ))
                    / (unwrap!(card_width.approx_as::<f64>())
                        * unwrap!(unwrap!(root_rendering_component
                            .game_data
                            .game_config
                            .as_ref())
                        .grid_items_hor
                        .approx_as::<f64>()));

                if unwrap!(max_grid_width.approx_as::<f64>()) * ratio
                    > unwrap!(max_grid_height.approx_as::<f64>())
                {
                    max_grid_width =
                        unwrap!((unwrap!(max_grid_height.approx_as::<f64>()) / ratio)
                            .approx_as::<usize>());
                } else {
                    max_grid_height =
                        unwrap!((unwrap!(max_grid_width.approx_as::<f64>()) * ratio)
                            .approx_as::<usize>());
                }
                console::log_1(&JsValue::from_str(&format!(
                    "max_grid_width {} max_grid_height {}",
                    max_grid_width, max_grid_height
                )));

                //return
                Size2d {
                    hor: max_grid_width,
                    ver: max_grid_height,
                }
            }
        }
        ///prepare the grid container
        fn div_grid_container<'a, 'bump>(
            root_rendering_component: &'a RootRenderingComponent,
            bump: &'bump Bump,
            xmax_grid_size: &Size2d,
        ) -> Node<'bump>
        where
            'a: 'bump,
        {
            //if the game config is none return empty <div>
            //or the game state is one, that does not render grid container
            if root_rendering_component.game_data.game_config.is_none()
                || !root_rendering_component
                    .game_data
                    .is_state_for_grid_container()
            {
                //return
                dodrio!(bump,
                    <div>
                    </div>
                )
            } else {
                let xstyle = format!(
                    "width:{}px; height:{}px;grid-template-columns: {} {} {} {};",
                    xmax_grid_size.hor,
                    xmax_grid_size.ver,
                    if unwrap!(root_rendering_component.game_data.game_config.as_ref())
                        .grid_items_hor
                        >= 1
                    {
                        "auto"
                    } else {
                        ""
                    },
                    if unwrap!(root_rendering_component.game_data.game_config.as_ref())
                        .grid_items_hor
                        >= 2
                    {
                        "auto"
                    } else {
                        ""
                    },
                    if unwrap!(root_rendering_component.game_data.game_config.as_ref())
                        .grid_items_hor
                        >= 3
                    {
                        "auto"
                    } else {
                        ""
                    },
                    if unwrap!(root_rendering_component.game_data.game_config.as_ref())
                        .grid_items_hor
                        >= 4
                    {
                        "auto"
                    } else {
                        ""
                    },
                );
                let xgrid_container = dodrio!(bump,
                    <div class= "grid_container" style={xstyle}>
                        {div_grid_items(root_rendering_component, bump)}
                    </div>
                );
                //return
                xgrid_container
            }
        }
        //endregion

        //region: create the whole virtual dom. The verbose stuff is in private functions

        if self.game_data.error_text == "" {
            let xmax_grid_size = max_grid_size(self);
            let xstyle2 = format!("width:{}px;", unwrap!(xmax_grid_size.hor.checked_add(2)));

            dodrio!(bump,
            <div class= "m_container" style={xstyle2}>
                {vec![div_grid_container(self,bump,&xmax_grid_size)]}
                {vec![div_game_status_and_player_actions(self, bump)]}
                {vec![div_grid_header(self, bump)]}
                {vec![self.players_and_scores.render(bump)]}
                {vec![self.cached_rules_and_description.render(bump)]}
            </div>
            )
        } else {
            //render only the error text to the screen.
            //because I want to debug the websocket lost connection
            dodrio!(bump,
                <div>
                    <h1 style= "color:red;" >
                        {vec![text(
                            bumpalo::format!(in bump, "error_text {} !", self.game_data.error_text)
                                .into_bump_str(),
                            )]}
                    </h1>
                </div>
            )
        }
        //endregion
    }
}
//endregion
