//! The Kanidm stand-in itself: stored groups, the request log and the route handling.

use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use fs_identity_kanidm::{HttpSeam, KanidmDirectory, Method, Request, Response};
use serde_json::{Value, json};

use super::{REALM, people};

#[derive(Clone, Debug)]
pub struct Stored {
    pub name: String,
    pub uuid: String,
    pub description: Option<String>,
    pub members: Vec<String>, // SPNs
}

#[derive(Default)]
pub struct FakeKanidm {
    pub groups: Mutex<Vec<Stored>>,
    pub log: Mutex<Vec<Request>>,
    pub accounts: Vec<String>,
}

impl FakeKanidm {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            accounts: ["ada", "bo", "cy"].map(|n| format!("{n}@{REALM}")).to_vec(),
            ..Self::default()
        })
    }
    pub fn directory(self: &Arc<Self>) -> KanidmDirectory {
        KanidmDirectory::new(self.clone(), Arc::new(people()))
    }
    pub fn calls(&self) -> Vec<(String, String)> {
        self.log
            .lock()
            .unwrap()
            .iter()
            .map(|r| (r.method.as_str().to_string(), r.path.clone()))
            .collect()
    }
    pub fn writes(&self) -> usize {
        self.log
            .lock()
            .unwrap()
            .iter()
            .filter(|r| r.method != Method::Get)
            .count()
    }
    pub fn clear_log(&self) {
        self.log.lock().unwrap().clear();
    }
    pub fn stored(&self, name: &str) -> Option<Stored> {
        self.groups
            .lock()
            .unwrap()
            .iter()
            .find(|g| g.name == name)
            .cloned()
    }
    fn resolve(&self, v: &str) -> Option<String> {
        let groups = self.groups.lock().unwrap();
        let name = v.split('@').next().unwrap_or(v);
        if let Some(g) = groups.iter().find(|g| g.name == name || g.uuid == v) {
            return Some(format!("{}@{REALM}", g.name));
        }
        self.accounts
            .iter()
            .find(|a| a.as_str() == v || a.split('@').next() == Some(v))
            .cloned()
    }
    fn entry(g: &Stored) -> Value {
        let mut attrs = json!({"class": ["group", "object"], "name": [g.name], "spn": [format!("{}@{REALM}", g.name)], "uuid": [g.uuid], "member": g.members});
        if let Some(d) = &g.description {
            attrs["description"] = json!([d]);
        }
        json!({ "attrs": attrs })
    }
}

fn reply(status: u16, body: Value) -> Response {
    Response { status, body }
}

fn strings(v: &Value) -> Vec<String> {
    v.as_array()
        .map(|a| {
            a.iter()
                .filter_map(|x| x.as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default()
}

#[async_trait]
impl HttpSeam for FakeKanidm {
    async fn send(&self, req: Request) -> fs_identity_kanidm::Result<Response> {
        self.log.lock().unwrap().push(req.clone());
        let parts: Vec<&str> = req.path.trim_start_matches('/').split('/').collect();
        let body = req.body.clone().unwrap_or(Value::Null);
        let find =
            |id: &str| {
                self.groups.lock().unwrap().iter().position(|g| {
                    g.name == id || g.uuid == id || format!("{}@{REALM}", g.name) == id
                })
            };
        Ok(match (req.method, parts.as_slice()) {
            (Method::Post, ["v1", "group"]) => {
                let a = &body["attrs"];
                let name = strings(&a["name"]).remove(0);
                if find(&name).is_some() {
                    return Ok(reply(409, json!({"error": "Attribute uniqueness error"})));
                }
                let Some(members) = strings(&a["member"])
                    .iter()
                    .map(|m| self.resolve(m))
                    .collect::<Option<Vec<_>>>()
                else {
                    return Ok(reply(500, json!("RefintNotUpheld")));
                };
                let mut groups = self.groups.lock().unwrap();
                let uuid = format!("00000000-0000-4000-8000-{:012}", groups.len() + 1);
                groups.push(Stored {
                    name,
                    uuid,
                    description: strings(&a["description"]).first().cloned(),
                    members,
                });
                reply(200, Value::Null)
            }
            (Method::Get, ["v1", "group", "_search", frag]) => {
                let found: Vec<_> = self
                    .groups
                    .lock()
                    .unwrap()
                    .iter()
                    .filter(|g| g.name.contains(frag))
                    .map(Self::entry)
                    .collect();
                reply(200, json!(found))
            }
            (Method::Get, ["v1", "group", id]) => reply(
                200,
                find(id).map_or(Value::Null, |i| {
                    Self::entry(&self.groups.lock().unwrap()[i])
                }),
            ),
            (Method::Delete, ["v1", "group", id]) => match find(id) {
                Some(i) => {
                    let gone = self.groups.lock().unwrap().remove(i);
                    let spn = format!("{}@{REALM}", gone.name);
                    self.groups
                        .lock()
                        .unwrap()
                        .iter_mut()
                        .for_each(|g| g.members.retain(|m| *m != spn));
                    reply(200, Value::Null)
                }
                None => reply(404, json!("NoMatchingEntries")),
            },
            (m, ["v1", "group", id, "_attr", attr]) => {
                let Some(i) = find(id) else {
                    return Ok(reply(404, json!("NoMatchingEntries")));
                };
                let values = strings(&body);
                let resolved: Option<Vec<String>> = if *attr == "member" {
                    values.iter().map(|v| self.resolve(v)).collect()
                } else {
                    Some(values.clone())
                };
                let Some(resolved) = resolved else {
                    return Ok(reply(500, json!("RefintNotUpheld")));
                };
                let mut groups = self.groups.lock().unwrap();
                let g = &mut groups[i];
                match (m, *attr) {
                    (Method::Put, "member") => g.members = resolved,
                    (Method::Post, "member") => g.members.extend(resolved),
                    (Method::Delete, "member") if body.is_null() => g.members.clear(),
                    (Method::Delete, "member") => g.members.retain(|x| !resolved.contains(x)),
                    (Method::Put, "description") => g.description = resolved.first().cloned(),
                    _ => return Ok(reply(400, json!("InvalidAttribute"))),
                }
                reply(200, Value::Null)
            }
            _ => reply(404, json!("no such route")),
        })
    }
}
