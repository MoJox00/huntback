use clap::{command, Arg, ArgAction};
use reqwest::Client;
use serde_json::Value;
use std::fs::File;
use std::io::{Write};

#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {

    let arguments = command!()
        .about("Retrive waybackurls and saves them for you :D")
        .author(":D")
        .version("1.0.0")
        .arg(
            Arg::new("DOMAIN")
                .short('d')
                .long("domain")
                .num_args(1)
                .required(true)
                .help("Domain here"),
        )
        .arg(
            Arg::new("SUBDOMAIN")
                .short('s')
                .long("subdomain")
                .required(false)
                .num_args(0)
                .action(ArgAction::Set)
                .help("To fetch subdomains from the URLs."),
        )
        .get_matches();


    let target_domain = arguments.get_one::<String>("DOMAIN");
    
    let subdomain_prefix = match arguments.contains_id("SUBDOMAIN") {
        true => "*.".to_string(),
        false => "".to_string(),
    };
    
    let (common_crawl_url, wayback_url) = (get_common_crawl_url(target_domain, &subdomain_prefix), get_wayback_url(target_domain, &subdomain_prefix));

    let (u1, u2) = tokio::join!(common_crawl_url, wayback_url);

    let mut urls = String::new();
    urls.push_str(u1.unwrap().as_str());
    urls.push_str(u2.unwrap().as_str());
    urls.push('\n');

    // Adding a println! statement for debugging
    println!("The collected URLs are: {}", urls);

    match File::create("urls.txt") {
        Ok(mut file) => {
            match &file.write(urls.as_bytes()) {
                Ok(_) => {
                    println!("Urls are written to file.")
                },
                Err(_) => {
                    println!("Urls are writting error to file.")
                }
            }
        },
        Err(_) => {
            println!("File creation error");
        }
    }
    Ok(())
}

async fn get_common_crawl_url(target_domain: Option<&String>, subdomain_prefix: &String) -> Result<String, reqwest::Error>{

    let target_url = match target_domain {
        Some(target_domain) => {
            format!("https://index.commoncrawl.org/CC-MAIN-2018-22-index?url={subdomain_prefix}{target_domain}/*&output=json")
        },
        _ => {
            String::from("Invalid domain")
        }
    };

    let response = Client::new().get(&target_url).send().await?.text().await?;
    
    let collect: Vec<_> = response.split("\n").collect();
    let mut urls = String::new();
    for json in collect {
        match serde_json::from_str::<Value>(json) {
            Ok(url) => {
                match url["url"].as_str() {
                    Some(target_url) => {
                        urls.push_str(&target_url);
                        urls.push('\n');
                    },
                    None => continue
                }
            }
            Err(_) => {
                continue;
            }
        };
    }
    Ok(urls)
}

async fn get_wayback_url(target_domain: Option<&String>, subdomain_prefix: &String) -> Result<String, reqwest::Error> {

    let target_url = match target_domain {
        Some(target_domain) => {
            format!("http://web.archive.org/cdx/search/cdx?url={subdomain_prefix}{target_domain}/*&output=json&fl=original&collapse=urlkey", )
        },
        _ => {
            String::from("Invalid domain")
        }
    };

    let response = Client::new().get(&target_url).send().await?.text().await?;
    let mut array_response: Vec<&str> = response.lines().collect();
    array_response.remove(0); // Remove the first line
    let urls = array_response
        .join("\n") // Join the remaining lines back into a single string
        .replace("[", "")
        .replace("]", "")
        .replace("\"", "")
        .replace(",", "");
    Ok(urls)
}