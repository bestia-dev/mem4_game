//! Learning to code Rust for a http + websocket server on the same port  
//! commons for mem4 wasm and server

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
    //clippy::missing_inline_in_public_items
)]
//endregion

//region: extern and use statements
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
//endregion

///`WsMessage` enum for websocket
#[derive(Serialize, Deserialize)]
pub enum WsMessage {
    ///Dummy
    Dummy {
        ///anything
        dummy: String,
    },
    ///Request websocket Uid - first message to WebSocket server
    RequestWsUid {
        ///anything
        test: String,
    },
    ///response from WebSocket server for first message
    ResponseWsUid {
        ///websocket Uid
        your_ws_uid: usize,
    },
    ///want to play
    WantToPlay {
        ///ws client instance unique id. To not listen the echo to yourself.
        my_ws_uid: usize,
        ///content folder name
        content_folder_name: String,
    },
    /// accept play
    AcceptPlay {
        ///ws client instance unique id. To not listen the echo to yourself.
        my_ws_uid: usize,
        ///player1 is the initiator of the game.
        player1_ws_uid: usize,
    },
    /// player1 initialize the game data ans sends it to all players
    /// I will send json string to not confuse the server with vectors
    GameDataInit{
        ///act is the action to take on the receiver
        card_grid_data: String,
        spelling:String,
        players_ws_uid:String,
    },
    ///player click
    PlayerClick {
        ///ws client instance unique id. To not listen the echo to yourself.
        my_ws_uid: usize,
        ///other player unique id. Used by the WebSocket server.
        players_ws_uid: String,
        ///card_index
        card_index: usize,
        ///count click inside one turn
        count_click_inside_one_turn: usize,
    },
    ///player change
    PlayerChange {
        ///ws client instance unique id. To not listen the echo to yourself.
        my_ws_uid: usize,
        ///other player unique id. Used by the WebSocket server.
        players_ws_uid: String,
    },
    ///end game
    EndGame {
        ///ws client instance unique id. To not listen the echo to yourself.
        my_ws_uid: usize,
        ///other player unique id. Used by the WebSocket server.
        players_ws_uid: String,
    },
    ///Request the spelling from the WebSocket server
    RequestSpelling {
        ///the file with the spelling
        filename: String,
    },
    ///Receive the spelling from the WebSocket server
    ResponseSpellingJson {
        ///the spelling from the server
        json: String,
    },
}
//endregion
