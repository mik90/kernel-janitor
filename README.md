# kernel-janitor
Helper for building, installing, and cleaning kernel installations. Specifically made to deal with Gentoo. 
Has no dependencies.

I could use sys-kernel/genkernel but instead I rolled my own. Originally it was fairly easy to build/install/cleanup but then I wanted
to automate more. I wrote a script in Python that became beefier and now I'm rewriting it in Rust.

### Tasklist
* [x] Implement kernel version ordering
    - [x] tested
* [x] Implement kernel version searching
    - [x] use builder pattern to create configurable search
    - [ ] tested
* [ ] Parse command line input (In progress)
    - [ ] tested
* [ ] Parse config files
    * [ ] Allow hooks that can be ran after building the kernel
        - Used for running `emerge @preserved-rebuild`
    - [ ] tested
* [ ] Logic for deleting old kernels and related files
    - [ ] tested
* [ ] Allow for a `keep` flag on `InstalledKernel`s so that certain kernels won't be removed
    - [ ] tested
