use std::error::Error;
use std::fs::File;
use std::io::{self, Write};
use std::collections::HashMap;

use reqwest;
use serde::{Deserialize, Serialize};
use serde_json::json;

fn input(prompt: &str) -> Result<String, Box<dyn std::error::Error>> {
    print!("{}", prompt);
    io::stdout().flush()?;
    let mut s = String::new();
    io::stdin().read_line(&mut s)?;
    Ok(s)
}

#[derive(Debug, Serialize, Deserialize)]
struct LoginResponse {
    access_token: String,
    home_server: String,
    user_id: String,
    device_id: String,
}

fn login(server: &str, username: &str, password: &str) -> Result<LoginResponse, Box<dyn Error>> {
    let body = json!({
        "type": "m.login.password",
        "user": username,
        "password": password
    })
    .to_string();
    let res = reqwest::Client::new()
        .post(&format!("https://{}/_matrix/client/r0/login", server))
        .body(body)
        .send()?
        .text()?;
    println!("login response was {}", res);
    Ok(serde_json::from_str(&res)?)
}

fn send_message(
    login_response: &LoginResponse,
    room: &str,
    msg: &str,
) -> Result<(), Box<dyn Error>> {
    let url = format!(
        "https://{}/_matrix/client/r0/rooms/{}/send/m.room.message?access_token={}",
        login_response.home_server, room, login_response.access_token
    );
    let body = json!({
        "msgtype": "m.text",
        "body": msg
    })
    .to_string();
    let res = reqwest::Client::new()
        .post(&url)
        .body(body)
        .send()?
        .text()?;
    println!("send_message response was {}", res);
    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct MessageContent {
    body: String,
    msgtype: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
struct RoomName {
    name: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
struct RoomMembership {
    displayname: Option<String>,
    membership: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
struct RoomCreate {
    creator: String,
    room_version: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ReactionContent {
    #[serde(rename = "m.relates_to")]
    relates_to: ReactionRelates,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ReactionRelates {
    event_id: String,
    key: String,
    rel_type: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
enum Event {
    #[serde(rename = "m.room.message")]
    RoomMessage {
        content: MessageContent,
        #[serde(flatten)]
        meta: EventMeta,
    },
    #[serde(rename = "m.room.name")]
    RoomName {
        content: RoomName,
        #[serde(flatten)]
        meta: EventMeta,
    },
    #[serde(rename = "m.room.member")]
    RoomMember {
        content: RoomMembership,
        #[serde(flatten)]
        meta: EventMeta,
    },
    #[serde(rename = "m.room.create")]
    RoomCreate {
        content: RoomCreate,
        #[serde(flatten)]
        meta: EventMeta,
    },
    #[serde(rename = "m.reaction")]
    RoomReaction {
        content: ReactionContent,
        #[serde(flatten)]
        meta: EventMeta,
    },
    #[serde(rename = "m.room.encryption")]
    RoomEncryption {
        #[serde(flatten)]
        meta: EventMeta,
    },
    #[serde(rename = "m.room.encrypted")]
    RoomEncrypted {
        #[serde(flatten)]
        meta: EventMeta,
    },
    #[serde(other)]
    Other,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
struct EventMeta {
    event_id: String,
    origin_server_ts: usize,
    sender: String,
    // unsigned: { "age": usize }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Timeline {
    events: Vec<Event>,
    limited: bool,
    prev_batch: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
struct State {
    events: Vec<Event>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
struct RoomData {
    //account_data: Vec<Event>,
    //ephemeral: Vec<Event>,
    state: State,
    //summary: {}
    timeline: Timeline,
    //unread_notifications: {
        //highlight_count: 0,
        //notification_count  2,
    //}
}
#[derive(Debug, Clone, Serialize, Deserialize)]
struct InviteJoinLeave {
    invite: HashMap<String, RoomData>,
    join: HashMap<String, RoomData>,
    leave: HashMap<String, RoomData>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
struct SyncResponse {
    //account_data: {
        //events: [
            // Events that set something about direct rooms?
            // Events that set notification settings?
        //]
    //},
    //device_lists: { changed: [], left: [] }
    //device_one_time_keys_count: { changed: [], left: [] }
    //groups: { invite: [], join: [], leave: [] }
    next_batch: String,
    //presence: { events: [] },
    rooms: InviteJoinLeave,
    //rooms: {
        //invite: {},
        //join: {
            //room_id: {
                //account_data: {
                    //events: {
                        // looks like have we read
                    //}
                //},
                //ephemeral: {
                    //events: {
                        // looks like other read recipts
                    //}
                //},
                //state: {
                    //events: {
                        // looks like general state
                    //}
                //},
                //summary: {},
                //timeline: {
                    //events: {
                        // looks like normal message events, name room event
                    //},
                    //limited: true,
                    //prev_batch: String
                //},
                //unread_notifications: {
                    //highlight_count: 0,
                    //notification_count  2,
                //}
            //}
        //},
        //leave: {}
    //},
    //to_device: {
        //events: {}
    //}
}
fn sync(login_response: &LoginResponse) -> Result<SyncResponse, Box<dyn Error>> {
    let url = format!(
        "https://{}/_matrix/client/r0/sync?access_token={}",
        login_response.home_server, login_response.access_token
    );
    let res = reqwest::Client::new().get(&url).send()?.text()?;
    //println!("sync response was {}", res);
    Ok(serde_json::from_str(&res)?)
}

#[derive(Debug, Serialize, Deserialize)]
struct RoomMessages {
    chunk: Vec<Event>,
    start: String,
    end: String,
}
fn room_messages(login_response: &LoginResponse, room: &str, from: &str) -> Result<RoomMessages, Box<dyn Error>> {
    let url = format!(
        "https://{}/_matrix/client/r0/rooms/{}/messages?access_token={}&from={}",
        login_response.home_server, room, login_response.access_token, from
    );
    let res = reqwest::Client::new().get(&url).send()?.text()?;
    //println!("room_messages response was {}", res);
    Ok(serde_json::from_str(&res)?)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let filename = "./saved_login.json";
    let login_response = if let Ok(Ok(login_response)) =
        File::open(filename).map(|reader| serde_json::from_reader(reader))
    {
        login_response
    } else {
        let server = input("Server: ")?.trim().to_owned();
        let username = format!("@{}:{}", input("Username: ")?.trim(), server);
        let password = input("Password: ")?.trim().to_owned();
        let login_response = login(&server, &username, &password)?;
        serde_json::to_writer(File::create(filename)?, &login_response)?;
        login_response
    };
    let synced = sync(&login_response)?;
    //println!("synced {:?}", synced);
    let room_keys = synced.rooms.join.iter().map(|(k,v)| k.clone()).collect::<Vec<String>>();
    for (i, room_id) in room_keys.iter().enumerate() {
        let room_name = synced.rooms.join.get(room_id).unwrap().state.events.iter().find_map(|e| match e { Event::RoomName { content, meta } => Some(content.name.clone()), _ => None}).unwrap_or("NONE".to_owned());
        println!("{}: {}/{}", i+1, room_name, room_id);
    }
    loop {
        let room = input("What room to send to? (enter number, 0 to sync, -room for room events): ")?
            .trim()
            .parse::<i64>()?;
        if room > 0 {
            let message = input("What is your message?: ")?.trim().to_owned();
            send_message(&login_response, &room_keys[(room-1) as usize], &message)?;
        } else if room == 0 {
        } else {
            println!("room events from sync:");
            let room_id = &room_keys[((-room)-1) as usize];
            let mut events = synced.rooms.join.get(room_id).unwrap().timeline.events.clone();
            let room_message_updates = room_messages(&login_response, room_id, &synced.next_batch)?;
            let mut next_batch = synced.next_batch.clone();
            while let Ok(mut room_message_updates) = room_messages(&login_response, room_id, &next_batch) {
                if room_message_updates.chunk.len() == 0 {
                    break;
                }
                next_batch = room_message_updates.end;
                events.append(&mut room_message_updates.chunk);
            }
            for event in &events {
                match event {
                    Event::RoomMessage { content, meta } => println!("\t{}: {}", meta.sender, content.body),
                    Event::RoomName { content, meta } => println!("\t{} set room name to \"{}\"", meta.sender, content.name),
                    _ => println!("\tUnknown event {:?}", event),
                }
            }
        }
    }
    //Ok(())
}
