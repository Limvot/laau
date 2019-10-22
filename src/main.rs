use std::error::Error;
use std::fs::File;
use std::io::{self, Write};

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
    println!("response was {}", res);
    Ok(serde_json::from_str(&res)?)
}

fn send_message(
    server: &str,
    access_token: &str,
    room: &str,
    msg: &str,
) -> Result<(), Box<dyn Error>> {
    let url = format!(
        "https://{}/_matrix/client/r0/rooms/{}/send/m.room.message?access_token={}",
        server, room, access_token
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
    println!("post response = {}", res);
    Ok(())
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

    let message = input("What is your message?: ")?.trim().to_owned();
    let room = input("What room to send to?: ")?.trim().to_owned();

    send_message(
        &login_response.home_server,
        &login_response.access_token,
        &room,
        &message,
    )?;
    Ok(())
}
