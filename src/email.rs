use async_imap::types::Fetch;
use async_imap::{extensions::idle, imap_proto};
use futures::TryStreamExt;
use std::str;

pub struct ImapConfig {
    pub server: String,
    pub port: u16,
    pub username: String,
    pub password: String,
}

fn parse_verification(msg: &Fetch) -> Option<(String, String)> {
    lazy_static::lazy_static! {
        static ref TO_REGEX: regex::Regex = regex::Regex::new(r"To: .+ <(.+)>").unwrap();
        static ref CODE_REGEX: regex::Regex = regex::Regex::new(r"--.+\r\n(.+\r\n)+\r\nYour Parler Verification Code:(\d{6})").unwrap();
    }

    let headers = str::from_utf8(msg.header()?).unwrap();
    let text = str::from_utf8(msg.text()?).unwrap();

    let to = TO_REGEX.captures(&headers)?.get(1)?.as_str();
    let code = CODE_REGEX.captures(&text)?.get(2)?.as_str();

    Some((to.into(), code.into()))
}

pub async fn run_code_finder(
    config: ImapConfig,
    verify_queue: &crate::VerifyQueue,
) -> anyhow::Result<()> {
    let client = async_imap::connect(
        (config.server.as_str(), config.port),
        &config.server,
        async_native_tls::TlsConnector::new(),
    )
    .await?;
    println!("email: connected to {}:{}", config.server, config.port);

    let mut session = client
        .login(&config.username, &config.password)
        .await
        .map_err(|e| e.0)?;
    println!("email: logged in as {}", config.username);

    let mailbox = session.select("INBOX").await?;
    let mut current_seq = mailbox.exists;

    println!("email: INBOX selected; highest seq: {current_seq}");

    let mut idle = session.idle();

    loop {
        idle.init().await?;
        println!("email: initialized idle");
        let (idle_wait, _interrupt) = idle.wait();
        let idle_res = idle_wait.await?;
        println!("email: idle resolved {:?}", idle_res);

        match idle_res {
            idle::IdleResponse::NewData(x) => {
                if let imap_proto::Response::MailboxData(imap_proto::MailboxDatum::Exists(
                    new_seq,
                )) = x.parsed()
                {
                    session = idle.done().await?;

                    println!("email: new highest seq: {new_seq}, catching up...");
                    let seq_set = format!("{}:{}", current_seq + 1, new_seq);
                    current_seq = *new_seq;

                    {
                        let mut msgs = session
                            .fetch(&seq_set, "(BODY[TEXT] BODY[HEADER.FIELDS (To)])")
                            .await?;

                        while let Some(msg) = msgs.try_next().await? {
                            if let Some((to, code)) = parse_verification(&msg) {
                                println!("email: to = {to}");
                                println!("email: code = {code}");

                                if let Some((_, tx)) = verify_queue.remove(&to) {
                                    match tx.send(code.into()) {
                                        Ok(_) => println!("code sent"),
                                        Err(_) => println!("error sending code"),
                                    }
                                }
                            } else {
                                println!("email: couldn't parse {}", msg.message);
                            }
                        }
                    }

                    let _ = session.store(&seq_set, "+FLAGS.SILENT (\\seen)").await?;

                    idle = session.idle();
                }
            }
            _ => {}
        }
    }
}
