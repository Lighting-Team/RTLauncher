use reqwest::Client;
use sqlite::State;
use serde::{Deserialize, Serialize};
use sqlite::Connection;
use std::time::Duration;
use tokio::time::sleep;

#[derive(Serialize, Deserialize, Debug)]
struct DeviceCodeResponse {
    device_code: String,
    user_code: String,
    verification_uri: String,
    expires_in: u64,
    interval: u64,
    message: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct TokenResponse {
    token_type: String,
    access_token: String,
    refresh_token: String,
    expires_in: u64,
}

#[derive(Serialize, Deserialize, Debug)]
struct XboxLiveTokenResponse {
    Token: String,
    DisplayClaims: DisplayClaims,
}

#[derive(Serialize, Deserialize, Debug)]
struct DisplayClaims {
    xui: Vec<Xui>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Xui {
    uhs: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct MinecraftLoginResponse {
    username: String,
    access_token: String,
    token_type: String,
    expires_in: u64,
}

#[derive(Serialize, Deserialize, Debug)]
struct MinecraftProfileResponse {
    id: String,
    name: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化 SQLite 数据库
    let connection = setup_database()?;
    let client = Client::new();
    let client_id = "1662e9cb-e526-4bea-8237-11526075b7f3";

    // Step 1: Get device code
    
   check_account_time(&client, &connection, client_id,"Elanda_seaweeds").await?;
    Ok(())
}

async fn get_device_code(client: &Client, client_id: &str) -> Result<DeviceCodeResponse, Box<dyn std::error::Error>> {
    let params = [
        ("client_id", client_id),
        ("scope", "XboxLive.signin offline_access"),
    ];
    let response = client
        .post("https://login.microsoftonline.com/consumers/oauth2/v2.0/devicecode")
        .form(&params)
        .send()
        .await?
        .json::<DeviceCodeResponse>()
        .await?;
    Ok(response)
}

async fn poll_for_token(
    client: &Client,
    client_id: &str,
    device_code: &str,
    interval: u64,
) -> Result<TokenResponse, Box<dyn std::error::Error>> {
    loop {
        let params = [
            ("grant_type", "urn:ietf:params:oauth:grant-type:device_code"),
            ("client_id", client_id),
            ("device_code", device_code),
        ];
        let response = client
            .post("https://login.microsoftonline.com/consumers/oauth2/v2.0/token")
            .form(&params)
            .send()
            .await?;
        if response.status().is_success() {
            return Ok(response.json::<TokenResponse>().await?);
        }
        sleep(Duration::from_secs(interval)).await;
    }
}

async fn authenticate_with_xbox_live(
    client: &Client,
    access_token: &str,
) -> Result<XboxLiveTokenResponse, Box<dyn std::error::Error>> {
    let body = serde_json::json!({
        "Properties": {
            "AuthMethod": "RPS",
            "SiteName": "user.auth.xboxlive.com",
            "RpsTicket": format!("d={}", access_token)
        },
        "RelyingParty": "http://auth.xboxlive.com",
        "TokenType": "JWT"
    });
    let response = client
        .post("https://user.auth.xboxlive.com/user/authenticate")
        .json(&body)
        .send()
        .await?
        .json::<XboxLiveTokenResponse>()
        .await?;
    Ok(response)
}

async fn get_xsts_token(
    client: &Client,
    xbox_token: &str,
) -> Result<XboxLiveTokenResponse, Box<dyn std::error::Error>> {
    let body = serde_json::json!({
        "Properties": {
            "SandboxId": "RETAIL",
            "UserTokens": [xbox_token]
        },
        "RelyingParty": "rp://api.minecraftservices.com/",
        "TokenType": "JWT"
    });
    let response = client
        .post("https://xsts.auth.xboxlive.com/xsts/authorize")
        .json(&body)
        .send()
        .await?
        .json::<XboxLiveTokenResponse>()
        .await?;
    Ok(response)
}

async fn authenticate_with_minecraft(
    client: &Client,
    user_hash: &str,
    xsts_token: &str,
) -> Result<MinecraftLoginResponse, Box<dyn std::error::Error>> {
    let body = serde_json::json!({
        "identityToken": format!("XBL3.0 x={};{}", user_hash, xsts_token)
    });
    let response = client
        .post("https://api.minecraftservices.com/authentication/login_with_xbox")
        .json(&body)
        .send()
        .await?
        .json::<MinecraftLoginResponse>()
        .await?;
    Ok(response)
}

async fn check_mc_purchase(client: &Client, access_token: &str) -> Result<String, Box<dyn std::error::Error>> {
    let response = client
        .get("https://api.minecraftservices.com/entitlements/mcstore")
        .bearer_auth(access_token)
        .send()
        .await?;
    if response.status().is_success() {
        let json: serde_json::Value = response.json().await?;
        if json.is_null() || json["items"].is_null() || json["items"].as_array().unwrap().is_empty() {
            return Ok("您还没有购买mc，请购买后再登录游玩".to_string());
        }
    }
    Ok("您已购买Minecraft".to_string())
}

async fn get_minecraft_profile(
    client: &Client,
    access_token: &str,
) -> Result<MinecraftProfileResponse, Box<dyn std::error::Error>> {
    let response = client
        .get("https://api.minecraftservices.com/minecraft/profile")
        .bearer_auth(access_token)
        .send()
        .await?
        .json::<MinecraftProfileResponse>()
        .await?;
    Ok(response)
}

async fn refresh_access_token(
    client: &Client,
    client_id: &str,
    refresh_token: &str,
) -> Result<TokenResponse, Box<dyn std::error::Error>> {
    let params = [
        ("grant_type", "refresh_token"),
        ("client_id", client_id),
        ("refresh_token", refresh_token),
    ];
    let response = client
        .post("https://login.microsoftonline.com/consumers/oauth2/v2.0/token")
        .form(&params)
        .send()
        .await?
        .json::<TokenResponse>()
        .await?;
    Ok(response)
}

// 初始化数据库
fn setup_database() -> Result<Connection, Box<dyn std::error::Error>> {
    let connection = sqlite::open("LaunchAccount.db")?;
    connection.execute(
        "CREATE TABLE IF NOT EXISTS accounts (
            uuid TEXT PRIMARY KEY,
            username TEXT,
            refresh_token TEXT,
            access_token TEXT,
            time INTEGER
        )",
    )?;
    Ok(connection)
}

// 将账户信息保存到数据库
fn save_account_info(
    connection: &Connection,
    username: &str,
    uuid: &str,
    refresh_token: &str,
    access_token: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let current_time = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_secs();
    connection.execute(format!(
        "INSERT OR REPLACE INTO accounts (uuid, username, refresh_token, access_token, time) VALUES ('{}', '{}', '{}', '{}', '{}')",
        uuid, username, refresh_token, access_token, current_time
    ))?;
    Ok(())
}
async fn check_account_time(
    client: &Client,
    connection: &Connection,
    client_id: &str,
    username: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let query = format!("SELECT uuid, refresh_token, access_token, time FROM accounts WHERE username = '{}'", username);
    let mut stmt = connection.prepare(query)?;

    if let State::Row = stmt.next()? {
        let uuid: String = stmt.read::<String, _>(0)?;
        let refresh_token: String = stmt.read::<String, _>(1)?;
        let access_token: String = stmt.read::<String, _>(2)?;
        let last_login_time: i64 = stmt.read::<i64, _>(3)?;

        let current_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs();

        if current_time - last_login_time as u64 > 29 * 24 * 3600 {
            // Token is older than 29 days, re-login using device code flow
            println!("Token is older than 29 days, initiating device code flow...");

            let device_code_response = get_device_code(client, client_id).await?;
            println!(
                "Please visit {} and enter code: {}",
                device_code_response.verification_uri, device_code_response.user_code
            );

            let token_response = poll_for_token(
                client,
                client_id,
                &device_code_response.device_code,
                device_code_response.interval,
            )
            .await?;

            let xbox_token_response = authenticate_with_xbox_live(client, &token_response.access_token).await?;
            let xsts_token_response = get_xsts_token(client, &xbox_token_response.Token).await?;
            let minecraft_login_response = authenticate_with_minecraft(
                client,
                &xbox_token_response.DisplayClaims.xui[0].uhs,
                &xsts_token_response.Token,
            )
            .await?;

            save_account_info(
                connection,
                username,
                &uuid,
                &token_response.refresh_token,
                &minecraft_login_response.access_token,
            )?;

            println!("Device code flow completed. Tokens updated.");
        } else if current_time - last_login_time as u64 > 11 * 3600 {
            // Token is older than 11 hours, refresh it
            println!("Token is older than 11 hours, refreshing access token...");

            let refreshed_token_response = refresh_access_token(client, client_id, &refresh_token).await?;

            save_account_info(
                connection,
                username,
                &uuid,
                &refreshed_token_response.refresh_token,
                &refreshed_token_response.access_token,
            )?;

            println!("Access token refreshed.");
        } else {
            println!("Token is still valid.");
        }
    } else {
        println!("No account found with username: {}", username);
    }

    Ok(())
}
