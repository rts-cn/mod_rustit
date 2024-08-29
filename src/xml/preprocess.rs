
use switch_sys::*;
pub fn expand_vars(s: &str) -> String {
    let mut expand = String::from(s);
    for (pos, _) in s.match_indices("$${") {
        let end = (s[pos..]).to_string().find("}");
        if let Some(end) = end {
            let vname = s[pos + 3..end + pos].to_string();
            let val = switch_sys::get_variable(&vname);
            expand = expand.replace(&format!("$${{{}}}", vname), &val);
        }
    }
    expand
}

fn set(data: &str) {
    let r = data.split_once("=");
    match r {
        Some((name, val)) => {
            if !name.is_empty() && !val.is_empty() {
                switch_sys::set_variable(name, val);
            }
        }
        None => {}
    }
}

fn env_set(data: &str) {
    let r = data.split_once("=");
    match r {
        Some((name, val)) => {
            if !name.is_empty() && !val.is_empty() {
                info!("name { }, val {}", name, val);
            }
        }
        None => {}
    }
}

fn stun_set(data: &str) {
    let r = data.split_once("=");
    match r {
        Some((name, val)) => {
            if !name.is_empty() && !val.is_empty() {
                info!("name { }, val {}", name, val);
            }
        }
        None => {}
    }
}

pub fn process(re: regex::Regex, text: &str) {
    if !text.contains("X-PRE-PROCESS") {
        return;
    }
    for line in text.lines() {
        for cap in re.captures_iter(&line) {
            let (full, [cmd, data]) = cap.extract();
            if cmd.eq_ignore_ascii_case("set") {
                set(&data)
            } else if cmd.eq_ignore_ascii_case("stun-set") {
                stun_set(&data)
            } else if cmd.eq_ignore_ascii_case("env-set") {
                env_set(&data)
            } else {
                warn!("Unsupported pre process command {}", full);
            }
        }
    }
}
