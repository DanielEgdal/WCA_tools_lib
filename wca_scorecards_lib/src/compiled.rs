pub const JS: &'static str = include_str!("../dependencies/js.js");
const ROUND: &'static str = include_str!("../dependencies/index.html");
pub const WASM: &[u8] = include_bytes!("../dependencies/group_menu_bg.wasm");
pub const WASM_JS: &'static str = include_str!("../dependencies/group_menu.js");

pub fn js_replace(competitors: &str, n: usize, eventid: &str, round: usize, capacity: u32, auth_code: &str, competition: &str) -> String {
    ROUND.replace("DATA", competitors)
        .replace("NUMBER", &n.to_string())
        .replace("EVENT", &format!("eventid={}&round={}&auth_code={}&competition={}", eventid, round, auth_code, competition))
        .replace("CAPACITY", &capacity.to_string())
}
