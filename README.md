# Parler Account Generator

This sadly doesn't work anymore because Parler fixed their captcha check.

## Credits

SHOUT OUT TO [Overlisted](https://overlisted.net/) FOR MAKING EVERYTHING WORK!!!!!!!

## Pre-requisites

- The Rust Compiler
- A mail server with IMAP
- A MongoDB database

## Usage

1. Get a mail server running with a wildcard MX record pointed at it. Example: `*.example.com` -> `mail.example.com`.
2. Set the IMAP account details in a `.env` file.
3. in `src/info.rs` add the domains you forwarded to the mail server to the `DOMAIN` array.
4. _Optional: If you want to use proxies, uncomment the `.proxy(reqwest::Proxy::https("..."))` line in `src/main.rs` and set the proxy URL._
5. Build the project with `cargo build --release`.
6. Run the project. Example: `./target/release/parler-account-generator 10 true`.

There is also a Docker version which is recommended for long term use because the generator randomly panics.

## Disclaimer

This is for educational purposes and you should definitely not use this to spam Parler. I am not responsible for any damage you cause with this tool.
