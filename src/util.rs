use std::env;

pub fn require_non_emtpy(var: &str) -> String {
    env::var(var).expect(&format!("Missing var {}", var))
}
