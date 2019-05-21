//! Learning to code Rust for a http + websocket server on the same port  
//! using Warp for a simple memory game for kids - mem4.
//! On the local public IP address on port 80 listens to http and websocket.
//! Route for http / serves static files from folder /mem4/
//! Route /mem4ws/ broadcast all websocket msg to all connected clients except sender

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
extern crate ansi_term;
extern crate clap;
extern crate env_logger;
extern crate futures;
#[macro_use]
extern crate log;
extern crate mem4_common;
extern crate regex;
extern crate serde;
extern crate serde_derive;
extern crate serde_json;
extern crate warp;

use clap::{App, Arg};
use env_logger::Env;
use futures::sync::mpsc;
use futures::{Future, Stream};
use mem4_common::WsMessage;
use regex::Regex;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::net::SocketAddr;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::process::Command;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc, Mutex,
};
use warp::ws::{Message, WebSocket};
use warp::Filter;
//endregion

//region: enum, structs, const,...
/// Our global unique user id counter.
static NEXT_USER_ID: AtomicUsize = AtomicUsize::new(1);

/// Our state of currently connected users.
/// - Key is their id
/// - Value is a sender of `warp::ws::Message`
type Users = Arc<Mutex<HashMap<usize, mpsc::UnboundedSender<Message>>>>;

//endregion

///main function of the binary
fn main() {
    //region: ansi terminal color output (for log also)
    //TODO: what is the difference between output and Log? When to use them?
    //only windows need this line
    enable_ansi_support();
    /*
    //region: examples
    eprintln!(
        "This is in red: {}",
        ansi_term::Colour::Red.paint("a red string")
    );

    eprintln!(
        "How about some {} and {}?",
        ansi_term::Style::new().bold().paint("bold"),
        ansi_term::Style::new().underline().paint("underline")
    );
    //endregion
    */
    //endregion

    //region: env_logger log text to stdout depend on ENV variable
    //in Linux : RUST_LOG=info ./mem4_server.exe
    //in Windows I don't know yet.
    //default for env variable info
    let mut builder = env_logger::from_env(Env::default().default_filter_or("info"));
    //nanoseconds in the logger
    builder.default_format_timestamp_nanos(true);
    builder.init();
    //endregion

    //region: cmdline parameters
    //TODO: is this a clear case for shadowing? The same value in different types
    //default ip and port
    //let df_local_ip = local_ip_get().expect("cannot get local ip");
    let df_local_ip = "127.0.0.22";
    let df_local_port = 80;
    //string representation of defaults
    let prm_ip = df_local_ip.to_string();
    let prm_port = df_local_port.to_string();

    let matches = App::new("mem4_server")
        .version("1.0.0")
        .author("Luciano Bestia")
        .about("server http and websocket for mem4 game")
        .arg(
            Arg::with_name("prm_ip")
                .value_name("ip")
                .default_value(&prm_ip)
                .help("ip address for listening"),
        )
        .arg(
            Arg::with_name("prm_port")
                .value_name("port")
                .default_value(&prm_port)
                .help("port for listening"),
        )
        .get_matches();

    //from string parameters to strong types
    let fnl_prm_ip = matches.value_of("prm_ip").expect("error on prm_ip");
    let fnl_prm_port = matches.value_of("prm_port").expect("error on prm_port");
    let local_ip = IpAddr::V4(fnl_prm_ip.parse::<Ipv4Addr>().expect("not an ip address"));
    let local_port = u16::from_str_radix(fnl_prm_port, 10).expect("not a number");
    let local_addr = SocketAddr::new(local_ip, local_port);

    info!(
        "mem4 http server listening on {} and websocket on /mem4ws/",
        ansi_term::Colour::Red.paint(local_addr.to_string())
    );
    //endregion

    // Keep track of all connected users, key is usize, value
    // is a websocket sender.
    let users = Arc::new(Mutex::new(HashMap::new()));
    // Turn our "state" into a new Filter...
    //let users = warp::any().map(move || users.clone());
    //Clippy recommands this crazyness instead of just users.clone()
    let users = warp::any().map(move || {
        Arc::<
            std::sync::Mutex<
                std::collections::HashMap<
                    usize,
                    futures::sync::mpsc::UnboundedSender<warp::ws::Message>,
                >,
            >,
        >::clone(&users)
    });

    //websocket server
    // GET from route /mem4ws/ -> websocket upgrade
    let websocket = warp::path("mem4ws")
        // The `ws2()` filter will prepare Websocket handshake...
        .and(warp::ws2())
        .and(users)
        .map(|ws: warp::ws::Ws2, users| {
            // This will call our function if the handshake succeeds.
            ws.on_upgrade(move |socket| user_connected(socket, users))
        });

    //static file server
    // GET files of route / -> are from folder /mem4/
    let fileserver = warp::fs::dir("./mem4/");

    let routes = fileserver.or(websocket);
    warp::serve(routes).run(local_addr);
}

