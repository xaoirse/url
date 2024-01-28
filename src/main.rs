use std::{
    error::Error,
    io::{IsTerminal, Read},
    str::FromStr,
};

use addr::parse_dns_name;
use clap::Parser;
use url::Url;

#[derive(Parser)]
#[clap(name = "URL", author, version)]
pub struct Opt {
    #[clap(help = "%s | scheme
%a | authority
%u | username
%x | password
%d | domain
%S | subdomain
%r | apex | root
%s | suffix
%P | port
%p | path
%q | query
%f | fragment
%@  Inserts an @ if user info is specified
%:  Inserts a colon if a port is specified
%?  Inserts a question mark if a query string exists
%#  Inserts a hash if a fragment exists
%%  A literal percent character
")]
    pattern: String,
    args: Vec<String>,
}

struct Furl {
    url: Url,
    scheme: bool,
    port: Option<String>,
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

        let port = url.port_or_known_default().map(|port| port.to_string());

        Ok(Self { url, scheme, port })
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
        if self.url.authority().is_empty() {
            None
        } else {
            Some(self.url.authority())
        }
    }
    fn username(&self) -> Option<&str> {
        if self.url.username().is_empty() {
            None
        } else {
            Some(self.url.username())
        }
    }
    fn password(&self) -> Option<&str> {
        self.url.password()
    }
    fn get_domain(&self) -> Option<addr::dns::Name<'_>> {
        self.url.domain().and_then(|d| parse_dns_name(d).ok())
    }
    fn domain(&self) -> Option<&str> {
        if let Some(domain) = self.get_domain() {
            if domain.root().is_some() && domain.is_icann() {
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

    fn port(&self) -> Option<&str> {
        self.port.as_deref()
    }

    fn path(&self) -> Option<&str> {
        if self.domain().is_some() {
            if self.url.path().is_empty() {
                None
            } else {
                Some(self.url.path())
            }
        } else if self.scheme {
            Some(&self.url.as_str()[self.scheme().map(|s| s.len() + 2).unwrap_or_default()..])
        } else {
            Some(&self.url.as_str()[8..])
        }
    }
    fn query(&self) -> Option<&str> {
        self.url.query()
    }
    fn keys(&self) -> Option<&str> {
        self.url
            .query_pairs()
            .for_each(|pair| println!("{}", pair.0));
        None
    }
    fn values(&self) -> Option<&str> {
        self.url
            .query_pairs()
            .for_each(|pair| println!("{}", pair.1));
        None
    }
    fn fragment(&self) -> Option<&str> {
        self.url.fragment()
    }
    fn at(&self) -> Option<&str> {
        self.username().map(|_| "@")
    }
    fn colon(&self) -> Option<&str> {
        self.port().map(|_| ":")
    }
    fn question(&self) -> Option<&str> {
        self.query().map(|_| "?")
    }
    fn hashtag(&self) -> Option<&str> {
        self.fragment().map(|_| "#")
    }

    fn format(&self, pat: &str) -> Option<String> {
        use aho_corasick::AhoCorasick;

        let patterns = &[
            "%s", "%a", "%u", "%x", "%d", "%S", "%r", "%t", "%P", "%p", "%q", "%f", "%@", "%:",
            "%?", "%#", "%%",
        ];
        let replace_with = &[
            self.scheme().unwrap_or_default(),
            self.authority().unwrap_or_default(),
            self.username().unwrap_or_default(),
            self.password().unwrap_or_default(),
            self.domain().unwrap_or_default(),
            self.subdomain().unwrap_or_default(),
            self.apex().unwrap_or_default(),
            self.suffix().unwrap_or_default(),
            self.port().unwrap_or_default(),
            self.path().unwrap_or_default(),
            self.query().unwrap_or_default(),
            self.fragment().unwrap_or_default(),
            self.at().unwrap_or_default(),
            self.colon().unwrap_or_default(),
            self.question().unwrap_or_default(),
            self.hashtag().unwrap_or_default(),
            "%",
        ];

        let ac = AhoCorasick::new(patterns);
        if let Ok(ac) = ac {
            Some(ac.replace_all(pat, replace_with))
        } else {
            None
        }
    }
}

static FUNC: phf::Map<&'static str, fn(&Furl) -> Option<&str>> = phf::phf_map! {
    "s" => Furl::scheme,
    "scheme" => Furl::scheme,
    "schemes" => Furl::scheme,

    "a"  => Furl::authority,
    "auth" => Furl::authority,
    "authority" => Furl::authority,

    "u"  => Furl::username,
    "user" => Furl::username,
    "users"  => Furl::username,
    "username" => Furl::username,
    "usernames" => Furl::username,

    "x" => Furl::password,
    "pass" => Furl::password,
    "password" => Furl::password,
    "passwords" => Furl::password,

    "d" => Furl::domain,
    "domain"=> Furl::domain,
    "domains" => Furl::domain,

    "S"=> Furl::subdomain,
    "sub"=> Furl::subdomain,
    "subdomain" => Furl::subdomain,
    "subdomains" => Furl::subdomain,

    "r" => Furl::apex,
    "root"=> Furl::apex,
    "roots"  => Furl::apex,
    "apex"=> Furl::apex,
    "apexes" => Furl::apex,

    "t" => Furl::suffix,
    "tld" => Furl::suffix,
    "suffix"=> Furl::suffix,

    "P" => Furl::port,
    "port" => Furl::port,
    "ports" => Furl::port,

    "p"=> Furl::path,
    "path"  => Furl::path,
    "paths" => Furl::path,

    "q" => Furl::query,
    "query"  => Furl::query,
    "queries" => Furl::query,

    "k"  => Furl::keys,
    "key" => Furl::keys,
    "keys" => Furl::keys,

    "v" => Furl::values,
    "val" => Furl::values,
    "value"  => Furl::values,
    "values" => Furl::values,

    "f"=> Furl::fragment,
    "fragment"=> Furl::fragment,
    "fragments" => Furl::fragment,


};

fn main() {
    let opt = Opt::parse();

    let mut stdin = String::new();
    if !std::io::stdin().is_terminal() {
        std::io::stdin().read_to_string(&mut stdin).unwrap();
    }

    let furls = opt
        .args
        .iter()
        .map(String::as_str)
        .chain(stdin.split_ascii_whitespace())
        .flat_map(Furl::from_str);

    if let Some(func) = FUNC.get(&opt.pattern) {
        furls.for_each(|furl| {
            if let Some(res) = func(&furl) {
                println!("{res}")
            }
        });
    } else {
        furls.for_each(|furl| {
            if let Some(res) = furl.format(opt.pattern.as_str()) {
                println!("{res}")
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_str() {
        let a = Furl::from_str("https://test.com").unwrap();
        assert_eq!(a.url, url::Url::from_str("https://test.com").unwrap());
        assert!(a.scheme);
        assert_eq!(a.port, Some("443".to_string()));

        let a = Furl::from_str("test.com").unwrap();
        assert_eq!(a.url, url::Url::from_str("https://test.com").unwrap());
        assert!(!a.scheme);
        assert_eq!(a.port, Some("443".to_string()));

        let a = Furl::from_str("http://test.com:743").unwrap();
        assert_eq!(a.url, url::Url::from_str("http://test.com:743").unwrap());
        assert!(a.scheme);
        assert_eq!(a.port, Some("743".to_string()));

        let a = Furl::from_str("test.com:743").unwrap();
        assert_eq!(a.url, url::Url::from_str("https://test.com:743/").unwrap());
        assert!(!a.scheme);
        assert_eq!(a.port, Some("743".to_string()));
    }

    #[test]
    fn domain() {
        assert_eq!(
            Furl::from_str("https://test.com").unwrap().domain(),
            Some("test.com")
        );

        assert_eq!(
            Furl::from_str("test.com").unwrap().domain(),
            Some("test.com")
        );

        assert_eq!(Furl::from_str("test.invalid").unwrap().domain(), None);

        assert_eq!(
            Furl::from_str("user:pass@test.com").unwrap().domain(),
            Some("test.com")
        );

        assert_eq!(
            Furl::from_str("test.com/foo/bar").unwrap().domain(),
            Some("test.com")
        );

        assert_eq!(Furl::from_str("foo/bar").unwrap().domain(), None);
    }
}
