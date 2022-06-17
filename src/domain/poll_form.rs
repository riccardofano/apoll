use serde::Deserialize;
use validator::{Validate, ValidationError};

// TODO: implement prettier messages
#[derive(Debug, Validate, Deserialize)]
pub struct PollFormData {
    #[validate(
        length(min = 3, max = 32, message = "length is invalid."),
        custom = "validate_has_only_allowed_characters"
    )]
    pub username: String,
    #[validate(length(min = 3, max = 64, message = "length is invalid."))]
    pub prompt: String,
}

fn validate_has_only_allowed_characters(s: &str) -> Result<(), ValidationError> {
    let mut chars = s.chars();
    // First character must be a letter or a number
    if let Some(c) = chars.next() {
        if !c.is_ascii_alphanumeric() {
            return Err(ValidationError::new(
                "first character must be a letter or a number",
            ));
        }
    }
    // String must only contain [a-zA-Z0-9_]
    for c in chars {
        if !(c.is_ascii_alphanumeric() || c == '_') {
            return Err(ValidationError::new(
                "string contains disallowed characters",
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use claim::{assert_err, assert_ok};
    use validator::Validate;

    use super::PollFormData;

    fn new_form(username: &str, prompt: &str) -> PollFormData {
        PollFormData {
            username: username.to_string(),
            prompt: prompt.to_string(),
        }
    }

    #[test]
    fn valid_username_and_prompt_are_accepted() {
        let f = new_form("username", "question");
        assert_ok!(f.validate());
    }

    #[test]
    fn empty_form_is_rejected() {
        let f = new_form("", "");
        assert_err!(f.validate());
    }

    #[test]
    fn short_username_is_rejected() {
        let username = "u".repeat(2);
        let f = new_form(&username, "What kind of question?");
        assert_err!(f.validate());
    }

    #[test]
    fn username_with_whitespace_is_rejected() {
        let f = new_form("user name", "What kind of question?");
        assert_err!(f.validate());
    }

    #[test]
    fn usernames_with_special_characters_are_rejected() {
        let test_usernames = vec!["user?name", "!user", "pèrson", "<p>user</p>"];
        for username in test_usernames {
            let f = new_form(username, "What kind of question?");
            assert_err!(
                f.validate(),
                "{}",
                &format!("assertion failed on username: {username}")
            );
        }
    }

    #[test]
    fn prompt_too_short_is_rejected() {
        let prompt = "a".repeat(2);
        let f = new_form("username", &prompt);
        assert_err!(f.validate());
    }

    #[test]
    fn prompt_too_long_is_rejected() {
        let prompt = "a".repeat(65);
        let f = new_form("username", &prompt);
        assert_err!(f.validate());
    }
}
