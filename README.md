## URL
Extract specific parts of URLs  
Fast Alternative for [unfurl](https://github.com/tomnomnom/unfurl)

### Compare
```bash
# input stdin and parameters
echo "foo.com" | url domain "bar.com" 
# bar.com
# foo.com

# Just print icann domains
echo example.domain | unfurl domain # example.domain
echo example.domain | url domain # 

# Recognize schemeless patterns
echo user:pass@example.com | unfurl domain #
echo user:pass@example.com | url domain # example.com
```

### Benchmark
```bash
hyperfine 'url v "user:pass@www.domain.tld/path?l=p&p=o#s"' 'echo  "user:pass@www.domain.tld/path?l=p&p=o#s" | unfurl values'


# ...Summary
  url v "user:pass@www.domain.tld/path?l=p&p=o#s" ran
    6.14 ± 4.53 times faster than echo  "user:pass@www.domain.tld/path?l=p&p=o#s" | unfurl values

```

### Install
- Install [Rust](https://www.rust-lang.org/tools/install)  
- `git clone https://github.com/xaoirse/url`
- `cd url`
- `cargo build --release`
- `./target/release/url --help`

### HELP
```bash
url --help

Usage: url <PATTERN> [ARGS]...

Arguments:
  <PATTERN>  %s | scheme
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
  [ARGS]...  

Options:
  -h, --help     Print help
  -V, --version  Print version
```

### TODO
- [ ] Tests  
- [ ] JSON  
