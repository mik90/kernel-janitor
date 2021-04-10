# kernel-janitor
Helper for building, installing, and cleaning kernel installations. Specifically made to deal with Gentoo. 
The only dependency is on the `libc` crate.

I could use sys-kernel/genkernel but instead I rolled my own. Originally it was fairly easy to build/install/cleanup but then I wanted
to automate more. I wrote a script in Python that became beefier and now I'm rewriting it in Rust.

## Usage
List installed kernels
```bash
user $ cargo run --release -- --list
```

Pretend to execute upgrade and clean (dry-run)
```bash
user $ cargo run --release -- --pretend
```

Run upgrade and clean (must be root)
```bash
root $ cargo run --release
```

Run upgrade and clean, skip copying of config in case manual edits were made.
Expects an updated `.config` to exist in the kernel source directory
```bash
root $ cargo run --release --manual-edit
```


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
    - [x] Logic for deleting old kernels and related files
    - [x] Check exit code of commands
    - [ ] tested
        - only tested manually so far, no unit tests
* [x] Use program specific error class instead of `Box<dyn std::error::Error>`
    - using `error::JanitorError`
* [x] If the latest version only has a source directory, that's fine since it'll happen whenever a new version is downloaded
    - is this still an issue?
    - Seems fixed
* [x] Print subcommand output as it runs
* [x] Create `Command` wrapper that allows for easier running of `pretend`
* [x] Deal with symlinks in conf file search
* [x] Grab `HOME` from environment variable, unsure why this wasn't working
* [ ] Actions should be interactive by default
* [ ] Deal with non-symlinks in ~/.config file search

### Ideas
* [ ] Allow for using trash-cli or just moving files to trash folder
* [ ] Create `move` func that can move directories or files. Either copy and remove the content
      or use `mv` as a `Command`. Renaming won't work across mount points.
* [ ] Add getter for InstalledKernel that returns paths without options if none are missing