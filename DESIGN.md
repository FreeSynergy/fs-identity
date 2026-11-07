# fs-identity – Design

`fs-identity` is the identity wrapper of FreeSynergy (name provisional). This repo holds the Kanidm adapter
(`crates/fs-identity-kanidm`). Kanidm is a mirror target, never the point of truth for groups (ADR 0047): fs-groups decides,
this crate brings Kanidm to that state. Without Kanidm, programs run alone (ADR 0024). Kanidm's vocabulary
(`member`, `description`, SPN, class) stays inside this crate; above it, only `fs_groups_core::GroupDirectory`.

## State

Version 0.1.0.

| Area | State |
|---|---|
| `KanidmDirectory`: `GroupDirectory` over REST v1 (`apply` idempotent and reading first, `remove`, `read`, `list`) | done |
| Own client (`reqwest` + rustls), routes checked against Kanidm v1.12.0 `server/core/src/https/v1.rs` | done |
| Key: group id <-> group name `fsg-<uuid>` (see below) | done |
| HTTP seam (`HttpSeam`) so tests need no server; a Kanidm stand-in in `tests/common` | done |
| Person map (`PersonMap`, `StaticPersonMap`) supplied by the caller | done |
| Capabilities report: `directory.apply`, `directory.read`; `invite.create` and `right.grant` unsupported with reasons | done |
| Run against a real Kanidm 1.12.0 (podman, throwaway container, own CA) | **done once**, see "Verified" |
| SCIM, PAM, RADIUS, sign-in / credential flows, OAuth2 clients, `kanidm_client` | **not built** |
| Tests | 26 offline + 1 live (`#[ignore]`) |

## Routes used (v1.12.0)

| Purpose | Route |
|---|---|
| create | `POST /v1/group` body `{"attrs": {"name": [..], "description": [..], "member": [..]}}` |
| read | `GET /v1/group/{id}`: 200 with `null` when absent; `{id}` may be name, SPN or UUID |
| list ours | `GET /v1/group/_search/fsg-`: names containing the text; we keep only names that start with it and hold a UUID |
| delete | `DELETE /v1/group/{id}`: 404 when absent (treated as done) |
| replace / append / remove | `PUT` / `POST` / `DELETE` on `/v1/group/{id}/_attr/{attr}`, body `["v", ...]`; `DELETE` without a body removes the whole attribute |

`apply` uses `PUT` for `description` and `member` (one modify per attribute, so one `memberof` recomputation). `POST` and
`DELETE` on attributes exist in the client and are tested, but `apply` does not need them.

## The key (id mapping)

Kanidm stores no foreign id on a group and has no custom attributes (research note, section 3). The group **name**
is derived from our id: `fsg-<hyphenated UUIDv7>` (40 characters; Kanidm allows `^[a-z][a-z0-9-_.]{0,63}$`).
Why the name: it is unique, addressable (`GET /v1/group/{name}` accepts it), never changed by us, and visible to admins.
Not the Kanidm UUID: Kanidm assigns it, so we would need our own table, which breaks when Kanidm is restored or recreated.
Not the `description`: free text that people edit, not unique. Not a name from the display name: changes on rename, collides.
The display name goes to `description`. The prefix lets `list` find our groups with one search and keeps hand-made groups out of reach.
Nested groups are sent as members by that name (Kanidm resolves names, SPNs and UUIDs the same way; an unknown value silently becomes a
non-existent reference, so the nested group must exist first: fs-groups applies children before parents).

## Structure and pattern map (ADR 0043)

| Module | Pattern | Why |
|---|---|---|
| `directory.rs` | Adapter (+ Strategy) | Kanidm behind `GroupDirectory` |
| `http/seam.rs` | Strategy | `HttpSeam` is the seam; tests plug in a stand-in |
| `http/reqwest_seam.rs` | Adapter | `reqwest` behind our seam; HTTPS only, private CA as root certificate |
| `http/token.rs` | Newtype | token that `Debug` hides |
| `client/*` | Facade | routes, bodies and status meaning in one small type; `Entry` is Kanidm's `v1::Entry` shape |
| `convert.rs` | Adapter | `DirectoryGroup` <-> Kanidm attributes, the only place of the attribute names |
| `naming.rs` | Adapter (pure) | id <-> name |
| `people.rs` | Strategy | the caller knows which person has which account |

## Decisions (the owner may change them)

* **People are mapped by the caller** (`PersonMap`); give the SPN, since Kanidm answers with SPNs. A person without an account fails
  the whole `apply` of that group before anything is written (error names them); `sync_to` reports it and goes on with other groups.
  The alternative (silently skipping them) would show as a difference at every run.
* **A member Kanidm has that nobody mapped** is read as `kanidm:<value>`, so it differs from the wanted state and the next `apply` removes it
  (fs-groups is the point of truth for groups it mirrors).
* **Roles are not mirrored.** `DirectoryGroup` has no role; a person is a plain member. The research note's idea of one Kanidm group per
  `(group, role)` would need the contract to say so.
* **Delete goes to Kanidm's recycle bin first.** Creating the same name again afterwards worked against 1.12.0 (live test).
* `capabilities()` lists `invite.create` and `right.grant` as unsupported, with the issue numbers, so a UI can say where each lives.
* The token is passed in by the caller, never read from the environment or a file by the library, never logged: a request holds no
  credentials, `ReqwestSeam` adds the header (marked sensitive by reqwest), `Debug` of token and transport hide it.
* Plain `http://` is refused. Certificate checks are never turned off; a private CA is added with `ca_pem`.

## Verified

* Offline: 26 tests against a stand-in written from reading the source (resolution of names / SPNs / UUIDs, SPNs in answers, 404 on delete,
  409 on a duplicate name, the end-to-end path `GroupRepo::sync_to` -> `KanidmDirectory`).
* Live, once, on 2026-11-06: `docker.io/kanidm/server:1.12.0` under podman, bound to 127.0.0.1, own throwaway CA, bootstrapped with
  `kanidmd recover-account idm_admin`, a service account in `idm_group_admins` with a read-write API token. The ignored test
  `tests/live.rs` (create, nested, read back, idempotent apply, hand edit healed, list, remove, re-create after delete) passed.
  Not covered live: `sync_to` end to end, people lists with several accounts, large groups, token expiry.

## Open points

* The name of the repo and crate.
* Handing control back when Kanidm gains a feature (ADR 0047): invitations (kanidm#4659), finer delegation (kanidm#4641), custom fields (kanidm#3493).
* A contract test in CI against a pinned container (research note 1.5, point 5) is not set up; the live test is run by hand.
* `kanidm_client` is not used; sign-in and credential flows are not built.

## Crates

`fs-groups-core` (git tag v0.1.0), `fs-objects` (v0.2.0), `fs-interfaces` (v0.3.0), `reqwest` 0.13 with rustls (MIT OR Apache-2.0),
`serde`, `serde_json`, `thiserror`, `async-trait`. No Kanidm crate, no `kanidm_proto`: Kanidm (MPL-2.0) is only called over the network.
