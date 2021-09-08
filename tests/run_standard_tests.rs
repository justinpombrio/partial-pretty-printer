// Weird directory structure to prevent Rust from re-compiling the shared test code for every test
// file. Is there a better way to do this?

#![feature(test)]
#![feature(shrink_to)]

mod standard;
