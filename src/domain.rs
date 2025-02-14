use unicode_segmentation::UnicodeSegmentation;

pub struct NewSubscriber {
    pub email: String,
    pub name: SubscriberName,
}
pub struct SubscriberName(String);

impl SubscriberName {
    // the caller gets a read-only reference to the inner value of the SubscriberName
    pub fn inner_ref(&self) -> &str {
        &self.0
    }

    pub fn parse(s: String) -> Result<Self, String> {
        if is_valid_name(&s) {
            Ok(Self(s))
        } else {
            Err(format!("{} is not a valid subscriber name.", s))
        }
    }
}

fn is_valid_name(name: &str) -> bool {
    let is_empty = name.trim().is_empty();

    let is_too_long = name.graphemes(true).count() > 256;
    let contains_forbidden_characters = name
        .chars()
        .any(|c| !(c.is_alphanumeric() || c == '-' || c == ' '));

    !is_empty && !is_too_long && !contains_forbidden_characters
}
