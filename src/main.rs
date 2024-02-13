use std::{
    collections::BTreeSet,
    error::Error,
    io::{IsTerminal, Read},
    ops::Sub,
    str::FromStr,
};

use addr::parse_dns_name;
use clap::Parser;
// use itertools::Itertools;
use url::Url;

#[derive(Parser)]
#[clap(name = "URL", author, version)]
pub struct Opt {
    #[clap(help = "%s | scheme
%c | url-like with scheme (https is default)
%a | authority
%u | username
%x | password
%d | domain
%S | subdomain
%r | apex | root
%n | name (example.tld -> example)
%t | tld | suffix
%P | port
%p | path
%q | query
%f | fragment
%/ | Inserts a :// if scheme is specified
%@  Inserts an @ if user info is specified
%:  Inserts a colon if a port is specified
%?  Inserts a question mark if a query string exists
%#  Inserts a hash if a fragment exists
%%  A literal percent character
dedup
")]
    pattern: String,
    args: Vec<String>,
}
#[derive(Debug)]
struct Furl {
    url: Url,
    scheme: bool,
    port: String,
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

        let port = url
            .port_or_known_default()
            .map(|port| port.to_string())
            .unwrap_or_default();

        Ok(Self { url, scheme, port })
    }
}

impl Furl {
    fn scheme(&self) -> &str {
        self.url.scheme()
    }

    fn url(&self) -> &str {
        self.url.as_str()
    }

    fn authority(&self) -> &str {
        self.url.authority()
    }
    fn username(&self) -> &str {
        self.url.username()
    }
    fn password(&self) -> &str {
        self.url.password().unwrap_or_default()
    }
    fn get_domain(&self) -> Option<addr::dns::Name<'_>> {
        self.url.domain().and_then(|d| {
            parse_dns_name(d)
                .ok()
                .filter(|d| d.is_icann() || d.is_private())
        })
    }
    fn domain(&self) -> &str {
        if let Some(domain) = self.get_domain() {
            return domain.as_str();
        }
        ""
    }
    fn subdomain(&self) -> &str {
        if let Some(domain) = self.get_domain() {
            return domain.prefix().unwrap_or_default();
        }
        ""
    }
    fn apex(&self) -> &str {
        if let Some(domain) = self.get_domain() {
            return domain.root().unwrap_or_default();
        }
        ""
    }
    fn name(&self) -> &str {
        if let Some(domain) = self.get_domain() {
            if let Some(root) = domain.root() {
                return root
                    .trim_end_matches(domain.suffix().unwrap_or_default())
                    .trim_end_matches('.');
            }
        }
        ""
    }
    fn suffix(&self) -> &str {
        self.domain().rsplit_once('.').unwrap_or_default().1
    }

    fn port(&self) -> &str {
        self.port.as_str()
    }

    fn path(&self) -> &str {
        if !self.domain().is_empty() {
            self.url.path()
        } else if self.scheme {
            &self.url.as_str()[self.scheme().len() + 2..]
        } else {
            &self.url.as_str()[7..]
        }
    }
    fn query(&self) -> &str {
        self.url.query().unwrap_or_default()
    }
    fn keys(&self) -> &str {
        self.url
            .query_pairs()
            .for_each(|pair| println!("{}", pair.0));
        ""
    }
    fn values(&self) -> &str {
        self.url
            .query_pairs()
            .for_each(|pair| println!("{}", pair.1));
        ""
    }
    fn fragment(&self) -> &str {
        self.url.fragment().unwrap_or_default()
    }
    fn slash(&self) -> &str {
        if !self.scheme().is_empty() {
            "://"
        } else {
            ""
        }
    }
    fn at(&self) -> &str {
        if !self.username().is_empty() {
            "@"
        } else {
            ""
        }
    }
    fn colon(&self) -> &str {
        if !self.port().is_empty() {
            ":"
        } else {
            ""
        }
    }
    fn question(&self) -> &str {
        if !self.query().is_empty() {
            "?"
        } else {
            ""
        }
    }
    fn hashtag(&self) -> &str {
        if !self.fragment().is_empty() {
            "#"
        } else {
            ""
        }
    }

    fn format(&self, pat: &str) -> Option<String> {
        use aho_corasick::AhoCorasick;

        let patterns = &[
            "%s", "%c", "%a", "%u", "%x", "%d", "%S", "%r", "%n", "%t", "%P", "%p", "%q", "%f",
            "%/", "%@", "%:", "%?", "%#", "%%",
        ];
        let replace_with = &[
            self.scheme(),
            self.url(),
            self.authority(),
            self.username(),
            self.password(),
            self.domain(),
            self.subdomain(),
            self.apex(),
            self.name(),
            self.suffix(),
            self.port(),
            self.path(),
            self.query(),
            self.fragment(),
            self.slash(),
            self.at(),
            self.colon(),
            self.question(),
            self.hashtag(),
            "%",
        ];

        let ac = AhoCorasick::new(patterns);
        if let Ok(ac) = ac {
            Some(ac.replace_all(pat, replace_with))
        } else {
            None
        }
    }
    fn json(&self) -> &str {
        todo!()
    }
}

