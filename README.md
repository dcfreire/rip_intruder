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
  -c, --concurrent-requests <CONCURRENT_REQUESTS>  Number of concurrent requests [default: 1]
  -h, --help                                       Print help information
  -V, --version                                    Print version information
```
