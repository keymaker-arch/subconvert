use base64::Engine;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    let sub_link: String;

    if args.len() == 1 {
        sub_link = String::from("https://dy.tagsub.net/api/v1/client/subscribe?token=801cd9b979676ac84b018debfd57567a");
    } else if args.len() == 2 {
        sub_link = args[1].clone();
    } else {
        println!("Usage: {} <SUB_LINK>", args[0]);
        return;
    }

    let res = match reqwest::blocking::get(sub_link) {
        Ok(res) => res,
        Err(e) => {
            println!("Failed to get subreddit: {}", e);
            return;
        }
    };
    let body = res.text().expect("Failed to get response body");

    let decoded_body = base64::engine::general_purpose::STANDARD.decode(body.as_bytes()).expect("Failed to decode body");
    let decoded_body = String::from_utf8(decoded_body).expect("Failed to convert body to string");
    let url_list = decoded_body.lines();

    for url in url_list {
        if url.starts_with("ss://") {
            println!("{}", url);
        }
    }
    println!(
        "Total {} links",
        url_list.clone().count()
    )
}