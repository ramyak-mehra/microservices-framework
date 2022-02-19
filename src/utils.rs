use std::borrow::Cow;

use uuid::Uuid;

pub(crate) fn generate_uuid() -> String {
    Uuid::new_v4().to_string()
}

fn remove_from_list<T: PartialEq + Eq>(list: &mut Vec<T>, value: &T) {
    list.retain(|t| {
        if t == value {
            return false;
        }
        true
    });
}
pub(crate) fn hostname() -> Cow<'static, str> {
    hostname::get()
        .map(|s| Cow::Owned(s.to_string_lossy().to_string().to_lowercase()))
        .unwrap_or_else(|_| Cow::Borrowed("unknown_host_name"))
}

pub(crate) fn ip_list() -> Vec<String> {
    get_if_addrs::get_if_addrs()
        .unwrap_or_default()
        .iter()
        .map(|interface| interface.addr.ip())
        .filter(|ip| ip.is_ipv4() && !ip.is_loopback())
        .map(|ip| ip.to_string())
        .collect()
}
