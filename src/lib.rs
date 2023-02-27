use std::{process, str::Chars};

use stilo::stylize;
use tts::{Tts, UtteranceId};

use Method::*;

pub fn run_text(speaker: &mut Tts, text: &str) {
    // Return if text is blank
    if text.trim().is_empty() {
        return;
    }

    // Run command if starts with a period
    if text.starts_with('.') {
        if let Err(error) = run_command(speaker, text) {
            eprintln!("{}", error);
        }
        return;
    }

    // Speak text
    speak_blocking(speaker, &text.trim(), false).expect("Failed to speak");
}

fn run_command(speaker: &mut Tts, text: &str) -> Result<(), &'static str> {
    // Remove first character
    let mut chars = text.chars();
    chars.next();

    // Get command character
    let Some(command) = chars.next() else {
        return Err("Missing command");
    };

    /// Create a set command
    macro_rules! create_command {
        ( $name: literal, $id: ident, $get: ident, $set: ident, $normal: ident, $min: ident, $max: ident ) => {{
            // Get method
            if let Some(method) = Method::from(&mut chars) {
                // Apply method
                let property = method.apply(
                    speaker.$get().unwrap(),
                    1.0,
                    speaker.$normal(),
                    speaker.$min(),
                    speaker.$max(),
                );

                // Set value
                speaker.$set(property).unwrap();
            }

            // Update the properties (such a hack)
            speaker.speak("THE FOG IS COMING", false).unwrap();
            speaker.stop().unwrap();

            /// Display a decimal float, multiplied by 10 and rounded
            fn display(value: f32) -> String {
                (value * 10.0).round().to_string()
            }

            // Print value
            println!(
                "{}",
                stylize!(
                    &format!(
                        "{}: {} ({}-{})",
                        $name,
                        stylize!(&display(speaker.$get().unwrap()), Yellow),
                        display(speaker.$min()),
                        display(speaker.$max()),
                    ),
                    -italic
                )
            );
        }};
    }

    match command {
        // Quit
        'x' => process::exit(0),

        // Volume
        'v' => create_command!(
            "Volume",
            volume,
            get_volume,
            set_volume,
            normal_volume,
            min_volume,
            max_volume
        ),

        // Pitch
        'p' => create_command!(
            "Pitch",
            pitch,
            get_pitch,
            set_pitch,
            normal_pitch,
            min_pitch,
            max_pitch
        ),

        // Rate
        'r' => create_command!(
            "Rate",
            rate,
            get_rate,
            set_rate,
            normal_rate,
            min_rate,
            max_rate
        ),

        // Unknown command
        _ => {
            return Err("Unknown command");
        }
    }

    Ok(())
}

enum Method {
    Exactly(Option<f32>),
    Increase(Option<f32>),
    Decrease(Option<f32>),
    Min,
    Max,
}

impl Method {
    pub fn from(chars: &mut Chars) -> Option<Method> {
        let method = chars.next()?;
        let value = chars.as_str().parse().ok();

        Some(match method {
            '=' => Exactly(value),
            '+' => Increase(value),
            '-' => Decrease(value),
            '<' => Min,
            '>' => Max,

            _ => return None,
        })
    }

    pub fn apply(&self, mut value: f32, step: f32, normal: f32, min: f32, max: f32) -> f32 {
        match self {
            Exactly(new) => value = new.unwrap_or(normal) / 10.0,
            Increase(new) => value += new.unwrap_or(step) / 10.0,
            Decrease(new) => value -= new.unwrap_or(step) / 10.0,
            Min => value = min,
            Max => value = max,
        }

        value.max(min).min(max)
    }
}

/// Use `speak` method, blocking thread with `tokio`, return utterance id
fn speak_blocking(
    speaker: &mut Tts,
    text: &str,
    interrupt: bool,
) -> Result<UtteranceId, tts::Error> {
    // Start speaking
    speaker.speak(text, interrupt)?;

    // Create sender and receiver
    let (tx, rx) = std::sync::mpsc::channel();

    // Create callback for speech end
    speaker.on_utterance_end(Some(Box::new(move |id| {
        // Send id
        tx.send(id).unwrap();
    })))?;

    // Return when received id
    Ok(rx.recv().unwrap())
}
