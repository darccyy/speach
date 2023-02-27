use std::io::{self, Write};

use stilo::stylize;
use tts::Tts;

use speach::run_text;

fn main() {
    println!("{}", stylize!("Speach - Enter text to speak", Cyan italic));

    // Create tts speaker
    let mut speaker = Tts::default().expect("Could not create TTS");

    loop {
        // Speak text input
        run_text(&mut speaker, &input("> \x1b[1m"));
    }
}

/// Read text line user cli input
fn input(prompt: &str) -> String {
    let mut s = String::new();
    print!("{}", prompt);

    let _ = io::stdout().flush();
    io::stdin()
        .read_line(&mut s)
        .expect("Did not enter a correct string");

    if let Some('\n') = s.chars().next_back() {
        s.pop();
    }
    if let Some('\r') = s.chars().next_back() {
        s.pop();
    }

    s
}
