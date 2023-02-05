pub fn event_list_to_html(rounds: Vec<(String, usize)>, auth_code: &str, competition: &str) -> String {
    rounds.iter().map(|(eventid, round)|format!("<a href=round/?eventid={e}&round={r}&auth_code={a}&competition={c}>{e}, {r}</a><br>", 
        r = round, 
        e = eventid, 
        a = auth_code, 
        c = competition)).collect()
}
