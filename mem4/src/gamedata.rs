//! game data

extern crate mem4_common;

use mem4_common::Player;
use rand::rngs::SmallRng;
use rand::seq::SliceRandom;
use rand::FromEntropy;
use rand::Rng;
use strum_macros::AsRefStr;
use wasm_bindgen::prelude::*;
use web_sys::console;
use web_sys::WebSocket;

///Aviation Spelling
///the zero element is card face down or empty, alphabet begins with 01 : A
///TODO: read dynamically from json file. Now I know how to do it in javascript, but not in Rust.
#[derive(Serialize, Deserialize, Clone)]
pub struct Spelling {
    ///names of spelling
    pub name: Vec<String>,
    ///card image width
    pub card_width: usize,
    ///card image height
    pub card_height: usize,
}

///the game can be in various states and that differentiate the UI and actions
#[derive(AsRefStr)]
pub enum GameState {
    ///the start of the game
    Start,
    ///Player1 Asking WantToPlay
    Asking,
    ///Player2 is asked WantToPlay
    Asked,
    ///Accepted
    Accepted,
    ///play (the turn is in RootRenderingComponent.player_turn)
    Play,
    ///end game
    EndGame,
}
///the 3 possible states of one card
#[derive(Serialize, Deserialize)]
pub enum CardStatusCardFace {
    ///card face down
    Down,
    ///card face Up Temporary
    UpTemporary,
    ///card face up Permanently
    UpPermanently,
}

///all the data for one card
#[derive(Serialize, Deserialize)]
pub struct Card {
    ///card status
    pub status: CardStatusCardFace,
    ///field for src attribute for HTML element imagea and filename of card image
    pub card_number_and_img_src: usize,
    ///field for id attribute for HTML element image contains the card index
    pub card_index_and_id: usize,
}

///game data
pub struct GameData {
    ///vector of cards
    pub vec_cards: Vec<Card>,
    ///count click inside one turn
    pub count_click_inside_one_turn: usize,
    ///card index of first click
    pub card_index_of_first_click: usize,
    ///card index of second click
    pub card_index_of_second_click: usize,
    ///web socket. used it to send message onclick.
    pub ws: WebSocket,
    ///my ws client instance unique id. To not listen the echo to yourself.
    pub my_ws_uid: usize,
    ///players
    pub players: Vec<Player>,
    ///game state: Start,Asking,Asked,Player1,Player2
    pub game_state: GameState,
    ///content folder name
    pub content_folder_name: String,
    ///want to play asks for a specific game
    pub asked_folder_name: String,
    ///What player am I
    pub my_player_number: usize,
    ///whose turn is now:  player 1,2,3,...
    pub player_turn: usize,
    ///content folders vector
    pub content_folders: Vec<String>,
    ///spellings
    pub spelling: Option<Spelling>,
    ///error text
    pub error_text: String,
}
impl GameData {
    ///prepare new random data
    pub fn prepare_random_data(&mut self) {
        let spelling_count_minus_one =
            unwrap!(unwrap!(self.spelling.as_ref(), "self.spelling.as_ref()")
                .name
                .len()
                .checked_sub(1));
        let players_count = self.players.len();
        let cards_count = unwrap!(players_count.checked_mul(16));
        let random_count = unwrap!(cards_count.checked_div(2));
        //if the number of cards is bigger than the images, i choose all the images.
        //for the rest I use random.
        //integer division rounds toward zero
        let multiple: usize = unwrap!(random_count.checked_div(spelling_count_minus_one));
        let rest = unwrap!(
            random_count.checked_sub(unwrap!(spelling_count_minus_one.checked_mul(multiple)))
        );
        //region: find random numbers between 1 and spelling_count
        //vec_of_random_numbers is 0 based
        let mut vec_of_random_numbers = Vec::new();
        let mut rng = SmallRng::from_entropy();
        vec_of_random_numbers.clear();
        for _i in 1..=rest {
            //how to avoid duplicates
            let mut num: usize;
            // a do-while is written as a  loop-break
            loop {
                //gen_range is lower inclusive, upper exclusive 26 + 1
                num = rng.gen_range(1, unwrap!(spelling_count_minus_one.checked_add(1)));
                if !vec_of_random_numbers.contains(&num) {
                    break;
                }
            }
            //push a pair of the same number
            vec_of_random_numbers.push(num);
            vec_of_random_numbers.push(num);
        }
        for _m in 1..=multiple {
            for i in 1..=spelling_count_minus_one {
                vec_of_random_numbers.push(i);
                vec_of_random_numbers.push(i);
            }
        }
        console::log_1(&JsValue::from_str(&format!(
            "spelling_count {}  rest {}   vec.len {}",
            spelling_count_minus_one,
            rest,
            vec_of_random_numbers.len()
        )));
        //endregion

        //region: shuffle the numbers
        let vrndslice = vec_of_random_numbers.as_mut_slice();
        vrndslice.shuffle(&mut rng);
        //endregion

        //region: create Cards from random numbers
        let mut vec_cards = Vec::new();

        //Index 0 is special and reserved for FaceDown. Cards start with base 1
        let new_card = Card {
            status: CardStatusCardFace::Down,
            card_number_and_img_src: 0,
            card_index_and_id: 0,
        };
        vec_cards.push(new_card);

        //create cards and push to the vector
        for (index, random_number) in vec_of_random_numbers.iter().enumerate() {
            let new_card = Card {
                status: CardStatusCardFace::Down,
                //dereference random number from iterator
                card_number_and_img_src: *random_number,
                //card base index will be 1. 0 is reserved for FaceDown.
                card_index_and_id: unwrap!(index.checked_add(1), "usize overflow"),
            };
            vec_cards.push(new_card);
        }
        //endregion
        self.vec_cards = vec_cards;
    }
    ///asociated function: before Accept, there are not random numbers, just default cards.
    pub fn prepare_for_empty() -> Vec<Card> {
        //prepare 32 empty cards. The random is calculated only on AcceptPlay.
        let mut vec_cards = Vec::new();
        //I must prepare the 0 index, but then I don't use it ever.
        for i in 0..=32 {
            let new_card = Card {
                status: CardStatusCardFace::Down,
                card_number_and_img_src: 1,
                card_index_and_id: i,
            };
            vec_cards.push(new_card);
        }
        vec_cards
    }
    ///constructor of game data
    pub fn new(ws: WebSocket, my_ws_uid: usize) -> Self {
        let mut players = Vec::new();
        players.push(Player {
            ws_uid: 0,
            points: 0,
        });
        //return from constructor
        GameData {
            vec_cards: Self::prepare_for_empty(),
            count_click_inside_one_turn: 0,
            card_index_of_first_click: 0,
            card_index_of_second_click: 0,
            ws,
            my_ws_uid,
            players,
            game_state: GameState::Start,
            content_folder_name: "alphabet".to_string(),
            asked_folder_name: "".to_string(),
            my_player_number: 1,
            player_turn: 0,
            content_folders: vec![
                String::from("alphabet"),
                String::from("animals"),
                String::from("playingcards"),
                String::from("triestine"),
            ],
            spelling: None,
            error_text: "".to_string(),
        }
    }
}