//region: websocket callbacks: connect, msg, disconnect
///new user connects
fn user_connected(ws: WebSocket, users: Users) -> impl Future<Item = (), Error = ()> {
    // Use a counter to assign a new unique ID for this user.
    let my_id = NEXT_USER_ID.fetch_add(1, Ordering::Relaxed);

    info!("new websocket user: {}", my_id);

    // Split the socket into a sender and receive of messages.
    let (user_ws_tx, user_ws_rx) = ws.split();

    // Use an unbounded channel to handle buffering and flushing of messages
    // to the websocket...
    let (tx, rx) = mpsc::unbounded();
    warp::spawn(
        rx.map_err(|()| -> warp::Error { unreachable!("unbounded rx never errors") })
            .forward(user_ws_tx)
            .map(|_tx_rx| ())
            .map_err(|ws_err| info!("websocket send error: {}", ws_err)),
    );

    // Save the sender in our list of connected users.
    users.lock().expect("error uses.lock()").insert(my_id, tx);

    // Return a `Future` that is basically a state machine managing
    // this specific user's connection.
    // Make an extra clone to give to our disconnection handler...
    //let users2 = users.clone();
    //Clippy reccomands this crazyness insted of users.clone()
    let users2 = Arc::<
        std::sync::Mutex<
            std::collections::HashMap<
                usize,
                futures::sync::mpsc::UnboundedSender<warp::ws::Message>,
            >,
        >,
    >::clone(&users);

    user_ws_rx
        // Every time the user sends a message, broadcast it to
        // all other users...
        .for_each(move |msg| {
            user_message(my_id, &msg, &users);
            Ok(())
        })
        // for_each will keep processing as long as the user stays
        // connected. Once they disconnect, then...
        .then(move |result| {
            user_disconnected(my_id, &users2);
            result
        })
        // If at any time, there was a websocket error, log here...
        .map_err(move |e| {
            info!("websocket error(uid={}): {}", my_id, e);
        })
}

///on receive websocket message
fn user_message(ws_uid_of_message: usize, messg: &Message, users: &Users) {
    // Skip any non-Text messages...
    let msg = if let Ok(s) = messg.to_str() {
        s
    } else {
        return;
    };

    let new_msg = msg.to_string();
    //info!("msg: {}", new_msg);

    //There are different messages coming from wasm
    //ConnectionTest returns a message YourWebSocketUid
    //WantToPlay must be broadcasted to all users
    //RequestSpelling must return a message ResponseSpellingJson to the same user
    //all others must be forwarded to exactly the other player.

    let msg: WsMessage = serde_json::from_str(&new_msg).unwrap_or_else(|_x| WsMessage::Dummy {
        dummy: String::from("error"),
    });

    match msg {
        WsMessage::Dummy { dummy } => info!("Dummy: {}", dummy),
        WsMessage::RequestWsUid { test } => {
            info!("RequestWsUid: {}", test);
            let j = serde_json::to_string(&WsMessage::ResponseWsUid { your_ws_uid: ws_uid_of_message })
                .expect("serde_json::to_string(&WsMessage::ResponseWsUid { your_ws_uid: ws_uid_of_message })");
            info!("send ResponseWsUid: {}", j);
            match users
                .lock()
                .expect("error users.lock()")
                .get(&ws_uid_of_message)
                .unwrap()
                .unbounded_send(Message::text(j))
            {
                Ok(()) => (),
                Err(_disconnected) => {}
            }
        }
        WsMessage::RequestSpelling { filename } => {
            info!("RequestSpelling: {}", filename);
            // read the file
            let mut pathbuf = env::current_dir().expect("env::current_dir()");
            pathbuf.push("mem4");
            pathbuf.push(filename);
            let filename =
                String::from(pathbuf.as_path().to_str().expect("path.as_path().to_str()"));
            info!("filename: {}", filename);
            let mut file = File::open(filename).expect("Unable to open the file");
            let mut contents = String::new();
            file.read_to_string(&mut contents)
                .expect("Unable to read the file");
            info!("read file : {}", contents);
            let j = serde_json::to_string(&WsMessage::ResponseSpellingJson { json: contents })
                .expect(
                    "serde_json::to_string(&WsMessage::ResponseSpellingJson { json: contents })",
                );
            info!("send ResponseSpellingJson: {}", j);
            match users
                .lock()
                .expect("error users.lock()")
                .get(&ws_uid_of_message)
                .unwrap()
                .unbounded_send(Message::text(j))
            {
                Ok(()) => (),
                Err(_disconnected) => {}
            }
        }
        WsMessage::WantToPlay { .. } => broadcast(users, ws_uid_of_message, &new_msg),
        mem4_common::WsMessage::ResponseWsUid { .. }
        | mem4_common::WsMessage::ResponseSpellingJson { .. }
        | mem4_common::WsMessage::AcceptPlay { .. }
        | mem4_common::WsMessage::PlayerClick { .. }
        | mem4_common::WsMessage::EndGame { .. }
        | mem4_common::WsMessage::PlayerChange { .. } => send_to_other_player(users, &new_msg),
    }
}
///New message from this user send only to the other player.
fn send_to_other_player(users: &Users, new_msg: &str) {
    //info!("send_to_other_player: {}", new_msg);
    //the other user is in the other_ws_uid field

    //serde_json knows: 1. map (key+value pair) and 2. array
    let json_map: serde_json::Map<String, serde_json::Value> =
        serde_json::from_str(new_msg).expect("error serde_json::from_str(new_msg)");
    //info!("js_other_ws_uid: {}", v);
    let tuple = json_map
        .iter()
        .next()
        .expect("error json_map.iter().next()");

    let object = tuple.1;
    info!("object: {}", object);

    let js_other_ws_uid = object
        .get("other_ws_uid")
        .expect("error object.get(other_ws_uid)");
    info!("js_other_ws_uid: {}", js_other_ws_uid);

    let i64_other_ws_uid = js_other_ws_uid.as_i64().expect("js_other_ws_uid.as_i64()");
    info!("i64_other_ws_uid: {}", i64_other_ws_uid);
    let other_ws_uid =
        usize::try_from(i64_other_ws_uid).expect("usize::try_from(i64_other_ws_uid)");

    match users
        .lock()
        .expect("error users.lock()")
        .get(&other_ws_uid)
        .unwrap()
        .unbounded_send(Message::text(String::from(new_msg)))
    {
        Ok(()) => (),
        Err(_disconnected) => {}
    }
}
///broadcast is the simplest
fn broadcast(users: &Users, ws_uid_of_message: usize, new_msg: &str) {
    // New message from this user, send it to everyone else (except same uid)...
    // We use `retain` instead of a for loop so that we can reap any user that
    // appears to have disconnected.
    info!("broadcast: {}", new_msg);
    for (&uid, tx) in users.lock().expect("error users.lock()").iter() {
        if ws_uid_of_message != uid {
            match tx.unbounded_send(Message::text(String::from(new_msg))) {
                Ok(()) => (),
                Err(_disconnected) => {
                    // The tx is disconnected, our `user_disconnected` code
                    // should be happening in another task, nothing more to
                    // do here.
                }
            }
        }
    }
}

