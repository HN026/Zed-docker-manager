//! Pure types shared across modules, independent of the Zed extension API.

// The result of a Docker slash command, ready to be presented to the user.
#[derive(Debug)]
pub struct CommandOutput {
    pub text: String,
    pub label: String,
}

impl CommandOutput {
    // Creates a new CommandOutput with the given label and text.
    pub fn new(label: impl Into<String>, text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            label: label.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_command_output_with_label_and_text() {
        let output = CommandOutput::new("Docker PS", "container list");
        assert_eq!(output.label, "Docker PS");
        assert_eq!(output.text, "container list");
    }

    #[test]
    fn accepts_string_and_str() {
        let output = CommandOutput::new(String::from("Label"), String::from("Text"));
        assert_eq!(output.label, "Label");
        assert_eq!(output.text, "Text");
    }
}
