//! rootrenderingcomponent.rs - renders the web page

//region: use, const
use crate::cardmoniker;
use crate::gamedata::{CardStatusCardFace, GameData};
use crate::gridcontainer;
use crate::playeractions;
use crate::playersandscores;
use crate::rulesanddescription;

use dodrio::builder::text;
use dodrio::bumpalo::{self, Bump};
use dodrio::{Cached, Node, Render};
use mem4_common::{GameStatus, Player, WsMessage};
use typed_html::dodrio;
use web_sys::{console, WebSocket};
//endregion

///Root Render Component: the card grid struct has all the needed data for play logic and rendering
pub struct RootRenderingComponent {
    ///game data will be inside of Root, but reference for all other RenderingComponents
    pub game_data: GameData,
    ///subComponent: score
    pub players_and_scores: Cached<playersandscores::PlayersAndScores>,
    ///subComponent: the static parts can be cached.
    /// I am not sure if a field in this struct is the best place to put it.
    pub cached_rules_and_description: Cached<rulesanddescription::RulesAndDescription>,
}

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
    pub fn check_invalidate_for_all_components(&mut self) {
        if self.players_and_scores.update_intern_cache(&self.game_data) {
            Cached::invalidate(&mut self.players_and_scores);
        }
    }
    ///on first click
    pub fn card_on_click_1_card(&mut self) {
        let this_click_card_index = self.game_data.card_index_of_first_click;
        //flip the card up
        unwrap!(
            self.game_data.vec_cards.get_mut(this_click_card_index),
            "error this_click_card_index"
        )
        .status = CardStatusCardFace::UpTemporary;
        self.check_invalidate_for_all_components();
    }

    ///on second click
    ///The onclick event passed by javascript executes all the logic
    ///and changes only the fields of the Card Grid struct.
    ///That stuct is the only permanent data storage for later render the virtual dom.
    pub fn card_on_click_2_card(&mut self) {
        let this_click_card_index = self.game_data.card_index_of_second_click;
        //flip the card up
        unwrap!(
            self.game_data.vec_cards.get_mut(this_click_card_index),
            "error this_click_card_index"
        )
        .status = CardStatusCardFace::UpTemporary;

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
            self.game_data.game_status = GameStatus::PlayBefore1Card;
            //if the sum of points is number of card/2, the game is over
            let mut point_sum = 0;
            for x in &self.game_data.players {
                point_sum += x.points;
            }
            if unwrap!(self.game_data.vec_cards.len().checked_div(2)) == point_sum {
                self.game_data.game_status = GameStatus::EndGame;
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
        //TODO: where is if cards don't match???
        self.check_invalidate_for_all_components();

    }
    ///fn on change for both click and we msg.
    pub fn take_turn(&mut self) {
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
        self.game_data.game_status = GameStatus::PlayBefore1Card;

        self.check_invalidate_for_all_components();
    }
    ///prepares the game data
    pub fn game_data_init(&mut self) {
        self.game_data.content_folder_name = self.game_data.asked_folder_name.clone();
        self.game_data.prepare_random_data();
        self.game_data.game_status = GameStatus::PlayBefore1Card;
        self.game_data.player_turn = 1;
    }
    ///reset the data to replay the game
    pub fn reset(&mut self) {
        self.game_data.vec_cards = GameData::prepare_for_empty();
        self.game_data.card_index_of_first_click = 0;
        self.game_data.card_index_of_second_click = 0;
        self.game_data.players.clear();
        self.game_data.game_status = GameStatus::WantToPlayAskBegin;
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
    pub fn on_response_ws_uid(&mut self, your_ws_uid: usize) {
        if self.game_data.my_ws_uid != your_ws_uid {
            self.game_data.error_text = "my_ws_uid is incorrect!".to_string();
        }
    }
    ///msg want to play
    pub fn on_msg_want_to_play(&mut self, my_ws_uid: usize, asked_folder_name: String) {
        console::log_1(&"rcv wanttoplay".into());
        self.reset();
        self.game_data.game_status = GameStatus::WantToPlayAsked;
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
    pub fn on_msg_play_accept(&mut self, my_ws_uid: usize) {
        if self.game_data.my_player_number == 1 {
            self.game_data.players.push(Player {
                ws_uid: my_ws_uid,
                points: 0,
            });
            self.check_invalidate_for_all_components();
        }
    }
    ///on game data init
    pub fn on_msg_game_data_init(
        &mut self,
        card_grid_data: &str,
        game_config: &str,
        players: &str,
    ) {
        self.game_data.content_folder_name = self.game_data.asked_folder_name.clone();
        self.game_data.game_status = GameStatus::PlayBefore1Card;
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
    pub fn on_end_game(&mut self) {
        self.game_data.game_status = GameStatus::EndGame;
    }
    ///msg response game_config json
    pub fn on_response_game_config_json(&mut self, json: &str) {
        self.game_data.game_config = unwrap!(
            serde_json::from_str(json),
            "error root_rendering_component.game_data.game_config = serde_json::from_str(&json)",
        );
    }
    ///msg player change
    pub fn on_player_change(&mut self) {
        self.take_turn();
    }
    ///msg player click
    pub fn on_player_click(&mut self, game_status: GameStatus, card_index: usize) {
        self.game_data.game_status = game_status;
        if self.game_data.game_status.as_ref() == GameStatus::PlayBefore1Card.as_ref() {
            self.game_data.card_index_of_first_click = card_index;
        } else if self.game_data.game_status.as_ref() == GameStatus::PlayBefore2Card.as_ref() {
            self.game_data.card_index_of_second_click = card_index;
        } else {
            panic!("this else must never be reached!");
        }
        if self.game_data.game_status.as_ref() == GameStatus::PlayBefore1Card.as_ref() {
            self.card_on_click_1_card();
        } else if self.game_data.game_status.as_ref() == GameStatus::PlayBefore2Card.as_ref() {
            self.card_on_click_2_card();
        } else {
            panic!("this else should never be reached");
        }
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
