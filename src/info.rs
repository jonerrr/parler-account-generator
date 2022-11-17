use fake::{
    faker::{
        lorem::en::{Word, Words},
        name::en::*,
        number::en::NumberWithFormat,
    },
    Fake,
};
use rand::prelude::*;
use serde::{Deserialize, Serialize};
use std::{path, str};

const INTERESTS: &[&str; 27] = &[
    "Arts and Entertainment",
    "Automotive",
    "Beauty",
    "Books and Literature",
    "Business",
    "Comedy",
    "Family",
    "Fashion",
    "Food and Drink",
    "Gaming",
    "Health and Fitness",
    "Home and Garden",
    "Humor",
    "Law, Government, and Politics",
    "Lifestyle",
    "Movies and Television",
    "Music",
    "News",
    "Outdoor Recreation",
    "Personal Finance",
    "Pets",
    "Philosophy",
    "Photography",
    "Random",
    "Recreation and Sports",
    "Science and Technology",
    "Travel",
];

// Domains for emails
const DOMAINS: &[&str] = &[
    "among-d.rip",
    "committed-suici.de",
    "daba.by",
    "is-a-sk.id",
    "me-p.lease",
    "idtlabs.net",
    "swatting.party",
    "taxevasion.rocks",
    "zingel300mgodpot.us",
];

fn capitalize(string: &str) -> String {
    format!(
        "{}{}",
        string[0..1].to_ascii_uppercase(),
        &str::to_lowercase(string)[1..]
    )
}

fn randomly_capitalize_words(rng: &mut impl rand::Rng, string: &str) -> String {
    let first = rng.gen_bool(0.9);
    let rest = first && rng.gen_bool(0.5);

    string
        .split(' ')
        .enumerate()
        .map(|(i, x)| {
            if i == 0 && first {
                capitalize(x)
            } else if i != 0 && rest {
                capitalize(x)
            } else {
                str::to_lowercase(x)
            }
        })
        .collect()
}

const CITY_FORMATS: &[fn(&mut rand::rngs::ThreadRng, &str) -> String] = &[
    |rng, x| {
        format!(
            "{}, {}",
            randomly_capitalize_words(rng, "israel"),
            randomly_capitalize_words(rng, x),
        )
    },
    |rng, x| {
        format!(
            "{} {}",
            randomly_capitalize_words(rng, "israel"),
            randomly_capitalize_words(rng, x),
        )
    },
    |rng, x| {
        format!(
            "{}, {}",
            randomly_capitalize_words(rng, x),
            randomly_capitalize_words(rng, "israel"),
        )
    },
    |rng, x| randomly_capitalize_words(rng, x),
    |rng, x| format!("{} ❤️", randomly_capitalize_words(rng, x)),
    |rng, _| format!("{} ❤️", randomly_capitalize_words(rng, "israel")),
    |rng, _| randomly_capitalize_words(rng, "israel"),
    |rng, _| randomly_capitalize_words(rng, "in israel"),
];

#[derive(Debug, Serialize, Deserialize)]
pub struct Info {
    pub email: String,
    pub name: String,
    pub username: String,
    pub password: String,
    pub captcha_key: String,
    pub interests: String,
    // pub pfp: path::PathBuf,
    // pub bio: String,
    pub location: String,
}

impl Info {
    pub fn new_random(resources: &crate::Resources) -> Self {
        let mut rng = rand::thread_rng();
        let name: (String, String) = (FirstName().fake(), LastName().fake());

        Info {
            email: format!(
                "{}{}@{}.{}",
                name.0,
                name.1,
                Word().fake::<String>(),
                DOMAINS[rng.gen_range(0..DOMAINS.len())]
            ),
            name: format!("{} {}", name.0, name.1),
            username: format!(
                "{}{}{}",
                name.0,
                name.1,
                NumberWithFormat("###").fake::<String>()
            ),
            password: format!("aosi{}djsdi0g@#odsfA", Word().fake::<String>()).to_string(),
            captcha_key: Words(1..10).fake::<Vec<String>>().join(""),
            interests: INTERESTS[rng.gen_range(0..INTERESTS.len())].to_string(),
            // pfp: resources.pfps.choose(&mut rng).unwrap().to_path_buf(),
            // bio: {
            //     let string = resources.sefaria_quotes.choose(&mut rng).unwrap();
            //     string[..string.as_bytes().len().min(100) - 1].into()
            // },
            location: CITY_FORMATS.choose(&mut rng).unwrap()(
                &mut rng.clone(),
                &resources.cities.choose(&mut rng).unwrap(),
            ),
        }
    }
}
