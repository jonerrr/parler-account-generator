use anyhow::bail;
// use reqwest::multipart;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::{fs, sync};
// use tokio_util::codec;

use crate::info;

#[derive(Debug, Deserialize)]
struct SignupResult {
    status: String,
    data: Option<Token>,
}

#[derive(Debug, Deserialize)]
struct Token {
    token: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct AuthData {
    access_token: String,
}

#[derive(Debug, Deserialize)]
struct Auth {
    status: String,
    data: Option<AuthData>,
    errors: Option<Vec<serde_json::Value>>,
}

#[derive(Debug, Deserialize)]
pub struct ProfileResponse {
    status: String,
}

#[serde_with::serde_as]
#[derive(Debug, Deserialize)]
pub struct Jwt {
    #[serde_as(as = "serde_with::TimestampSecondsWithFrac")]
    exp: chrono::DateTime<chrono::Utc>,
    uuid: String,
}

macro_rules! static_headers {
    { $($name:literal: $value:literal,)* } => {{
        let mut count = 0;

        $(
            let _ = $name;
            count += 1;
        )*

        let mut result = ::reqwest::header::HeaderMap::with_capacity(count);

        $(
            result.insert($name, ::reqwest::header::HeaderValue::from_static($value));
        )*

        result
    }};
}

const ACCOUNT_VERSION: u32 = 6;

pub async fn generate(
    collection: mongodb::Collection<bson::Document>,
    resources: Arc<crate::Resources>,
    infinite: bool,
) -> anyhow::Result<()> {
    loop {
        let info = info::Info::new_random(&resources);

        println!("email: {}", info.email);

        let client = reqwest::Client::builder()
        // .proxy(reqwest::Proxy::https("socks5://")?)
        .default_headers(static_headers! {
            "user-agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:105.0) Gecko/20100101 Firefox/105.0",
            "accept": "application/json, text/plain, */*",
            "accept-language": "application/json, text/plain, */*",
            "referer": "https://parler.com/",
            "origin": "https://parler.com",
            "dnt": "1",
            "connection": "keep-alive",
            "sec-fetch-dest": "empty",
            "sec-fetch-mode": "cors",
            "sec-fetch-site": "same-site",
        })
        .build()
        .unwrap();

        let result: SignupResult = client
            .post("https://api.parler.com/v0/public/user/signup")
            .json(&serde_json::json!({
                "email": &info.email,
                "name": &info.name,
                "username": &info.username,
                "phone": "",
                "device_id": "",
                "password": &info.password,
                "consentSms": false,
                "captchaKey": &info.captcha_key,
                "interests": [&info.interests]
            }))
            .send()
            .await?
            .json()
            .await?;

        if result.status != "success" || result.data.is_none() {
            bail!("{} - error signing up", info.email);
        }

        let (verify_tx, verify_rx) = sync::oneshot::channel();
        resources.verify_queue.insert(info.email.clone(), verify_tx);

        let token = result.data.unwrap().token;

        let security_code = verify_rx.await.unwrap();

        let d: Auth = client
            .post("https://api.parler.com/v0/public/user/signup/confirm")
            .json(&serde_json::json!({ "token": token, "security_code": security_code }))
            .send()
            .await?
            .json()
            .await?;

        if d.status != "success" || d.errors.is_some() {
            bail!(
                "{} - error verifying code: {:#?}\n code: {}\n token: {}",
                info.email,
                d,
                security_code,
                token
            );
        }

        let payload = serde_json::from_slice::<Jwt>(&base64::decode_config(
            d.data
                .as_ref()
                .unwrap()
                .access_token
                .split(".")
                .nth(1)
                .unwrap(),
            base64::URL_SAFE,
        )?)?;

        let p: ProfileResponse = client
            .put("https://api.parler.com/v0/user")
            .header(
                "authorization",
                format!("Bearer {}", d.data.as_ref().unwrap().access_token),
            )
            .json(&serde_json::json!({
            //    "bio": &info.bio,
               "website": "",
               "location": &info.location,
               "name": &info.name,
               "username": &info.username,
            }))
            .send()
            .await?
            .json()
            .await?;

        if p.status != "success" {
            bail!("{} - error setting profile bio and location", info.email);
        }

        // let p: ProfileResponse = client
        //     .post("https://api.parler.com/v0/images/upload/profile")
        //     .header(
        //         "authorization",
        //         format!("Bearer {}", d.data.as_ref().unwrap().access_token),
        //     )
        //     .multipart(
        //         multipart::Form::new().part(
        //             "image",
        //             multipart::Part::stream(reqwest::Body::wrap_stream(codec::FramedRead::new(
        //                 fs::File::open(info.pfp).await?,
        //                 codec::BytesCodec::new(),
        //             )))
        //             .mime_str("image/png")?
        //             .file_name("image.png"),
        //         ),
        //     )
        //     .send()
        //     .await?
        //     .json()
        //     .await?;

        // if p.status != "success" {
        //     bail!("{} - error setting profile picture", info.email);
        // }

        println!("SUCCESS - {} - inserting", info.email);

        collection
            .insert_one(
                bson::doc! {
                    "_id": payload.uuid,
                    "version": ACCOUNT_VERSION,
                    "created": chrono::Utc::now(),
                    "jwt": d.data.unwrap().access_token,
                    "jwt_exp": payload.exp,
                    "username": info.username,
                    "password": info.password,
                    "email": info.email
                },
                None,
            )
            .await?;
        if !infinite {
            break;
        }
    }
    Ok(())
}
