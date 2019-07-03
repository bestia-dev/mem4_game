//! playeractions - renders the div to inform player what to do next
//! and get a click action from the user

//region: use
use crate::rootrenderingcomponent::RootRenderingComponent;
use crate::websocketcommunication;

use dodrio::builder::text;
use dodrio::bumpalo::{self, Bump};
use dodrio::Node;
use mem4_common::{GameStatus, Player, WsMessage};
use typed_html::dodrio;
use wasm_bindgen::prelude::*;
use web_sys::console;
//endregion

///render html element to inform player what to do and get a click action from user
pub fn div_player_actions_from_game_status<'a, 'bump>(
    root_rendering_component: &'a RootRenderingComponent,
    bump: &'bump Bump,
) -> Node<'bump>
where
    'a: 'bump,
{
    if !root_rendering_component
        .game_data
        .is_status_want_to_play_ask_begin()
        && (root_rendering_component.game_data.is_reconnect
            || root_rendering_component.game_data.ws.ready_state() != 1)
    {
        //ready_state: 0	CONNECTING, 1	OPEN, 2	CLOSING, 3	CLOSED
        div_reconnect(root_rendering_component, bump)
    } else if let GameStatus::WantToPlayAskBegin = root_rendering_component.game_data.game_status {
        div_want_to_play_ask_begin(root_rendering_component, bump)
    } else if let GameStatus::EndGame = root_rendering_component.game_data.game_status {
        div_play_again(root_rendering_component, bump)
    } else if let GameStatus::WantToPlayAsking = root_rendering_component.game_data.game_status {
        div_want_to_play_asking(root_rendering_component, bump)
    } else if let GameStatus::PlayAccepted = root_rendering_component.game_data.game_status {
        div_play_accepted(root_rendering_component, bump)
    } else if let GameStatus::WantToPlayAsked = root_rendering_component.game_data.game_status {
        div_want_to_play_asked(root_rendering_component, bump)
    } else if let GameStatus::PlayBefore1Card = root_rendering_component.game_data.game_status {
        div_click_one(root_rendering_component, bump)
    } else if let GameStatus::PlayBefore2Card = root_rendering_component.game_data.game_status {
        div_take_turn(root_rendering_component, bump)
    } else {
        div_unpredicted(root_rendering_component, bump)
    }
}
///render reconnect
fn div_reconnect<'a, 'bump>(
    _root_rendering_component: &'a RootRenderingComponent,
    bump: &'bump Bump,
) -> Node<'bump>
where
    'a: 'bump,
{
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
            let ws = websocketcommunication::setup_ws_connection(href, my_ws_uid);
            websocketcommunication::setup_all_ws_events(&ws,vdom.clone());

            root_rendering_component.game_data.ws=ws;
            console::log_1(&"before game_data.is_reconnect = false and schedule_render".into());
            root_rendering_component.game_data.is_reconnect = false;
            vdom.schedule_render();
        }}>
            <h2 id= "ws_elem" style= "color:green;">
                {vec![text(
                //Reconnect?
                bumpalo::format!(in bump, "Reconnect?{}", "").into_bump_str(),
                )]}
            </h2>
        </div>
        <h2 style= "color:red;">
            {vec![text(
                //connection lost
                bumpalo::format!(in bump, "Connection lost.{}", "").into_bump_str(),
            )]}
        </h2>
    </div>
    )
}
///render want to play ask begin, ask to play for multiple contents/folders
fn div_want_to_play_ask_begin<'a, 'bump>(
    root_rendering_component: &'a RootRenderingComponent,
    bump: &'bump Bump,
) -> Node<'bump>
where
    'a: 'bump,
{
    console::log_1(&"GameStatus::WantToPlayAskBegin".into());
    let mut vec_of_nodes = Vec::new();
    //I don't know how to solve the lifetime problems. So I just clone the small data.
    let ff = root_rendering_component.game_data.content_folders.clone();
    for folder_name in ff {
        let folder_name_clone2 = folder_name.clone();
        vec_of_nodes.push(dodrio!(bump,
        <div class="div_clickable" onclick={move |root, vdom, _event| {
                let rrc = root.unwrap_mut::<RootRenderingComponent>();

                div_want_to_play_ask_begin_on_click(rrc, &folder_name);

                vdom.schedule_render();
                }}>
            <h2 id= "ws_elem" style= "color:green;">
                {vec![text(
                //show Ask Player2 to Play!
                bumpalo::format!(in bump, "Invite for {} !", folder_name_clone2)
                    .into_bump_str(),
                )]}
            </h2>
        </div>
        ));
    }
    dodrio!(bump,
    <div>
        {vec_of_nodes}
    </div>
    )
}
/// on click updates some data and sends msgs
/// msgs will be asynchronously received and processed
fn div_want_to_play_ask_begin_on_click(rrc: &mut RootRenderingComponent, folder_name: &str) {
    rrc.game_data.my_player_number = 1;
    rrc.game_data.players.clear();
    rrc.game_data.players.push(Player {
        ws_uid: rrc.game_data.my_ws_uid,
        points: 0,
    });
    rrc.game_data.game_status = GameStatus::WantToPlayAsking;
    rrc.game_data.asked_folder_name = folder_name.to_string();

    //send request to Websocket server for game_configs (async over websocket messages)
    websocketcommunication::ws_send_msg(
        &rrc.game_data.ws,
        &WsMessage::RequestGameConfig {
            filename: format!(
                "content/{}/game_config.json",
                rrc.game_data.asked_folder_name
            ),
        },
    );

    //send the msg WantToPlay
    websocketcommunication::ws_send_msg(
        &rrc.game_data.ws,
        &WsMessage::WantToPlay {
            my_ws_uid: rrc.game_data.my_ws_uid,
            asked_folder_name: folder_name.to_string(),
        },
    );
}

