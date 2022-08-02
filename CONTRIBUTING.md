# Contributing code to `rust-opa-wasm`

Everyone is welcome to contribute code to `rust-opa-wasm`, provided that they are willing to license their contributions under the same license as the project itself.
We follow a simple 'inbound=outbound' model for contributions: the act of submitting an 'inbound' contribution means that the contributor agrees to license the code under the same terms as the project's overall 'outbound' license - in this case, Apache Software License v2 (see [LICENSE](./LICENSE)).

## How to contribute

The preferred and easiest way to contribute changes to the project is to fork it on GitHub, and then create a pull request to ask us to pull your changes into our repo.

We use GitHub's pull request workflow to review the contribution, and either ask you to make any refinements needed or merge it and make them ourselves.

Things that should go into your PR description:

 - References to any bugs fixed by the change
 - Notes for the reviewer that might help them to understand why the change is necessary or how they might better review it

Your PR must also:

 - be based on the `main` branch
 - adhere to the [code style](#code-style)
 - pass the [test suite](#tests)
 - include a [sign off](#sign-off)

## Tests

Integration tests require to compile Open Policy Agent policies to WebAssembly.
It can be done by running `make build-opa` from the project root directory.
It requires the [`opa`](https://www.openpolicyagent.org/docs/latest/#running-opa) CLI tool to be available.

Running the integration tests require all the features to be enabled, so run them with

```sh
cargo test --all-features
```

The integration tests leverage snapshots with [`cargo-insta`](https://insta.rs/).

## Code style

We use the standard Rust code style, and enforce it with `rustfmt`/`cargo fmt`.
A few code style options are set in the [`.rustfmt.toml`](./.rustfmt.toml) file, and some of them are not stable yet and require a nightly version of rustfmt.

If you're using [`rustup`](https://rustup.rs), the nightly version of `rustfmt` can be installed by doing the following:

```
rustup component add rustfmt --toolchain nightly
```

And then format your code by running:

```
cargo +nightly fmt
```

---

We also enforce some code style rules via [`clippy`](https://github.com/rust-lang/rust-clippy).

Some of those rules are from the `clippy::pedantic` ruleset, which can be sometime too restrictive.
There are legitimate reasons to break some of those rules, so don't hesitate to allow some of them locally via the `#[allow(clippy::name_of_the_rule)]` attribute.

Make sure to have Clippy lints also pass both with all the features flag enabled and with none of them:

```sh
cargo clippy --bins --tests
cargo clippy --bins --tests --all-features
cargo clippy --bins --tests --no-default-features
```

## Sign off

In order to have a concrete record that your contribution is intentional and you agree to license it under the same terms as the project's license, we've adopted the same lightweight approach that the Linux Kernel (https://www.kernel.org/doc/Documentation/SubmittingPatches), Docker (https://github.com/docker/docker/blob/master/CONTRIBUTING.md), and many other projects use: the DCO (Developer Certificate of Origin: http://developercertificate.org/).
This is a simple declaration that you wrote the contribution or otherwise have the right to contribute it to Matrix:

```
Developer Certificate of Origin
Version 1.1

Copyright (C) 2004, 2006 The Linux Foundation and its contributors.
660 York Street, Suite 102,
San Francisco, CA 94110 USA

Everyone is permitted to copy and distribute verbatim copies of this
license document, but changing it is not allowed.

Developer's Certificate of Origin 1.1

By making a contribution to this project, I certify that:

(a) The contribution was created in whole or in part by me and I
    have the right to submit it under the open source license
    indicated in the file; or

(b) The contribution is based upon previous work that, to the best
    of my knowledge, is covered under an appropriate open source
    license and I have the right under that license to submit that
    work with modifications, whether created in whole or in part
    by me, under the same open source license (unless I am
    permitted to submit under a different license), as indicated
    in the file; or

(c) The contribution was provided directly to me by some other
    person who certified (a), (b) or (c) and I have not modified
    it.

(d) I understand and agree that this project and the contribution
    are public and that a record of the contribution (including all
    personal information I submit with it, including my sign-off) is
    maintained indefinitely and may be redistributed consistent with
    this project or the open source license(s) involved.
```

If you agree to this for your contribution, then all that's needed is to include the line in your commit or pull request comment:

```
Signed-off-by: Your Name <your@email.example.org>
```

We accept contributions under a legally identifiable name, such as your name on government documentation or common-law names (names claimed by legitimate usage or repute).
Unfortunately, we cannot accept anonymous contributions at this time.

Git allows you to add this signoff automatically when using the `-s` flag to `git commit`, which uses the name and email set in your `user.name` and `user.email` git configs.

If you forgot to sign off your commits before making your pull request and are on Git 2.17+ you can mass signoff using rebase:

```
git rebase --signoff origin/main
```
