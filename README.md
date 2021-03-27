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
    - [x] tested
* [x] Parse command line input
    - [x] handle flags
    - [x] print out help
    - [x] tested
* [x] Parse config files
    - [x] impl ini parsing
    - [x] Parse entry
    - ~~[ ] Parse section~~
        - not really needed
    - [x] Parse file
    - [x] Use enum to allow parse error, comment, entry
    - [x] ignore comments
    - [x] search for config files
    - [x] tested
* [x] Use `try_main`
    - reference `https://benhoyt.com/writings/count-words/`
* [x] Handle parsing version from module names which won't have 'linux-' prepended
* [x] impl `InstalledItem` and use it instead of a `(KernelVersion, InstalledItemKind, PathBuf)` tuple
* [x] Verions that are old shouldn't expect source dirs or module dirs with `.old` on them
* update.rs 
    - [x] copy config
    - [x] building kernel
    - [x] gen grub cfg or run portage commands
    - [ ] Logic for deleting old kernels and related files
    - [ ] tested
### Ideas
* [ ] Allow hooks that can be ran after building the kernel
    - Used for running `emerge @preserved-rebuild`
* [ ] Use program specific error class instead of `Box<dyn std::error::Error>`
* [ ] Create command wrapper that allows for easier running of `pretend`
* [ ] Add getter for InstalledKernel that returns paths without options if none are missing
* [ ] Allow for a `keep` flag on `InstalledKernel`s so that certain kernels won't be removed
    - [ ] tested