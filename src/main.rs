extern crate hyper;
extern crate hyper_native_tls;
extern crate url;
extern crate serde_json;

use hyper::Client;
use hyper::client::Response;
use hyper::header::{Accept, Authorization, Headers, UserAgent, qitem};
use hyper::mime::Mime;
use hyper::net::HttpsConnector;
use hyper_native_tls::NativeTlsClient;
use std::env;
use url::Url;

fn build_headers() -> Headers {
    let mut headers = Headers::new();

    let mime: Mime = "application/vnd.github.v3+json".parse().unwrap();
    headers.set(Accept(vec![qitem(mime)]));

    let token = env::var("TOKEN").unwrap();
    headers.set(Authorization(format!("token {}", token)));

    let product = "ghrepos";
    let version = "0.1.0";
    headers.set(UserAgent(format!("{}/{}", product, version)));

    headers
}

fn build_url(page: i32) -> Url {
    Url::parse("https://api.github.com")
        .and_then(|url| url.join("/user/repos"))
        .and_then(|ref mut url| {
                      url.query_pairs_mut()
                          .append_pair("affiliation", "owner,collaborator")
                          .append_pair("per_page", "100")
                          .append_pair("page", &page.to_string());
                      Ok(url.to_owned())
                  })
        .unwrap()
}

fn http_get(url: Url, headers: Headers) -> Response {
    let ssl = NativeTlsClient::new().unwrap();
    let connector = HttpsConnector::new(ssl);
    let client = Client::with_connector(connector);
    client.get(url).headers(headers).send().unwrap()
}

fn parse(response: Response) -> Vec<String> {
    let json: serde_json::Value = serde_json::from_reader(response).unwrap();
    let repos: &Vec<serde_json::Value> = json.as_array().unwrap();
    repos
        .into_iter()
        .map(|repo| repo.get("full_name").unwrap())
        .map(|full_name| full_name.as_str().unwrap())
        .map(|full_name_str| full_name_str.to_owned())
        .collect()
}

fn main() {
    let mut all_repo_names = vec![];
    let mut page = 1;
    loop {
        let url = build_url(page);
        let headers = build_headers();
        let response = http_get(url, headers);
        let repo_names = parse(response);
        if repo_names.len() == 0 {
            break;
        } else {
            all_repo_names.extend(repo_names.iter().cloned());
            page += 1;
        }
    }
    for repo_name in all_repo_names {
        println!("{}", repo_name);
    }
}
