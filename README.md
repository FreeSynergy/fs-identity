# FreeSynergy Identity — know who is allowed in

**Today this is a small bridge, not yet a sign-in. We say so plainly.**

FreeSynergy Identity is meant to become the one place where signing in and access to systems are handled. We describe how it should behave, and Kanidm, an open identity server, does the work where it can. The name of this repo is provisional.

What exists now is one piece of that: an adapter that makes Kanidm mirror the groups of FreeSynergy Groups. Groups decides who belongs to what. Kanidm never decides. If you do not run Kanidm, all programs still run alone.

## What it can do today

Latest tag: `v0.0.1`.

- Create, read, update, list and remove groups in Kanidm so that they match FreeSynergy Groups, safely repeatable.
- Group names in Kanidm are derived from our ids, so it does not need custom attributes.
- Our own small client for Kanidm's REST interface, checked against Kanidm 1.12.0.
- Run once against a real Kanidm 1.12.0 container; the other 26 tests run offline against a stand-in.

## What is not there yet

- No sign-in, no passwords or credential flows, no OAuth2 clients, no SCIM, no PAM, no RADIUS.
- The adapter cannot create invitations or grant rights in Kanidm; it reports that as not possible.
- The people map is supplied by the caller.
- The name and shape of this repo are not final.

## Part of FreeSynergy

Identity is the bridge between FreeSynergy's own groups and a directory you may already run. FreeSynergy is a family of programs that are free for everyone, forever, and that keep your data yours. The documentation, the manifesto and the overall plan live in [fs-documentation](https://github.com/FreeSynergy/fs-documentation); the reading version is at <https://freesynergy.github.io/fs-documentation/>. Start with the [manifesto](https://freesynergy.github.io/fs-documentation/manifesto/) if you want to know why we do this.


## For developers

- Workspace crate: `fs-identity-kanidm`, an implementation of the `GroupDirectory` trait of [fs-groups](https://github.com/FreeSynergy/fs-groups). Kanidm's vocabulary stays inside this crate.
- Build and test: `cargo build --workspace` and `cargo test --workspace`. The live test is ignored by default.
- Design and the routes used: [DESIGN.md](DESIGN.md).
- Licence: MIT OR Apache-2.0.
