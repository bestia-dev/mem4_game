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
//region: mod is used only in lib file. All the rest use `use crate`
mod cardmoniker;
mod gridcontainer;
mod playeractions;
mod playersandscores;
mod rulesanddescription;
mod websocketcommunication;
mod gamedata;

//endregion
use crate::gamedata::{CardStatusCardFace, GameData};

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


use dodrio::builder::{br, text};
use dodrio::bumpalo::{self, Bump};
use dodrio::{Cached, Node, Render};
use mem4_common::{Player, WsMessage,GameState};
use rand::rngs::SmallRng;
use rand::FromEntropy;
use rand::Rng;
use typed_html::dodrio;
use wasm_bindgen::prelude::*;
use web_sys::{console, WebSocket};
//endregion

//region: enum, structs, const,...

///Root Render Component: the card grid struct has all the needed data for play logic and rendering
pub struct RootRenderingComponent {
    ///game data will be inside of Root, but reference for all other RenderingComponents
    game_data: GameData,
    ///subComponent: score
    players_and_scores: Cached<playersandscores::PlayersAndScores>,
    ///subComponent: the static parts can be cached.
    /// I am not sure if a field in this struct is the best place to put it.
    cached_rules_and_description: Cached<rulesanddescription::RulesAndDescription>,
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
    let ws = websocketcommunication::setup_ws_connection(location_href.clone(), my_ws_uid);
    //I don't know why is needed to clone the websocket connection
    let ws_c = ws.clone();

    // Construct a new `RootRenderingComponent`.
    //I added ws_c so that I can send messages on websocket

    let mut root_rendering_component = RootRenderingComponent::new(ws_c, my_ws_uid);
    root_rendering_component.game_data.href = location_href;

    // Mount the component to the `<div id="div_for_virtual_dom">`.
    let vdom = dodrio::Vdom::new(&div_for_virtual_dom, root_rendering_component);

    websocketcommunication::setup_all_ws_events(&ws, vdom.weak());

    // Run the component forever. Forget to drop the memory.
    vdom.forget();

    Ok(())
}
//endregion

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

        let game_rule_01 = rulesanddescription::RulesAndDescription {};
        let cached_rules_and_description = Cached::new(game_rule_01);
        let players_and_scores = Cached::new(playersandscores::PlayersAndScores::new(my_ws_uid));

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
        let this_click_card_index = if self.game_data.game_state.as_ref()==GameState::PlayBefore1Card.as_ref() {
            self.game_data.card_index_of_first_click
        } else {
            self.game_data.card_index_of_second_click
        };

        if self.game_data.game_state.as_ref()==GameState::PlayBefore1Card.as_ref()
            || self.game_data.game_state.as_ref()==GameState::PlayBefore2Card.as_ref()
        {
            //flip the card up
            unwrap!(
                self.game_data.vec_cards.get_mut(this_click_card_index),
                "error this_click_card_index"
            )
            .status = CardStatusCardFace::UpTemporary;

            if self.game_data.game_state.as_ref()==GameState::PlayBefore2Card.as_ref() {
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
                    self.game_data.game_state=GameState::PlayBefore1Card;
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
        self.game_data.game_state = GameState::PlayBefore1Card;

        self.check_invalidate_for_all_components();
    }
    ///prepares the game data
    fn game_data_init(&mut self) {
        self.game_data.content_folder_name = self.game_data.asked_folder_name.clone();
        self.game_data.prepare_random_data();
        self.game_data.game_state = GameState::PlayBefore1Card;
        self.game_data.player_turn = 1;
    }
    ///reset the data to replay the game
    fn reset(&mut self) {
        self.game_data.vec_cards = GameData::prepare_for_empty();
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
        self.game_data.game_state = GameState::PlayBefore1Card;
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
    fn on_player_click(&mut self, game_state: GameState, card_index: usize) {
        self.game_data.game_state = game_state;
        if self.game_data.game_state.as_ref() == GameState::PlayBefore1Card.as_ref() {
            self.game_data.card_index_of_first_click = card_index;
        } else if self.game_data.game_state.as_ref() == GameState::PlayBefore2Card.as_ref()  {
            self.game_data.card_index_of_second_click = card_index;
        } else {
            //the last else for satisfing Clippy
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


        //region: create the whole virtual dom. The verbose stuff is in private functions

        if self.game_data.error_text == "" {
            let xmax_grid_size = gridcontainer::max_grid_size(self);
            let xstyle2 = format!("width:{}px;", unwrap!(xmax_grid_size.hor.checked_add(2)));

            dodrio!(bump,
            <div class= "m_container" style={xstyle2}>
                {vec![cardmoniker::div_grid_card_moniker(self, bump)]}
                {vec![gridcontainer::div_grid_container(self,bump,&xmax_grid_size)]}
                {vec![playeractions::div_player_actions_from_game_status(self, bump)]}
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
