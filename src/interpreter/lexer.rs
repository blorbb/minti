use super::{Error, Result};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub(super) enum GroupKind {
    Number,
    Text,
    Separator,
}

impl TryFrom<char> for GroupKind {
    type Error = Error;

    fn try_from(value: char) -> Result<Self> {
        if value.is_ascii_alphabetic() {
            Ok(Self::Text)
        } else if value.is_ascii_digit() || value == '.' {
            Ok(Self::Number)
        } else if value == ':' {
            Ok(Self::Separator)
        } else {
            Err(Error::InvalidCharacter(value))
        }
    }
}

/// A string that has one 'type' of characters.
///
/// The three variants are:
/// - `Text` if all characters are letters.
/// - `Number` if all characters are digits or ".".
/// - `Separator` if the string is ":".
#[derive(Debug, PartialEq, Eq)]
pub(super) struct Group {
    pub variant: GroupKind,
    pub string: String,
}

/// Splits a string into separate strings with types.
///
/// See [`Group`] for the variants and conditions.
///
/// # Errors
/// Errors if any character could not be parsed into a token.
/// Characters `[A-Za-z0-9.:]` are the only accepted characters.
pub(super) fn lex(input: &str) -> Result<Vec<Group>> {
    // only ascii is parsed anyways
    let input = input.to_ascii_lowercase().replace(' ', "");

    let mut token_list: Vec<Group> = Vec::new();
    let mut prev_token_type = GroupKind::Separator; // will be overwritten

    for ch in input.chars() {
        log::trace!("parsing character {ch:?}");
        let curr_token_type = GroupKind::try_from(ch)?;

        // Always new token if its a separator
        let is_new_token = curr_token_type != prev_token_type
            || token_list.is_empty()
            || curr_token_type == GroupKind::Separator;

        if is_new_token {
            log::trace!("character is a new token");
            // create new token: add to the vec
            token_list.push(Group {
                variant: curr_token_type,
                string: ch.to_string(),
            });

            prev_token_type = curr_token_type;
        } else {
            log::trace!("character is same type as previous");
            // add to the last token in the vec
            token_list
                .last_mut()
                .expect("new token should always be appended first")
                .string
                .push(ch);
        }
    }

    log::trace!("successfully built unparsed tokens");
    Ok(token_list)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn separate_time_unit() {
        assert_eq!(
            lex("1d"),
            Ok(vec![
                Group {
                    variant: GroupKind::Number,
                    string: "1".to_string()
                },
                Group {
                    variant: GroupKind::Text,
                    string: "d".to_string()
                },
            ])
        );
        assert_eq!(
            lex("1.3 h"),
            Ok(vec![
                Group {
                    variant: GroupKind::Number,
                    string: "1.3".to_string()
                },
                Group {
                    variant: GroupKind::Text,
                    string: "h".to_string()
                },
            ])
        );
        assert_eq!(
            lex("3M "),
            Ok(vec![
                Group {
                    variant: GroupKind::Number,
                    string: "3".to_string()
                },
                Group {
                    variant: GroupKind::Text,
                    string: "m".to_string()
                },
            ])
        );
        assert_eq!(
            lex("94 ms"),
            Ok(vec![
                Group {
                    variant: GroupKind::Number,
                    string: "94".to_string()
                },
                Group {
                    variant: GroupKind::Text,
                    string: "ms".to_string()
                },
            ])
        );
    }

    #[test]
    fn separate_multiple_time_unit() {
        assert_eq!(
            lex("1d3h"),
            Ok(vec![
                Group {
                    variant: GroupKind::Number,
                    string: "1".to_string()
                },
                Group {
                    variant: GroupKind::Text,
                    string: "d".to_string()
                },
                Group {
                    variant: GroupKind::Number,
                    string: "3".to_string()
                },
                Group {
                    variant: GroupKind::Text,
                    string: "h".to_string()
                },
            ])
        );
        assert_eq!(
            lex("5h 92m 1ms"),
            Ok(vec![
                Group {
                    variant: GroupKind::Number,
                    string: "5".to_string()
                },
                Group {
                    variant: GroupKind::Text,
                    string: "h".to_string()
                },
                Group {
                    variant: GroupKind::Number,
                    string: "92".to_string()
                },
                Group {
                    variant: GroupKind::Text,
                    string: "m".to_string()
                },
                Group {
                    variant: GroupKind::Number,
                    string: "1".to_string()
                },
                Group {
                    variant: GroupKind::Text,
                    string: "ms".to_string()
                },
            ])
        );
    }

    #[test]
    fn separate_separators() {
        assert_eq!(
            lex("3:4:7"),
            Ok(vec![
                Group {
                    variant: GroupKind::Number,
                    string: "3".to_string()
                },
                Group {
                    variant: GroupKind::Separator,
                    string: ":".to_string()
                },
                Group {
                    variant: GroupKind::Number,
                    string: "4".to_string()
                },
                Group {
                    variant: GroupKind::Separator,
                    string: ":".to_string()
                },
                Group {
                    variant: GroupKind::Number,
                    string: "7".to_string()
                },
            ])
        );
        assert_eq!(
            lex("1::2"),
            Ok(vec![
                Group {
                    variant: GroupKind::Number,
                    string: "1".to_string()
                },
                Group {
                    variant: GroupKind::Separator,
                    string: ":".to_string()
                },
                Group {
                    variant: GroupKind::Separator,
                    string: ":".to_string()
                },
                Group {
                    variant: GroupKind::Number,
                    string: "2".to_string()
                },
            ])
        );
    }
}
