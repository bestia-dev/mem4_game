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
    clippy::cargo_common_metadata,
    clippy::multiple_crate_versions,
    clippy::wildcard_dependencies,
    //Rust is more idiomatic without return statement
    clippy::implicit_return,
    //I have private function inside a function. Self does not work there.
    clippy::use_self,
    //Cannot add #[inline] to the start function with #[wasm_bindgen(start)]
    //because then wasm-pack build --target web returns an error: export `run` not found 
    clippy::missing_inline_in_public_items,
    clippy::integer_arithmetic,
)]
//endregion

//region: extern and use statements
mod gamedata;
mod playersandscores;
mod rulesanddescription;
mod websocketcommunication;
use crate::gamedata::{CardStatusCardFace, GameData, GameState};
use crate::playersandscores::PlayersAndScores;
use crate::rulesanddescription::RulesAndDescription;
use crate::websocketcommunication::setup_ws_connection;
use crate::websocketcommunication::setup_ws_msg_recv;

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

use dodrio::builder::*;
use dodrio::bumpalo::{self, Bump};
use dodrio::{Cached, Node, Render};

use mem4_common::{Player, WsMessage};
use rand::rngs::SmallRng;
use rand::FromEntropy;
use rand::Rng;
use wasm_bindgen::prelude::{wasm_bindgen, JsValue, UnwrapThrowExt};
use wasm_bindgen::JsCast;
use web_sys::{console, WebSocket};
#[macro_use]
extern crate unwrap;
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
    //gen_range is lower inclusive, upper exclusive 26 + 1
    let my_ws_uid: usize = rng.gen_range(1, 9999);

    //find out URL
    let location_href = unwrap!(window.location().href(), "href not known");

    //websocket connection
    let ws = setup_ws_connection(location_href.as_str());
    //I don't know why is needed to clone the websocket connection
    let ws_c = ws.clone();

    // Construct a new `RootRenderingComponent`.
    //I added ws_c so that I can send messages on websocket

    let root_rendering_component = RootRenderingComponent::new(ws_c, my_ws_uid);

    // Mount the component to the `<div id="div_for_virtual_dom">`.
    let vdom = dodrio::Vdom::new(&div_for_virtual_dom, root_rendering_component);

    //websocket on receive message callback
    setup_ws_msg_recv(&ws, &vdom);

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
                            .get_mut(self.game_data.player_turn - 1),
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
                    if self.game_data.vec_cards.len() / 2 == point_sum {
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
            self.game_data.player_turn + 1
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
        self.game_data.count_all_clicks = 0;
        self.game_data.players.clear();
        self.game_data.game_state = GameState::Start;
        self.game_data.content_folder_name = "alphabet".to_string();
        self.game_data.my_player_number = 1;
        self.game_data.player_turn = 0;
        self.game_data.spelling = None;

        self.check_invalidate_for_all_components();
    }
    //region: all functions for receive message (like events)
    // I separate the code into functions to avoid looking at all that boilerplate in the big match around futures and components.
    // All the data changing must be encapsulated inside these functions.
    ///msg response we uid
    fn on_response_ws_uid(&mut self, your_ws_uid: usize) {
        self.game_data.my_ws_uid = your_ws_uid;
    }
    ///msg want to play
    fn on_want_to_play(&mut self, my_ws_uid: usize, content_folder_name: String) {
        console::log_1(&"rcv wanttoplay".into());
        self.reset();
        self.game_data.game_state = GameState::Asked;
        self.game_data.players.push(Player {
            ws_uid: my_ws_uid,
            points: 0,
        }); //the first player is the initiator
        self.game_data.players.push(Player {
            ws_uid: self.game_data.my_ws_uid,
            points: 0,
        });
        self.game_data.my_player_number = 2; //temporary number
        self.game_data.content_folder_name = content_folder_name;
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
    fn on_game_data_init(&mut self, card_grid_data: &str, spelling: &str, players: &str) {
        self.game_data.game_state = GameState::Play;
        self.game_data.player_turn = 1;
        self.game_data.vec_cards = unwrap!(
            serde_json::from_str(card_grid_data),
            "error serde_json::from_str(card_grid_data)"
        );

        self.game_data.spelling = unwrap!(
            serde_json::from_str(spelling),
            "error serde_json::from_str(spelling)"
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
                self.game_data.my_player_number = index + 1;
            }
        }
        self.check_invalidate_for_all_components();
    }
    ///msg end game
    fn on_end_game(&mut self) {
        self.game_data.game_state = GameState::EndGame;
    }
    ///msg response spelling json
    fn on_response_spelling_json(&mut self, json: &str) {
        self.game_data.spelling = unwrap!(
            serde_json::from_str(json),
            "error root_rendering_component.game_data.spelling = serde_json::from_str(&json)",
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

        ///prepare a vector<Node> for the Virtual Dom for 'css grid' item with <img>
        ///the grid container needs only grid items. There is no need for rows and columns in 'css grid'.
        fn div_grid_items<'a, 'bump>(
            root_rendering_component: &'a RootRenderingComponent,
            bump: &'bump Bump,
        ) -> Vec<Node<'bump>> {
            use dodrio::builder::*;
            //this game_data mutable reference is dropped on the end of the function
            let game_data = &root_rendering_component.game_data;

            let mut vec_grid_item_bump = Vec::new();
            //4x4 is 16 cards. index goes from PlayerNUmber-1*16+1 to Player
            console::log_1(&JsValue::from_str(&format!(
                "my_player_number {}",
                &root_rendering_component.game_data.my_player_number
            )));

            let start_index = ((game_data.my_player_number - 1) * 16) + 1;
            let end_index = game_data.my_player_number * 16;
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

                //creating 16 <div> in loop
                let grid_item_bump = div(bump)
                    .attr("class", "grid_item")
                    .children([img(bump)
                        .attr("src", img_src)
                        .attr("id", img_id)
                        .attr("style", opacity)
                        //on click needs a code Closure in Rust. Dodrio and wasm-bindgen
                        //generate the javascript code to call it properly.
                        .on("click", move |root, vdom, event| {
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
                                        game_data.count_all_clicks += 1;
                                    } else if game_data.count_click_inside_one_turn == 2 {
                                        game_data.card_index_of_second_click =
                                            this_click_card_index;
                                        game_data.count_all_clicks += 1;
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
                        })
                        .finish()])
                    .finish();
                vec_grid_item_bump.push(grid_item_bump);
            }
            vec_grid_item_bump
        }

        ///the header can show only the game title or two spellings. Not everything together.
        fn div_grid_header<'a>(
            root_rendering_component: &'a RootRenderingComponent,
            bump: &'a Bump,
        ) -> Node<'a> {
            use dodrio::builder::*;
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
                    div(bump)
                .attr("class", "grid_container_header")
                .attr(
                    "style",
                    bumpalo::format!(in bump, "grid-template-columns: auto auto; color:{}",color)
                        .into_bump_str(),
                )
                .children([
                    div(bump)
                        .attr("class", "grid_item")
                        .attr("style", "text-align: left;")
                        .children([text(
bumpalo::format!(in bump, "{}",
 unwrap!(unwrap!(root_rendering_component.game_data.spelling.clone(),"root_rendering_component.game_data.spelling.clone()")
 .name.get(unwrap!(game_data.vec_cards.get(game_data.card_index_of_first_click),"game_data.vec_cards.get(game_data.card_index_of_first_click")
                                .card_number_and_img_src),".card_number_and_img_src")
)
                        .into_bump_str(),
                        )])
                        .finish(),
                    div(bump)
                        .attr("class", "grid_item")
                        .attr("style", "text-align: right;")
                        .children([text(
                            bumpalo::format!(in bump, "{}",
                            unwrap!(unwrap!(root_rendering_component.game_data.spelling.clone(),"root_rendering_component.game_data.spelling.clone()")
                            .name.get(unwrap!(game_data.vec_cards.get(game_data.card_index_of_second_click)
                            ,"game_data.card_index_of_second_click)")
                                .card_number_and_img_src),".card_number_and_img_src)")
                                )
                        .into_bump_str(),
                        )])
                        .finish(),
                ])
                .finish()
                }
            } else {
                {
                    div(bump)
                        .attr("class", "grid_container_header")
                        .attr("style", "grid-template-columns: auto;")
                        .children([div(bump)
                            .attr("class", "grid_item")
                            .attr("style", "text-align: center;")
                            .children([text(GAME_TITLE)])
                            .finish()])
                        .finish()
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
            let a = root_rendering_component.game_data.content_folders.clone();
            for folder_name in a {
                vec_of_nodes.push(
                    h3(bump)
                        .attr("id", "ws_elem")
                        .attr("style", "color:green;")
                        .children([text(
                            //show Ask Player2 to Play!
                            bumpalo::format!(in bump, "{} for {} !", invite_string, folder_name)
                                .into_bump_str(),
                        )])
                        .on("click", move |root, vdom, _event| {
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
                            root_rendering_component.game_data.content_folder_name =
                                folder_name.clone();

                            //send request to Websocket server for spellings (async over websocket messages)
                            unwrap!(
                                root_rendering_component.game_data.ws.send_with_str(
                                    &serde_json::to_string(&WsMessage::RequestSpelling {
                                        filename: format!(
                                            "content/{}/text.json",
                                            root_rendering_component.game_data.content_folder_name
                                        ),
                                    })
                                    .expect("error sending RequestSpelling"),
                                ),
                                "Failed to send RequestSpelling"
                            );

                            unwrap!(
                                root_rendering_component.game_data.ws.send_with_str(
                                    &serde_json::to_string(&WsMessage::WantToPlay {
                                        my_ws_uid: root_rendering_component.game_data.my_ws_uid,
                                        content_folder_name: folder_name.clone(),
                                    })
                                    .expect("error sending WantToPlay"),
                                ),
                                "Failed to send WantToPlay"
                            );

                            //endregion
                            vdom.schedule_render();
                        })
                        .finish(),
                );
            }
            div(bump).children(vec_of_nodes).finish()
        }

        ///html element to inform player what to do and get a click action from user
        fn div_game_status_and_player_actions<'a, 'bump>(
            root_rendering_component: &'a RootRenderingComponent,
            bump: &'bump Bump,
        ) -> Node<'bump>
        where
            'a: 'bump,
        {
            use dodrio::builder::{h3, text};
            if let GameState::Start = root_rendering_component.game_data.game_state {
                // 1S Ask Player2 to play!
                console::log_1(&"GameState::Start".into());
                //return Ask Player2 to play!
                ask_to_play(root_rendering_component, bump, "Invite")
            } else if let GameState::EndGame = root_rendering_component.game_data.game_state {
                //end game ,Play again?
                h3(bump)
                    .attr("id", "ws_elem")
                    .attr("style", "color:green;")
                    .children([text(
                        //Play again?
                        bumpalo::format!(in bump, "Play again{}?", "").into_bump_str(),
                    )])
                    .on("click", move |root, vdom, _event| {
                        let root_rendering_component = root.unwrap_mut::<RootRenderingComponent>();
                        root_rendering_component.reset();
                        root_rendering_component.game_data.game_state = GameState::Start;
                        vdom.schedule_render();
                    })
                    .finish()
            } else if let GameState::Asking = root_rendering_component.game_data.game_state {
                //return wait for the other player
                div(bump)
                    .children([
                         h3(bump)
                .attr("id", "ws_elem")
                .attr("style", "color:red;")
                .children([text(
                    bumpalo::format!(in bump, "Players accepted: {}. Start Game?", root_rendering_component.game_data.players.len()-1).into_bump_str(),
                )])
                 .on("click", move |root, vdom, _event| {
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
spelling: unwrap!(serde_json::to_string(&root_rendering_component.game_data.spelling)
                ,"serde_json::to_string(&self.game_data.spelling)"),
                                    })
                                    .expect("error sending WantToPlay"),
                                )
                                ,"Failed to send WantToPlay");

                            //endregion
                            vdom.schedule_render();
                        })
                .finish(),
                        ask_to_play(root_rendering_component, bump, "Reinvite"),
                    ])
                    .finish()
            } else if let GameState::Accepted = root_rendering_component.game_data.game_state {
                console::log_1(&"GameState::Accepted".into());
                h3(bump)
                    .attr("id", "ws_elem")
                    .attr("style", "color:red;")
                    .children([text(
                        bumpalo::format!(in bump, "Game {} accepted.", root_rendering_component.game_data.content_folder_name)
                            .into_bump_str(),
                    )])
                    .finish()
            } else if let GameState::Asked = root_rendering_component.game_data.game_state {
                // 2S Click here to Accept play!
                console::log_1(&"GameState::Asked".into());
                //return Click here to Accept play
                h3(bump)
                    .attr("id", "ws_elem")
                    .attr("style", "color:green;")
                    .children([text(
                        //show Ask Player2 to Play!
                        bumpalo::format!(in bump, "Click here to Accept {}!", root_rendering_component.game_data.content_folder_name)
                            .into_bump_str(),
                    )])
                    .on("click", move |root, vdom, _event| {
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
                    })
                    .finish()
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

                if root_rendering_component.game_data.my_player_number
                    == (if root_rendering_component.game_data.player_turn
                        < root_rendering_component.game_data.players.len()
                    {
                        root_rendering_component.game_data.player_turn + 1
                    } else {
                        1
                    })
                {
                    //return Click here to take your turn
                    h3(bump)
                        .attr("id", "ws_elem")
                        .attr("style", "color:green;")
                        .children([text(
                            bumpalo::format!(in bump, "Click here to take your turn !{}", "")
                                .into_bump_str(),
                        )])
                        .on("click", move |root, vdom, _event| {
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
                        })
                        .finish()
                } else {
                    //return wait for the other player
                    div_wait_for_other_player(bump)
                }
            } else if root_rendering_component
                .game_data
                .count_click_inside_one_turn
                < 2
            {
                if root_rendering_component.game_data.my_player_number
                    == root_rendering_component.game_data.player_turn
                {
                    h3(bump)
                        .attr("id", "ws_elem")
                        .attr("style", "color:orange;")
                        .children([text(
                            bumpalo::format!(in bump, "Play !{}", "").into_bump_str(),
                        )])
                        .finish()
                } else {
                    //return wait for the other player
                    div_wait_for_other_player(bump)
                }
            } else {
                //unpredictable situation
                //return
                h3(bump)
                    .attr("id", "ws_elem")
                    .children([text(
                        bumpalo::format!(in bump, "gamestate: {} player {}", root_rendering_component.game_data.game_state.as_ref(),root_rendering_component.game_data.my_player_number)
                            .into_bump_str(),
                    )])
                    .finish()
            }
        }
        ///the text 'wait for other player' is used multiple times
        fn div_wait_for_other_player(bump: &Bump) -> Node {
            h3(bump)
                .attr("id", "ws_elem")
                .attr("style", "color:red;")
                .children([text(
                    bumpalo::format!(in bump, "Wait for the other player.{}", "").into_bump_str(),
                )])
                .finish()
        }
        //endregion

        //region: create the whole virtual dom. The verbose stuff is in private functions

        div(bump)
            .attr("class", "m_container")
            .children([
                div_grid_header(self, bump),
                //div for the css grid object defined in css with <img> inside
                div(bump)
                    .attr("class", "grid_container")
                    .attr("style", "margin-left: auto;margin-right: auto;")
                    .children(div_grid_items(self, bump))
                    .finish(),
                self.players_and_scores.render(bump),
                div_game_status_and_player_actions(self, bump),
                h5(bump)
                    .children([text(
                        bumpalo::format!(in bump, "Count of Clicks: {}", self.game_data.count_all_clicks)
                            .into_bump_str(),
                    )])
                    .finish(),
                self.cached_rules_and_description.render(bump),
            ])
            .finish()
        //endregion
    }
}
//endregion
