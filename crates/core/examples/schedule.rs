use std::io::{self, BufRead};

fn main() -> io::Result<()> {
    for line in io::stdin().lock().lines() {
        println!("{}", lexique_core::scheduling::schedule_json(&line?));
    }
    Ok(())
}
