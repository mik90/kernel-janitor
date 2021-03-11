# kernel-janitor
Helper for building, installing, and cleaning kernel installations. Specifically made to deal with Gentoo. 
I could use genkernel but instead I rolled my own. Originally it was fairly easy to build/install/cleanup but then I wanted
to automate more. I wrote a script in Python that became beefier and now I'm rewriting it in Rust.

### TODO
* [x] use builder pattern to create configurable search
* [ ] Parse config files
* [ ] Parse command line input 
* [ ] Logic for deleting old kernels and related files
* [ ] Add a `keep` flag so that certain kernels won't be removed
