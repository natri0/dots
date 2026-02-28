# my personal dotfiles repo

currently contains dotfiles for:
- [`avalanche`](./avalanche/configuration.nix): my small upcloud vps running nixos

also contains the source to some small utilities:

## tinycd

[`tinycd`](./tinycd.rs) is a *tiny* continuous delivery Thing, written in a single Rust file.

### building

it requires nightly Cargo and can be build with this command:
```sh
cargo build -Zscript -Zunstable-options --manifest-path tinycd.rs --artifact-dir .
```

this outputs a `tinycd` executable in the current directory.

### running

either run it directly (e.g. `utils/tinycd.rs CONFIG_PATH`) or run the executable from the previous step.
it expects a config file as the 1st and only parameter, that looks smth like this:
```toml
listen-addr = "some.ip.addr"
port = some-port

base-dir = "folder with the git repo. will be pulled on every command run"

pubkey = "ed25519 pubkey, hex"

[commands.command-name]
command = "echo test"
workdir = "this field is optional"
```
