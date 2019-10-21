use std::io::{self, Write};

use reqwest;
use serde_json::{json, Value};

fn input(prompt: &str) -> Result<String, Box<dyn std::error::Error>> {
    print!("{}", prompt);
    io::stdout().flush()?;
    let mut s = String::new();
    io::stdin().read_line(&mut s)?;
    Ok(s)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let server = input("Server: ")?.trim().to_owned();
    let username = format!("@{}:{}", input("Username: ")?.trim(), server);
    let password = input("Password: ")?.trim().to_owned();
    let body = json!({
        "type": "m.login.password",
        "user": username,
        "password": password
    })
    .to_string();
    println!("body is {}", body);
    let res = reqwest::Client::new()
        .post(&format!("https://{}/_matrix/client/r0/login", server))
        .body(body)
        .send()?
        .text()?;
    println!("response was {}", res);
    let res_json: Value = serde_json::from_str(&res)?;
    println!("json response was {:?}", res_json);

    let message = input("What is your message?: ")?.trim().to_owned();
    let room = input("What room to send to?: ")?.trim().to_owned();

    let access_token = res_json["access_token"].as_str().unwrap();
    let body = json!({
        "msgtype": "m.text",
        "body": message
    })
    .to_string();

    let url = format!(
        "https://{}/_matrix/client/r0/rooms/{}/send/m.room.message?access_token={}",
        server, room, access_token
    );
    let res = reqwest::Client::new()
        .post(&url)
        .body(body)
        .send()?
        .text()?;

    println!("post response = {}", res);
    Ok(())
}
