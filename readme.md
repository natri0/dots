# my personal dotfiles repo

currently contains dotfiles for:
- [`avalanche`](./avalanche/configuration.nix): my small upcloud vps running nixos

also contains the source to some small utilities:

## tinycd

[`tinycd`](./utils/tinycd.rs) is a *tiny* continuous delivery Thing, written in a single Rust file.

### running

> [!NOTE]
> nightly Cargo is required. the tool was tested with `cargo 1.95.0-nightly (f298b8c82 2026-02-24)`; if using older versions, ymmv.

either run it directly (e.g. `utils/tinycd.rs CONFIG_PATH`) or run the executable from the previous step.
it expects a config file as the 1st and only parameter. for an example, see [`utils/tinycd.example.toml`](./utils/tinycd.example.toml).

### usage

> [!NOTE]
> all subprocess stdio is sent to the `tinycd` stdio. what this means is if your ssh key has a passphrase and you're testing locally, you'll need to enter it.
>
> for production(-ish) deployments, using something like [github deploy keys](https://docs.github.com/en/authentication/connecting-to-github-with-ssh/managing-deploy-keys#deploy-keys) should be good, as those are read-only *and* scoped to a single repo so should be fine to leave passwordless.
>
> also, this should go without saying but please don't run this as `root` or your privileged user. make a separate `tinycd` user, and whitelist some specific commands in your sudoers if it really needs to run privileged commands.
> i tried my best to make it secure and the only user input it takes is verified against a narrow set of possible variants (i.e. the command names you set in the config) *and* uses a secure signature algorithm, but you can never be too safe.

first, generate an ed25519 keypair. you can use openssl for that:
```sh
openssl genpkey -algorithm ed25519 -out private.pem # for signing
openssl pkey -in private.pem -pubout -outform DER | tail -c 32 | xxd -p -c 32 # the output is the pubkey; put in the .toml `pubkey` field
```

then, generate a timestamp and a signature:
```sh
timestamp=$(date -u +"%Y-%m-%dT%H:%M:%SZ") # generate timestamp. timezone doesn't matter, UTC here for simplicity
echo $timestamp
echo -n "test:$timestamp" | openssl pkeyutl -sign -inkey private.pem -in /dev/stdin | xxd -p -c 64
```

pass the signature in the `curl` command:
```sh
curl -i --header 'CD-Signature: SIGNATURE_HERE' --header 'CD-Timestamp: TIMESTAMP_HERE' http://localhost:6969/run/test
```

if successful, you'll see the output of both the `git pull` and the configured command in the `tinycd` output, and a small message in your `curl`:
```
% curl -i --header 'CD-Signature: 10c17a3431440c7ef51acbbc94613bc52524610b0f0e893a253b90c9c5db82abb84de84040bc27ccb333340049885acd0d42571abca4046f7d15de2b47fbff04' --header 'CD-Timestamp: 2026-03-01T13:48:34Z' localhost:6969/run/test
HTTP/1.1 200 OK
content-type: text/plain; charset=utf-8
content-length: 16
date: Sun, 01 Mar 2026 13:49:12 GMT

ran successfully
```
