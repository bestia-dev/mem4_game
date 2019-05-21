//! game data

use rand::rngs::SmallRng;
use rand::seq::SliceRandom;
use rand::FromEntropy;
use rand::Rng;
use strum_macros::AsRefStr;
use web_sys::WebSocket;

///Aviation Spelling
///the zero element is card face down or empty, alphabet begins with 01 : A
///TODO: read dynamically from json file. Now I know how to do it in javascript, but not in Rust.
#[derive(Serialize, Deserialize, Clone)]
pub struct Spelling {
    ///names of spelling
    pub name: Vec<String>,
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
    //First turn: Player1 clicks 2 times and opens 2 cards.
    //If cards match, Player1 receives one point and countinues: 2 click for 2 cards.
    //If not match: Player2 clicks the Change button to close opened cards.
    //Then starts the Player2 turn.
    ///count click inside one turn
    pub count_click_inside_one_turn: usize,
    ///card index of first click
    pub card_index_of_first_click: usize,
    ///card index of second click
    pub card_index_of_second_click: usize,
    ///counts only clicks that flip the card. The third click is not counted.
    pub count_all_clicks: usize,
    ///web socket. used it to send message onclick.
    pub ws: WebSocket,
    ///my ws client instance unique id. To not listen the echo to yourself.
    pub my_ws_uid: usize,
    ///other ws client instance unique id. To listen only to one accepted other player.
    pub other_ws_uid: usize,
    ///game state: Start,Asking,Asked,Player1,Player2
    pub game_state: GameState,
    ///content folder name
    pub content_folder_name: String,
    ///What player am I
    pub this_machine_player_number: usize,
    ///whose turn is now:  player 1 or 2
    pub player_turn: usize,
    ///player1 points
    pub player1_points: usize,
    ///player2 points
    pub player2_points: usize,
    ///content folders vector
    pub content_folders: Vec<String>,
    ///spellings
    pub spelling: Option<Spelling>,
}
impl GameData {
    ///prepare new random data
    pub fn prepare_random_data(&mut self) {
        //region: find 8 distinct random numbers between 1 and 26 for the alphabet cards
        //vec_of_random_numbers is 0 based
        let mut vec_of_random_numbers = Vec::new();
        let mut rng = SmallRng::from_entropy();
        let mut i = 0;
        while i < 8 {
            //gen_range is lower inclusive, upper exclusive 26 + 1
            let num: usize = rng.gen_range(1, 27);
            if vec_of_random_numbers.contains(&num) {
                //do nothing if the random number is repeated
                //debug!("random duplicate {} in {:?}", num, vec_of_random_numbers);
            } else {
                //push a pair of the same number
                vec_of_random_numbers.push(num);
                vec_of_random_numbers.push(num);
                i += 1;
            }
        }
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

        //create the 16 card and push to the vector
        for (index, random_number) in vec_of_random_numbers.iter().enumerate() {
            let new_card = Card {
                status: CardStatusCardFace::Down,
                //dereference random number from iterator
                card_number_and_img_src: *random_number,
                //card base index will be 1. 0 is reserved for FaceDown.
                card_index_and_id: index.checked_add(1).expect("usize overflow"),
            };
            vec_cards.push(new_card);
        }
        //endregion
        self.vec_cards = vec_cards;
    }
    ///asociated function: before Accept, there are not random numbers, just default cards.
    pub fn prepare_for_empty() -> Vec<Card> {
        //prepare 16 empty cards. The random is calculated only on AcceptPlay.
        let mut vec_cards = Vec::new();
        //I must prepare the 0 index, but then I don't use it ever.
        for i in 0..=16 {
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
        //return from constructor
        GameData {
            vec_cards: Self::prepare_for_empty(),
            count_click_inside_one_turn: 0,
            card_index_of_first_click: 0,
            card_index_of_second_click: 0,
            count_all_clicks: 0,
            ws,
            my_ws_uid,
            other_ws_uid: 0, //zero means not accepted yet
            game_state: GameState::Start,
            content_folder_name: "alphabet".to_string(),
            player1_points: 0,
            player2_points: 0,
            this_machine_player_number: 0, //unknown until WantToPlay+Accept
            player_turn: 0,
            content_folders: vec![
                String::from("alphabet"),
                String::from("animals"),
                String::from("negative"),
            ],
            spelling: None,
        }
    }
}
