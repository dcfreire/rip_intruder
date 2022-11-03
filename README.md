# rip_intruder

This program is intended to be a viable alternative to Burp Suite's Intruder. Eventually implementing all of its most relevant features.
In its current state the only supported attack type is the "Battering Ram" attack type, where using a single set of payloads it places the same payload at all defined payload positions.

Do note that this is still in its very early stages of development, but it is already much faster than
Burp Suite **Community** Edition's Intruder.

```
Usage: rip_intruder [OPTIONS] <REQ_F> <PASS_F>

Arguments:
  <REQ_F>   Path to request template file
  <PASS_F>  Path to password file

Options:
  -c, --concurrent-requests <CONCURRENT_REQUESTS>
          Number of concurrent requests [default: 1]
  -p, --pattern <PATTERN>
          Regex pattern [default: §§]
      --hit-type <HIT_TYPE>
          What is considered a hit [default: ok] [possible values: ok, all]
  -o <OF>
          Output to file
  -s <STOP>
          Stop after n hits, -1 to try all provided words [default: 1]
      --format <OUT_FORMAT>
          Output format [default: csv] [possible values: csv, jsonl]
  -h, --help
          Print help information
  -V, --version
          Print version information
```

Example usage:
```
$ cargo run sample_req /usr/share/seclists/Passwords/Common-Credentials/best1050.txt -c 5 -p "§.*§" --hit-type all -s 200 -o test.jsonl --format jsonl
$ bat test.jsonl
───────┬────────────────────────────────────────────────────────────────────────────
       │ File: test.jsonl
───────┼────────────────────────────────────────────────────────────────────────────
   1   │ {"Body":"Invalid email or password.","Payload":"00000","Status":401}
   2   │ {"Body":"Invalid email or password.","Payload":"------","Status":401}
   3   │ {"Body":"Invalid email or password.","Payload":"0000000","Status":401}
   4   │ {"Body":"Invalid email or password.","Payload":"0","Status":401}
   5   │ {"Body":"Invalid email or password.","Payload":"000000","Status":401}
   6   │ {"Body":"Invalid email or password.","Payload":"00000000","Status":401}
   7   │ {"Body":"Invalid email or password.","Payload":"0987654321","Status":401}
   8   │ {"Body":"Invalid email or password.","Payload":"1111","Status":401}
   9   │ {"Body":"Invalid email or password.","Payload":"1","Status":401}
  10   │ {"Body":"Invalid email or password.","Payload":"11111","Status":401}
  11   │ {"Body":"Invalid email or password.","Payload":"111111","Status":401}
  12   │ {"Body":"Invalid email or password.","Payload":"1111111","Status":401}
  13   │ {"Body":"Invalid email or password.","Payload":"11111111","Status":401}
:
$ cat test.jsonl | grep "\"Status\":200"
{"Body":"{\"authentication\":{\"token\":\"eyJ0eXAiOiJKV1QiLCJhbGciOiJSUzI1NiJ9.eyJzdGF0dXMiOiJzdWNjZXNzIiwiZGF0YSI6eyJpZCI6MSwidXNlcm5hbWUiOiIiLCJlbWFpbCI6ImFkbWluQGp1aWNlLXNoLm9wIiwicGFzc3dvcmQiOiIwMTkyMDIzYTdiYmQ3MzI1MDUxNmYwNjlkZjE4YjUwMCIsInJvbGUiOiJhZG1pbiIsImRlbHV4ZVRva2VuIjoiIiwibGFzdExvZ2luSXAiOiIwLjAuMC4wIiwicHJvZmlsZUltYWdlIjoiYXNzZXRzL3B1YmxpYy9pbWFnZXMvdXBsb2Fkcy9kZWZhdWx0LnN2ZyIsInRvdHBTZWNyZXQiOiIiLCJpc0FjdGl2ZSI6dHJ1ZSwiY3JlYXRlZEF0IjoiMjAyMi0xMS0wMyAyMzowMDoyMC4yMTcgKzAwOjAwIiwidXBkYXRlZEF0IjoiMjAyMi0xMS0wMyAyMzowMDoyMC4yMTcgKzAwOjAwIiwiZGVsZXRlZEF0IjpudWxsfSwiaWF0IjoxNjY3NTE4MzM0LCJleHAiOjE2Njc1MzYzMzR9.QiGr6Ay1Jz9kA9QBULChJMZW0M0CbnFFhni0AevH-2GvcPrKzDIL0e8Gz6nK4fU6yQXLAUt5dj5SWFAwC78g2M0C2FOPlYDpxagp-odfODk4VfcItMNBVVCtbedeCHX3nu0LDZWOpD5pIAJkq20n9SielfF-IZCzSoqppc1_tjM\",\"bid\":1,\"umail\":\"admin@juice-sh.op\"}}","Payload":"admin123","Status":200}
```
