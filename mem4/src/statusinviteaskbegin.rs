//! statusinviteaskbegin.rs - code flow from this status

//region: use
use crate::rootrenderingcomponent::RootRenderingComponent;
use crate::websocketcommunication;
use crate::logmod;
use crate::fetchmod;

use dodrio::builder::text;
use dodrio::bumpalo::{self, Bump};
use dodrio::Node;
use mem4_common::{GameStatus, Player, WsMessage};
use typed_html::dodrio;
use web_sys::{Request, RequestInit};
//endregion

///render invite ask begin, ask to play for multiple contents/folders
pub fn div_invite_ask_begin<'a, 'bump>(
    root_rendering_component: &'a RootRenderingComponent,
    bump: &'bump Bump,
) -> Node<'bump>
where
    'a: 'bump,
{
    logmod::log1_str("GameStatus::InviteAskBegin");
    let mut vec_of_nodes = Vec::new();
    //I don't know how to solve the lifetime problems. So I just clone the small data.
    let ff = root_rendering_component.game_data.content_folders.clone();
    for folder_name in ff {
        let folder_name_clone2 = folder_name.clone();
        vec_of_nodes.push(dodrio!(bump,
        <div class="div_clickable" onclick={move |root, vdom, _event| {
                let rrc = root.unwrap_mut::<RootRenderingComponent>();
                let v2= vdom.clone();
                div_invite_ask_begin_on_click(rrc, &folder_name,v2);

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
pub fn div_invite_ask_begin_on_click(
    rrc: &mut RootRenderingComponent,
    folder_name: &str,
    vdom_weak: dodrio::VdomWeak,
) {
    rrc.game_data.my_player_number = 1;
    rrc.game_data.players.clear();
    rrc.game_data.players.push(Player {
        ws_uid: rrc.game_data.my_ws_uid,
        points: 0,
    });
    rrc.game_data.game_status = GameStatus::InviteAsking;
    rrc.game_data.asked_folder_name = folder_name.to_string();

    //async fetch_response() for gameconfig.json
    let url_config = format!(
        "{}/content/{}/game_config.json",
        rrc.game_data.href, rrc.game_data.asked_folder_name
    );
    logmod::log1_str(url_config.as_str());
    let webrequest = create_webrequest(url_config.as_str());
    fetchmod::fetch_response(vdom_weak, &webrequest, &set_config_json);

    //send the msg Invite
    websocketcommunication::ws_send_msg(
        &rrc.game_data.ws,
        &WsMessage::Invite {
            my_ws_uid: rrc.game_data.my_ws_uid,
            asked_folder_name: folder_name.to_string(),
        },
    );
}

///create web request from string
pub fn create_webrequest(url: &str) -> web_sys::Request {
    let mut opts = RequestInit::new();
    opts.method("GET");

    let w_webrequest = unwrap!(Request::new_with_str_and_init(url, &opts));

    logmod::log1_str("let w_webrequest =");
    //return
    w_webrequest
}

#[allow(clippy::needless_pass_by_value)]
/// update a field in the struct
pub fn set_config_json(rrc: &mut RootRenderingComponent, respbody: String) {
    //respbody is json.
    logmod::log1_str(format!("respbody {}", respbody).as_str());
    rrc.game_data.game_config = unwrap!(serde_json::from_str(respbody.as_str()));
}

///msg invite
pub fn on_msg_invite(
    rrc: &mut RootRenderingComponent,
    my_ws_uid: usize,
    asked_folder_name: String,
) {
    logmod::log1_str("rcv invite");
    rrc.reset();
    rrc.game_data.game_status = GameStatus::InviteAsked;
    //the first player is the initiator
    rrc.game_data.players.push(Player {
        ws_uid: my_ws_uid,
        points: 0,
    });
    rrc.game_data.players.push(Player {
        ws_uid: rrc.game_data.my_ws_uid,
        points: 0,
    });
    rrc.game_data.my_player_number = 2; //temporary number
    rrc.game_data.asked_folder_name = asked_folder_name;
}
