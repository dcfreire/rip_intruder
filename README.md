# rip_intruder

This program is intended to be a viable alternative to Burp Suite's Intruder. Eventually implementing all of its most relevant features. 
Right now it can only apply the payloads to the body. After building it you can use it with:
./rip_intruder [<template_request>] [<pass_file>]

There are a lot of optimizations to be made still, such as not building a request from scratch every iteration, but it is already much faster than
Burp Suite **Community** Edition's Intruder. Do note that this is still in its very early stages of development.
