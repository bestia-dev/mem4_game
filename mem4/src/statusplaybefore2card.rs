//! statusplaybefore2card.rs - code flow from this status

//region: use
use crate::gamedata::CardStatusCardFace;
use crate::rootrenderingcomponent::RootRenderingComponent;
use crate::websocketcommunication;
use mem4_common::{GameStatus, WsMessage};
//endregion

///on click
pub fn card_on_click_2_card(rrc: &mut RootRenderingComponent, this_click_card_index: usize) {
    rrc.game_data.card_index_of_second_click = this_click_card_index;
    //region: send WsMessage over WebSocket
    websocketcommunication::ws_send_msg(
        &rrc.game_data.ws,
        &WsMessage::PlayerClick2Card {
            my_ws_uid: rrc.game_data.my_ws_uid,
            players: unwrap!(
                serde_json::to_string(&rrc.game_data.players),
                "serde_json::to_string(&game_data.players)",
            ),
            card_index: rrc.game_data.card_index_of_second_click,
            game_status: rrc.game_data.game_status.clone(),
        },
    );
    //endregion
    card_click_2_card(rrc);
}

///on second click
///The on click event passed by JavaScript executes all the logic
///and changes only the fields of the Card Grid struct.
///That struct is the only permanent data storage for later render the virtual dom.
pub fn card_click_2_card(rrc: &mut RootRenderingComponent) {
    //flip the card up
    unwrap!(
        rrc.game_data
            .vec_cards
            .get_mut(rrc.game_data.card_index_of_second_click),
        "error this_click_card_index"
    )
    .status = CardStatusCardFace::UpTemporary;

    //if the cards match, player get one point and continues another turn
    if unwrap!(
        rrc.game_data
            .vec_cards
            .get(rrc.game_data.card_index_of_first_click),
        "error game_data.card_index_of_first_click"
    )
    .card_number_and_img_src
        == unwrap!(
            rrc.game_data
                .vec_cards
                .get(rrc.game_data.card_index_of_second_click),
            "error game_data.card_index_of_second_click"
        )
        .card_number_and_img_src
    {
        //give points
        unwrap!(
            rrc.game_data
                .players
                .get_mut(unwrap!(rrc.game_data.player_turn.checked_sub(1))),
            "rrc.game_data.players.get_mu(rrc.game_data.player_turn - 1)"
        )
        .points += 1;

        // the two cards matches. make them permanent FaceUp
        let x1 = rrc.game_data.card_index_of_first_click;
        let x2 = rrc.game_data.card_index_of_second_click;
        unwrap!(
            rrc.game_data.vec_cards.get_mut(x1),
            "error game_data.card_index_of_first_click"
        )
        .status = CardStatusCardFace::UpPermanently;
        unwrap!(
            rrc.game_data.vec_cards.get_mut(x2),
            "error game_data.card_index_of_second_click"
        )
        .status = CardStatusCardFace::UpPermanently;
        rrc.game_data.game_status = GameStatus::PlayBefore1Card;
        //if the sum of points is number of card/2, the game is over
        let mut point_sum = 0;
        for x in &rrc.game_data.players {
            point_sum += x.points;
        }
        if unwrap!(rrc.game_data.vec_cards.len().checked_div(2)) == point_sum {
            rrc.game_data.game_status = GameStatus::PlayAgain;
            //send message
            unwrap!(
                rrc.game_data.ws.send_with_str(
                    &serde_json::to_string(&WsMessage::PlayAgain {
                        my_ws_uid: rrc.game_data.my_ws_uid,
                        players: unwrap!(
                            serde_json::to_string(&rrc.game_data.players),
                            "serde_json::to_string(&rrc.game_data.players)"
                        ),
                    })
                    .expect("error sending PlayAgain"),
                ),
                "Failed to send PlayAgain"
            );
        }

    }
    //TODO: where is if cards don't match???
    rrc.check_invalidate_for_all_components();
}
///msg player click
pub fn on_msg_player_click_2_card(
    rrc: &mut RootRenderingComponent,
    game_status: GameStatus,
    card_index: usize,
) {
    rrc.game_data.game_status = game_status;
    if rrc.game_data.game_status.as_ref() == GameStatus::PlayBefore2Card.as_ref() {
        rrc.game_data.card_index_of_second_click = card_index;
        card_click_2_card(rrc);
    } else {
        panic!("this else must never be reached!");
    }
}

///msg player click
pub fn on_msg_play_again(rrc: &mut RootRenderingComponent) {
    rrc.game_data.game_status = GameStatus::PlayAgain;
}
