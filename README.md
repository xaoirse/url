## URL
Extract specific parts of URLs  
Fast Alternative for [unfurl](https://github.com/tomnomnom/unfurl)

### Compare
```bash
# input stdin and parameters
echo "foo.com" | url domain "bar.com" 
# bar.com
# foo.com

# No icann domain
echo example.domain | unfurl domain # example.domain
echo example.domain | url domain # 

# Authority
echo user:pass@example.com | unfurl domain #
echo user:pass@example.com | url domain # example.com
```

### Benchmark
```bash
hyperfine 'url v "user:pass@www.domain.tld/path?l=p&p=o#s"' 'echo  "user:pass@www.domain.tld/path?l=p&p=o#s" | unfurl values'


# ...Summary
  ./target/release/url v "user:pass@www.domain.tld/path?l=p&p=o#s" ran
    6.14 ± 4.53 times faster than echo  "user:pass@www.domain.tld/path?l=p&p=o#s" | unfurl values

```

### Install
TODO

### HELP
TODO

### TODO
-[ ] Tests  
-[ ] JSON  
