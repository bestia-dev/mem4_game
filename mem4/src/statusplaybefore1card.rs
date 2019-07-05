//! statusplaybefore1card.rs - code flow from this status

//region: use
use crate::gamedata::CardStatusCardFace;
use crate::rootrenderingcomponent::RootRenderingComponent;
use crate::websocketcommunication;
use mem4_common::{GameStatus, WsMessage};
//div_grid_container is in divgridcontainer

/// on click
pub fn card_on_click_1_card(rrc: &mut RootRenderingComponent,this_click_card_index:usize) {
    rrc.game_data.card_index_of_first_click = this_click_card_index;
    //region: send WsMessage over WebSocket
    websocketcommunication::ws_send_msg(
        &rrc.game_data.ws,
        &WsMessage::PlayerClick1Card {
            my_ws_uid: rrc.game_data.my_ws_uid,
            players: unwrap!(
                serde_json::to_string(&rrc.game_data.players),
                "serde_json::to_string(&game_data.players)",
            ),
            card_index: rrc.game_data.card_index_of_first_click,
            game_status: rrc.game_data.game_status.clone(),
        },
    );
    //endregion
    card_click_1_card(rrc);
}

///on click
pub fn card_click_1_card(rrc: &mut RootRenderingComponent) {
    //flip the card up
    unwrap!(
        rrc.game_data
            .vec_cards
            .get_mut(rrc.game_data.card_index_of_first_click),
        "error this_click_card_index"
    )
    .status = CardStatusCardFace::UpTemporary;
    rrc.check_invalidate_for_all_components();
}

///msg player click
pub fn on_msg_player_click_1_card(
    rrc: &mut RootRenderingComponent,
    game_status: GameStatus,
    card_index: usize,
) {
    rrc.game_data.game_status = game_status;
    if rrc.game_data.game_status.as_ref() == GameStatus::PlayBefore1Card.as_ref() {
        rrc.game_data.card_index_of_first_click = card_index;
        card_click_1_card(rrc);
    } else {
        panic!("this else should never be reached");
    }
}