///play again
fn div_play_again<'a, 'bump>(
    _root_rendering_component: &'a RootRenderingComponent,
    bump: &'bump Bump,
) -> Node<'bump>
where
    'a: 'bump,
{
    //end game ,Play again?  reload webpage
    dodrio!(bump,
    <div class="div_clickable" onclick={
                move |root, vdom, _event| {
                //reload the webpage
                let window = unwrap!(web_sys::window(), "error: web_sys::window");
                let x = window.location().reload();
            }}>
    <h2 class= "m_container" id= "ws_elem" style= "color:green;">
            {vec![text(
                //Play again?
                bumpalo::format!(in bump, "Play again{}?", "").into_bump_str(),
            )]}
    </h2>
    </div>
    )
}
///render
fn div_want_to_play_asking<'a, 'bump>(
    root_rendering_component: &'a RootRenderingComponent,
    bump: &'bump Bump,
) -> Node<'bump>
where
    'a: 'bump,
{
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
            <h2 id="ws_elem" style= "color:green;">
                {vec![
                    text(bumpalo::format!(in bump, "Start Game?{}", "").into_bump_str()),
                ]}
            </h2>
        </div>
        <div>
            <h2 style= "color:red;">
                {vec![
                    text(bumpalo::format!(in bump, "Players accepted: {}.", root_rendering_component.game_data.players.len()-1).into_bump_str()),
                ]}
            </h2>
        </div>
    </div>
    )
}
///render play accepted
fn div_play_accepted<'a, 'bump>(
    root_rendering_component: &'a RootRenderingComponent,
    bump: &'bump Bump,
) -> Node<'bump>
where
    'a: 'bump,
{
    console::log_1(&"GameStatus::PlayAccepted".into());
    dodrio!(bump,
    <h2 id= "ws_elem" style= "color:red;">
        {vec![text(bumpalo::format!(in bump, "Game {} accepted.", root_rendering_component.game_data.asked_folder_name).into_bump_str(),)]}
    </h2>
    )
}
///render asked
fn div_want_to_play_asked<'a, 'bump>(
    root_rendering_component: &'a RootRenderingComponent,
    bump: &'bump Bump,
) -> Node<'bump>
where
    'a: 'bump,
{
    // 2S Click here to Accept play!
    console::log_1(&"GameStatus::WantToPlayAsked".into());
    //return Click here to Accept play
    dodrio!(bump,
    <div class="div_clickable" onclick={move |root, vdom, _event| {
                let rrc = root.unwrap_mut::<RootRenderingComponent>();

                div_want_to_play_asked_on_click(rrc);

                vdom.schedule_render();
            }}>
        <h2 id= "ws_elem" style= "color:green;">
                {vec![text(
                    //show Ask Player2 to Play!
                    bumpalo::format!(in bump, "Click here to Accept {}!", root_rendering_component.game_data.asked_folder_name)
                        .into_bump_str(),
                )]}
        </h2>
    </div>
    )
}
/// on click
fn div_want_to_play_asked_on_click(rrc: &mut RootRenderingComponent) {
    rrc.game_data.game_status = GameStatus::PlayAccepted;

    websocketcommunication::ws_send_msg(
        &rrc.game_data.ws,
        &WsMessage::PlayAccept {
            my_ws_uid: rrc.game_data.my_ws_uid,
            players: unwrap!(serde_json::to_string(&rrc.game_data.players)),
        },
    );
}


///render click one
fn div_click_one<'a, 'bump>(
    root_rendering_component: &'a RootRenderingComponent,
    bump: &'bump Bump,
) -> Node<'bump>
where
    'a: 'bump,
{
    if root_rendering_component.game_data.my_player_number
        == root_rendering_component.game_data.player_turn
    {
        dodrio!(bump,
        <div class="div_clickable">
            <h2 id= "ws_elem" style= "color:orange;">
                {vec![text(bumpalo::format!(in bump, "Play player{} !", root_rendering_component.game_data.player_turn).into_bump_str())]}
            </h2>
        </div>
        )
    } else {
        //return wait for the other player
        dodrio!(bump,
        <h2 id="ws_elem" style= "color:red;">
            {vec![text(bumpalo::format!(in bump, "Wait for player{} !", root_rendering_component.game_data.player_turn).into_bump_str())]}
        </h2>
        )
    }
}
///render take turn
fn div_take_turn<'a, 'bump>(
    root_rendering_component: &'a RootRenderingComponent,
    bump: &'bump Bump,
) -> Node<'bump>
where
    'a: 'bump,
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
            <h2 id= "ws_elem" style= "color:green;">
                {vec![text(
                    bumpalo::format!(in bump, "Click here to take your turn !{}", "")
                        .into_bump_str(),
                )]}
            </h2>
        </div>
        )
    } else {
        //return wait for the other player
        dodrio!(bump,
        <h2 id="ws_elem" style= "color:red;">
            {vec![text(bumpalo::format!(in bump, "Wait for player{} !", next_player).into_bump_str())]}
        </h2>
        )
    }
}
///render unpredicted
fn div_unpredicted<'a, 'bump>(
    root_rendering_component: &'a RootRenderingComponent,
    bump: &'bump Bump,
) -> Node<'bump>
where
    'a: 'bump,
{
    //unpredictable situation
    //return
    dodrio!(bump,
    <h2 id= "ws_elem">
        {vec![text(bumpalo::format!(in bump, "gamestatus: {} player {}", root_rendering_component.game_data.game_status.as_ref(),root_rendering_component.game_data.my_player_number).into_bump_str())]}
    </h2>
    )
}