///disconnect user
fn user_disconnected(my_id: usize, users: &Users) {
    info!("good bye user: {}", my_id);

    // Stream closed up, so remove from the user list
    users.lock().expect("users.lock").remove(&my_id);
}
//endregion

//region: local ip (linux and windows distict versions)
#[cfg(target_family = "unix")]
///get local ip for unix with ifconfig
pub fn local_ip_get() -> Option<IpAddr> {
    info!("local_ip_get for unix{}", "");
    let output = Command::new("ifconfig")
        .output()
        .expect("failed to execute `ifconfig`");

    let stdout = String::from_utf8(output.stdout).unwrap();

    let re = Regex::new(r#"(?m)^.*inet (addr:)?(([0-9]*\.){3}[0-9]*).*$"#).unwrap();
    for cap in re.captures_iter(&stdout) {
        let host = cap.get(2).map_or("", |m| m.as_str());
        if host != "127.0.0.1" {
            if let Ok(addr) = host.parse::<Ipv4Addr>() {
                return Some(IpAddr::V4(addr));
            }
            if let Ok(addr) = host.parse::<Ipv6Addr>() {
                return Some(IpAddr::V6(addr));
            }
        }
    }
    return None;
}

#[cfg(target_family = "windows")]
///get local ip for windows with ipconfig
pub fn local_ip_get() -> Option<IpAddr> {
    info!("local_ip_get for windows{}", "");
    let output = Command::new("ipconfig")
        .output()
        .expect("failed to execute `ipconfig`");

    let stdout = String::from_utf8(output.stdout).expect("failed stdout");
    //variables are block scope and will not interfere with the other block
    {
        let re =
            Regex::new(r"(?m)^   IPv4 Address\. \. \. \. \. \. \. \. \. \. \. : ([0-9\.]*)\s*$")
                .expect("failed regex");
        let cap = re.captures(&stdout).expect("failed capture");
        let host = cap.get(1).map_or("", |m| m.as_str());
        if let Ok(addr) = host.parse::<Ipv4Addr>() {
            return Some(IpAddr::V4(addr));
        }
    }
    //variables are block scope and will not interfere with the other block
    {
        let re =
            Regex::new(r"(?m)^   Link-local IPv6 Address \. \. \. \. \. : ([:%a-f0-9\.]*)\s*$")
                .expect("failed regex");
        let cap = re.captures(&stdout).expect("capture");
        let host = cap.get(1).map_or("", |m| m.as_str());
        if let Ok(addr) = host.parse::<Ipv6Addr>() {
            return Some(IpAddr::V6(addr));
        }
    }
    None
}
//endregion

//region: only windows need enable ansi support
#[cfg(target_family = "windows")]
///ansi support
pub fn enable_ansi_support() {
    let _enabled = ansi_term::enable_ansi_support();
}
#[cfg(target_family = "unix")]
///ansi support
pub fn enable_ansi_support() {
    //do nothing
}
