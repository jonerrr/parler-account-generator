use anyhow::anyhow;
use dotenv::dotenv;
use std::sync::Arc;
use std::{env, iter, path};
// use tokio::fs;

mod email;
mod generate;
mod info;

// #[derive(serde::Deserialize)]
// struct SefariaMerged {
//     text: Vec<Vec<String>>,
// }

pub type VerifyQueue = dashmap::DashMap<String, tokio::sync::oneshot::Sender<String>>;

pub struct Resources {
    pub verify_queue: VerifyQueue,
    // pub pfps: Vec<path::PathBuf>,
    // pub sefaria_quotes: Vec<String>,
    pub cities: Vec<String>,
}

// const SEFARIA_FILE_LIMIT: usize = 100;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();
    // let html_tag_regex = regex::Regex::new(r"</?[^>]+>").unwrap();
    let count = env::args()
        .skip(1)
        .next()
        .ok_or(anyhow!(
            "usage: <number: amount of accounts to generate> <boolean: infinite loop>"
        ))?
        .parse()?;
    let infinite = env::args()
        .skip(2)
        .next()
        .ok_or(anyhow!(
            "usage: <number: amount of accounts to generate> <boolean: infinite loop>"
        ))?
        .parse::<bool>()?;

    let mongo =
        mongodb::Client::with_uri_str(env::var("MONGO_URL").expect("MONGO_URL not set")).await?;
    let collection = mongo.database("accounts").collection("accounts");

    let resources = Arc::new(Resources {
        verify_queue: VerifyQueue::new(),
        // pfps: {
        //     let mut read_dir = fs::read_dir("./pfps").await?;
        //     let mut result = Vec::new();
        //     while let Some(x) = read_dir.next_entry().await? {
        //         result.push(x.path())
        //     }
        //     result
        // },
        // not async :rage:
        // sefaria_quotes: glob::glob("./Sefaria-Export/json/**/English/merged.json")
        //     .unwrap()
        //     .take(SEFARIA_FILE_LIMIT)
        //     .filter_map(|entry| {
        //         if let Ok(merged) = serde_json::from_reader::<_, SefariaMerged>(
        //             std::fs::File::open(entry.unwrap().as_path()).unwrap(),
        //         ) {
        //             Some(merged.text.into_iter())
        //         } else {
        //             None
        //         }
        //     })
        //     .flatten()
        //     .flatten()
        //     .filter(|x| !x.is_empty()) // remove empty
        //     .map(|x| html_tag_regex.replace_all(&x, "").to_string()) // remove html
        //     .collect(),
        // not async :rage:
        cities: serde_json::from_reader::<_, Vec<String>>(std::io::BufReader::new(
            std::fs::File::open("./cities.json")?,
        ))?,
    });

    println!(
        "profile arsenal: 0 pfps, 0 quotes, {} cities",
        // resources.pfps.len(),
        // resources.sefaria_quotes.len(),
        resources.cities.len(),
    );

    tokio::spawn({
        let resources = resources.clone();
        async move {
            if let Err(e) = email::run_code_finder(
                email::ImapConfig {
                    server: env::var("IMAP_SERVER").expect("IMAP_URL not set"),
                    port: env::var("IMAP_PORT")
                        .expect("IMAP_PORT not set")
                        .parse::<u16>()
                        .expect("Invalid IMAP_PORT"),
                    username: env::var("IMAP_USERNAME").expect("IMAP_USERNAME not set"),
                    password: env::var("IMAP_PASSWORD").expect("IMAP_PASSWORD not set"),
                },
                &resources.verify_queue,
            )
            .await
            {
                eprintln!("email error {:?}", e);
            }
        }
    });

    let acc_iter =
        iter::repeat_with(|| generate::generate(collection.clone(), resources.clone(), infinite));

    let _ = futures::future::try_join_all(acc_iter.take(count))
        .await
        .map_err(|e| eprintln!("generation error: {:?}", e));

    Ok(())
}
