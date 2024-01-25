## URL
Alternative for unfurl

### Compare:
```bash
# No icann domain
echo example.domain | unfurl domain # example.domain
echo example.domain | url domain # 

# authority
echo user:pass@example.com | unfurl domain #
echo user:pass@example.com | url domain # example.com
 
```