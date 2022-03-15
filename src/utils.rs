use lazy_static::lazy_static;
use process_uptime::ProcessUptime;
use regex::Regex;
use std::time::Duration;
use std::{borrow::Cow, collections::HashMap, future::Future, ops::Index};
use tokio::time;
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
pub(crate) fn service_from_action(action_name: &str) -> String {
    let service: Vec<&str> = action_name.split('.').collect();
    service.get(0).unwrap().to_string()
}
pub(crate) fn match_str(text: &str, pattern: &str) -> bool {
    //Simple patterns
    if pattern.find("?").is_none() {
        // Exact match (eg. "prefix.event")
        let first_start_pos = pattern.find("*");
        if first_start_pos.is_none() {
            return pattern == text;
        }
        // Eg. "prefix**"
        let len = pattern.len();
        if len > 2 && pattern.ends_with("**") && first_start_pos.unwrap() > len - 3 {
            let new_pattern: String = pattern.chars().take(len - 2).collect();
            return text.starts_with(&new_pattern);
        }
        // Eg. "prefix*"
        if len > 1 && pattern.ends_with("*") && first_start_pos.unwrap() > len - 2 {
            let new_pattern: String = pattern.chars().take(len - 1).collect();
            if text.starts_with(&new_pattern) {
                return text[len..].find(".").is_none();
            }
            return false;
        }
        // Accept simple text, without point character (*)
        if len == 1 && first_start_pos.unwrap() == 0 {
            return text.find(".").is_none();
        }
        // Accept all inputs (**)
        let rf = pattern.rfind("*");
        if len == 2 && first_start_pos.unwrap() == 0 && rf.is_some() && rf.unwrap() == 1 {
            return true;
        }
    }

    // Regex (eg. "prefix.ab?cd.*.foo")
    todo!("Regex pattern matching left");
    let original_pattern = pattern.to_owned();
    let mut pattern = pattern;
    // if pattern.starts_with("$"){
    //     pattern =  &("//".to_owned() + &original_pattern);
    // }
    // pattern = &pattern.replace(r"/\?/g", ".");
}
pub(crate) fn process_uptime() -> Duration {
    ProcessUptime::new().unwrap().uptime
}
pub(crate) fn set_interval<F, Fut>(mut f: F, dur: Duration)
where
    F: Send + 'static + Fn() -> Fut,
    Fut: Future<Output = ()> + Send + 'static,
{
    // Create stream of intervals.
    let mut interval = time::interval(dur);

    tokio::spawn(async move {
        // Skip the first tick at 0ms.
        interval.tick().await;
        loop {
            // Wait until next tick.
            interval.tick().await;
            // Spawn a task for this tick.
            tokio::spawn(f());
        }
    });
}
pub(crate) async fn get_cpu_usage() -> u32 {
    todo!("get cpu usage")
}
