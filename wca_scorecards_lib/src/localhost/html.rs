pub fn event_list_to_html(rounds: Vec<(String, usize)>) -> String {
    rounds.iter().map(|(eventid, round)|format!("<a href=round/?eventid={e}&round={r}>{e}, {r}</a><br>", r = round, e = eventid)).collect()
}