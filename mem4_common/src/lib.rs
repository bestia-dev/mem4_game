// region: lmake_readme include "readme.md" //! A
//! **mem4_common - commons for mem4 wasm and server**
//!
//! version: 19.9.9  
//! Look also at the workspace readme https://github.com/bestia-dev/mem4_game  
//!
//! ## mem4_common
//! Learning to code Rust for a http + WebSocket.  
//! Here are just the structures, that are in common between frontend and backend.  
//!
//!
//!
//!

// endregion: lmake_readme include "readme.md" //! A

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
    //clippy::use_self,
    //Cannot add #[inline] to the start function with #[wasm_bindgen(start)]
    //because then wasm-pack build --target no-modules returns an error: export `run` not found 
    clippy::missing_inline_in_public_items,
    //Why is this bad : Doc is good. rustc has a MISSING_DOCS allowed-by-default lint for public members, but has no way to enforce documentation of private items. This lint fixes that.
    clippy::doc_markdown,
)]
//endregion

//region: extern and use statements
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate strum_macros;

use strum_macros::{Display, AsRefStr};
//endregion

///`WsMessage` enum for WebSocket
#[derive(Serialize, Deserialize)]
pub enum WsMessage {
    ///Dummy
    Dummy {
        ///anything
        dummy: String,
    },
    ///Request WebSocket Uid - first message to WebSocket server
    RequestWsUid {
        ///anything
        test: String,
    },
    ///response from WebSocket server for first message
    ResponseWsUid {
        ///WebSocket Uid
        your_ws_uid: usize,
    },
    ///invite
    Invite {
        ///ws client instance unique id. To not listen the echo to yourself.
        my_ws_uid: usize,
        ///content folder name
        asked_folder_name: String,
    },
    /// accept play
    PlayAccept {
        ///ws client instance unique id. To not listen the echo to yourself.
        my_ws_uid: usize,
        ///json of vector of players
        players: String,
    },
    /// player1 initialize the game data and sends it to all players
    /// I will send json string to not confuse the server with vectors
    GameDataInit {
        ///vector of cards status
        card_grid_data: String,
        ///json of game_config
        game_config: String,
        ///json of vector of players
        players: String,
    },
    ///player click
    PlayerClick1stCard {
        ///this identifies the smartphone, but not the player-in-turn
        my_ws_uid: usize,
        ///all players
        players: String,
        ///vector of cards status
        card_grid_data: String,
        ///game status PlayerBefore1stCard or PlayerBefore2ndCard
        game_status: GameStatus,
        ///have to send all the state of the game
        card_index_of_first_click: usize,
        ///have to send all the state of the game
        card_index_of_second_click: usize,
    },
    ///player click
    PlayerClick2ndCard {
        ///this identifies the smartphone, but not the player-in-turn
        my_ws_uid: usize,
        ///all players
        players: String,
        ///vector of cards status
        card_grid_data: String,
        ///game status PlayerBefore1stCard or PlayerBefore2ndCard
        game_status: GameStatus,
        ///have to send all the state of the game
        card_index_of_first_click: usize,
        ///have to send all the state of the game
        card_index_of_second_click: usize,
    },
    ///take turn begin
    TakeTurnBegin {
        ///this identifies the smartphone, but not the player-in-turn
        my_ws_uid: usize,
        ///all players
        players: String,
        ///vector of cards status
        card_grid_data: String,
        ///game status PlayerBefore1stCard or PlayerBefore2ndCard
        game_status: GameStatus,
        ///have to send all the state of the game
        card_index_of_first_click: usize,
        ///have to send all the state of the game
        card_index_of_second_click: usize,
    },
    ///Play Again
    GameOverPlayAgainBegin {
        ///this identifies the smartphone, but not the player-in-turn
        my_ws_uid: usize,
        ///all players
        players: String,
        ///vector of cards status
        card_grid_data: String,
        ///game status PlayerBefore1stCard or PlayerBefore2ndCard
        game_status: GameStatus,
        ///have to send all the state of the game
        card_index_of_first_click: usize,
        ///have to send all the state of the game
        card_index_of_second_click: usize,
    },
    ///player change
    TakeTurnEnd {
        ///ws client instance unique id. To not listen the echo to yourself.
        my_ws_uid: usize,
        ///all players
        players: String,
    },
}

///the game can be in various statuses and that differentiate the UI and actions
/// all players have the same game status
#[derive(Display, AsRefStr, Serialize, Deserialize, Clone)]
pub enum GameStatus {
    /// invite ask begin
    InviteAskBegin,
    ///Player1 Invite Asking
    InviteAsking,
    ///Player2 Invite Asked
    InviteAsked,
    ///PlayAccepted
    PlayAccepted,
    ///Play before first card
    PlayBefore1stCard,
    ///Play before second card
    PlayBefore2ndCard,
    ///take turn begin
    TakeTurnBegin,
    ///take turn end
    TakeTurnEnd,
    ///end game
    GameOverPlayAgainBegin,
    ///Reconnect after a lost connection
    Reconnect,
}

///data for one player
#[derive(Serialize, Deserialize)]
pub struct Player {
    ///ws_uid
    pub ws_uid: usize,
    ///field for src attribute for HTML element image and filename of card image
    pub points: usize,
}
//endregion
