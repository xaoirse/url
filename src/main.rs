use std::{
    error::Error,
    io::{IsTerminal, Read},
    str::FromStr,
};

use addr::parse_dns_name;
use clap::Parser;
use url::Url;

#[derive(Parser)]
#[clap(name = "URL",author, version, about, long_about = None)]
pub struct Opt {
    #[clap(short, long, global = true, help = "Quiet mode")]
    pub quiet: bool,

    pattern: String,
    args: Vec<String>,
}

struct Furl {
    url: Url,
    scheme: bool,
}

impl FromStr for Furl {
    type Err = Box<dyn Error>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut scheme = true;

        let url = if let Ok(url) = Url::from_str(s) {
            if url.cannot_be_a_base() {
                scheme = false;
                Url::from_str(&format!("https://{s}"))?
            } else {
                url
            }
        } else {
            scheme = false;
            Url::from_str(&format!("https://{s}"))?
        };

        Ok(Self { url, scheme })
    }
}

impl Furl {
    fn scheme(&self) -> Option<&str> {
        if self.scheme {
            Some(self.url.scheme())
        } else {
            None
        }
    }
    fn authority(&self) -> Option<&str> {
        Some(self.url.authority())
    }
    fn username(&self) -> Option<&str> {
        Some(self.url.username())
    }
    fn password(&self) -> Option<&str> {
        self.url.password()
    }
    fn get_domain(&self) -> Option<addr::dns::Name<'_>> {
        self.url.domain().and_then(|d| parse_dns_name(d).ok())
    }
    fn domain(&self) -> Option<&str> {
        if let Some(domain) = self.get_domain() {
            if domain.is_icann() {
                return Some(domain.as_str());
            }
        }
        None
    }
    fn subdomain(&self) -> Option<&str> {
        if let Some(domain) = self.get_domain() {
            if domain.is_icann() {
                return domain.prefix();
            }
        }
        None
    }
    fn apex(&self) -> Option<&str> {
        if let Some(domain) = self.get_domain() {
            if domain.is_icann() {
                return domain.root();
            }
        }
        None
    }
    fn suffix(&self) -> Option<&str> {
        if let Some(domain) = self.get_domain() {
            if domain.is_icann() {
                return domain.suffix();
            }
        }
        None
    }
    fn path(&self) -> Option<&str> {
        Some(self.url.path())
    }
    fn query(&self) -> Option<&str> {
        self.url.query()
    }
    fn keys(&self) -> Option<&str> {
        todo!()
        // Some(
        //     self.url
        //         .query_pairs()
        //         .map(|pair| format!("{}\n", pair.0))
        //         .collect::<String>()
        //         .as_str(),
        // )
    }
    fn values(&self) -> Option<&str> {
        todo!()
        // Some(
        //     self.url
        //         .query_pairs()
        //         .map(|pair| format!("{}\n", pair.1))
        //         .collect::<String>()
        //         .as_str(),
        // )
    }
    fn fragment(&self) -> Option<&str> {
        self.url.fragment()
    }
}

fn main() {
    let opt = Opt::parse();

    let mut buf = String::new();
    if !std::io::stdin().is_terminal() {
        std::io::stdin().read_to_string(&mut buf).unwrap();
    }

    let f = match opt.pattern.as_str() {
        "s" | "scheme" | "schemes" => Furl::scheme,
        "a" | "authority" | "auth" => Furl::authority,
        "u" | "username" | "usernames" => Furl::username,
        "pass" | "password" | "passwords" => Furl::password,
        "d" | "domain" | "domains" => Furl::domain,
        "sub" | "subdomain" | "subdomains" => Furl::subdomain,
        "r" | "root" | "roots" | "apex" | "apexes" => Furl::apex,
        "suffix" | "tld" => Furl::suffix,
        "p" | "path" | "paths" | "pathes" => Furl::path,
        "q" | "query" | "queries" => Furl::query,
        "k" | "key" | "keys" => Furl::keys,
        "v" | "val" | "value" | "values" => Furl::values,
        "f" | "fragment" | "fragments" => Furl::fragment,

        _ => todo!(),
    };

    opt.args
        .iter()
        .map(String::as_str)
        .chain(buf.split_ascii_whitespace())
        .flat_map(Furl::from_str)
        .for_each(|furl| {
            if let Some(res) = f(&furl) {
                println!("{res}")
            }
        })
}
