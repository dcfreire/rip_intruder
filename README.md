# rip_intruder

This program is intended to be a viable alternative to Burp Suite's Intruder. Eventually implementing all of its most relevant features. 
Right now it can only apply the payloads to the body. 

There are a lot of optimizations to be made still, such as not building a request from scratch every iteration, but it is already much faster than
Burp Suite **Community** Edition's Intruder. Do note that this is still in its very early stages of development.

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
