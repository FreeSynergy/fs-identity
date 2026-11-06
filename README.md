# fs-identity

The identity wrapper of FreeSynergy: signing in, and access to systems. We define how it behaves; Kanidm carries it out where it can (ADR 0047).

> **The name `fs-identity` is provisional.** The docs say "name still open" (`docs/en/programs/platform/identity.md`); renaming the repo and crate is cheap now and costly later.

* `crates/fs-identity-kanidm`: the Kanidm adapter of the `GroupDirectory` interface of [fs-groups](https://github.com/FreeSynergy/fs-groups). REST API v1 with a service-account Bearer token, through our own small client. Not built: SCIM (ADR 0047), PAM, RADIUS, sign-in flows. See `DESIGN.md`.

Licence: MIT OR Apache-2.0.