static FUNC: phf::Map<&'static str, fn(&Furl) -> &str> = phf::phf_map! {
    "s" => Furl::scheme,
    "scheme" => Furl::scheme,
    "schemes" => Furl::scheme,

    "c" => Furl::url,
    "url" => Furl::url,

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

    "n" => Furl::name,
    "name"=> Furl::name,
    "names"=> Furl::name,

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

    "json"=> Furl::json,
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

    if opt.pattern == "dedup" {
        let mut args = furls.collect::<Vec<_>>();
        args.sort();
        args.dedup_by(|a, b| {
            if a == b {
                b.url.query_pairs_mut().extend_pairs(a.url.query_pairs());
                true
            } else {
                false
            }
        });

        for f in args {
            println!("{}", f.url);
        }
    } else if let Some(func) = FUNC.get(&opt.pattern) {
        furls.for_each(|furl| {
            let res = func(&furl);
            if !res.is_empty() {
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

impl Ord for Furl {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self.scheme().cmp(other.scheme()) {
            std::cmp::Ordering::Equal => match self.authority().cmp(other.authority()) {
                std::cmp::Ordering::Equal => {
                    match (self.url.path_segments(), other.url.path_segments()) {
                        (Some(sp), Some(op)) => {
                            let sp = sp
                                .filter(|s| !s.chars().all(|c| c.is_numeric()))
                                .collect::<BTreeSet<_>>();
                            let op = op
                                .filter(|s| !s.chars().all(|c| c.is_numeric()))
                                .collect::<BTreeSet<_>>();

                            if sp.len() == op.len() {
                                let so = sp.sub(&op);
                                let os = op.sub(&sp);

                                if so.len() > 1 || os.len() > 1 {
                                    return so.cmp(&os);
                                } else {
                                    return std::cmp::Ordering::Equal;
                                }
                            }

                            sp.len().cmp(&op.len())
                        }
                        (None, None) => std::cmp::Ordering::Equal,
                        (Some(_), None) => std::cmp::Ordering::Greater,
                        (None, Some(_)) => std::cmp::Ordering::Less,
                    }
                }

                o => o,
            },
            o => o,
        }
    }
}

impl PartialOrd for Furl {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl PartialEq for Furl {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other).is_eq()
    }
}

impl Eq for Furl {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_str() {
        let a = Furl::from_str("https://test.com").unwrap();
        assert_eq!(a.url, url::Url::from_str("https://test.com").unwrap());
        assert!(a.scheme);
        assert_eq!(a.port, "443".to_string());

        let a = Furl::from_str("test.com").unwrap();
        assert_eq!(a.url, url::Url::from_str("https://test.com").unwrap());
        assert!(!a.scheme);
        assert_eq!(a.port, "443".to_string());

        let a = Furl::from_str("http://test.com:743").unwrap();
        assert_eq!(a.url, url::Url::from_str("http://test.com:743").unwrap());
        assert!(a.scheme);
        assert_eq!(a.port, "743".to_string());

        let a = Furl::from_str("test.com:743").unwrap();
        assert_eq!(a.url, url::Url::from_str("https://test.com:743/").unwrap());
        assert!(!a.scheme);
        assert_eq!(a.port, "743".to_string());
    }

    #[test]
    fn domain() {
        assert_eq!(
            Furl::from_str("https://test.com").unwrap().domain(),
            "test.com"
        );

        assert_eq!(Furl::from_str("test.com").unwrap().domain(), "test.com");

        assert_eq!(Furl::from_str("test.invalid").unwrap().domain(), "");

        assert_eq!(
            Furl::from_str("user:pass@test.com").unwrap().domain(),
            "test.com"
        );

        assert_eq!(
            Furl::from_str("test.com/foo/bar").unwrap().domain(),
            "test.com"
        );

        assert_eq!(Furl::from_str("foo/bar").unwrap().domain(), "");
    }

    #[test]
    fn domain2() {
        assert!(Furl::from_str("googleapis.com")
            .unwrap()
            .get_domain()
            .unwrap()
            .is_private());
    }

    #[test]
    fn furl_eq() {
        let a = Furl::from_str("test.com/a/b").unwrap();
        let b = Furl::from_str("test.com/a/b").unwrap();
        assert_eq!(a, b);

        let a = Furl::from_str("https://test.com/a/b").unwrap();
        let b = Furl::from_str("https://test.com/a/b").unwrap();
        assert_eq!(a, b);

        let a = Furl::from_str("test.com/a/b?k=v").unwrap();
        let b = Furl::from_str("test.com/a/b?j=r").unwrap();
        assert_eq!(a, b);

        let a = Furl::from_str("test.com/a/b").unwrap();
        let b = Furl::from_str("test.com/a/c").unwrap();
        assert_ne!(a, b);

        let a = Furl::from_str("test.com/a/b/c").unwrap();
        let b = Furl::from_str("test.com/a/c/d").unwrap();
        assert_ne!(a, b);

        let a = Furl::from_str("test.com/a/b/c").unwrap();
        let b = Furl::from_str("test.com/a/d").unwrap();
        assert!(a.gt(&b));

        let a = Furl::from_str("test.com/a/b/e").unwrap();
        let b = Furl::from_str("test.com/a/x/d").unwrap();
        assert!(a.lt(&b));

        let a = Furl::from_str("ftp://test.com/a/b/e").unwrap();
        let b = Furl::from_str("test.com/a/x/d").unwrap();
        assert!(a.lt(&b));
    }

    #[test]
    fn furl_ne() {
        let a = Furl::from_str("test.com/a/b").unwrap();
        let b = Furl::from_str("test.com/a/c/d/fs").unwrap();
        assert_ne!(a, b);

        let a = Furl::from_str("test.com/a/b").unwrap();
        let b = Furl::from_str("test.com/a/c").unwrap();
        assert_ne!(a, b);

        let a = Furl::from_str("test.com/a/b/c").unwrap();
        let b = Furl::from_str("test.com/a/c/d").unwrap();
        assert_ne!(a, b);
    }
}
