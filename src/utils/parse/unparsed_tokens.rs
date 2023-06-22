use super::{
    errors::ParseError,
    structs::{UnparsedToken, UnparsedTokenType},
};

pub(super) fn build_unparsed_tokens(input: &str) -> Result<Vec<UnparsedToken>, ParseError> {
    let input = input.to_lowercase().replace(' ', "");

    let mut token_list: Vec<UnparsedToken> = vec![];
    let mut prev_token_type = UnparsedTokenType::Separator; // will be overwritten

    for ch in input.chars() {
        let curr_token_type = UnparsedTokenType::try_from(ch)?;

        if curr_token_type != prev_token_type
            || token_list.is_empty()
            || curr_token_type == UnparsedTokenType::Separator
        // Always new token if its a separator
        {
            // new token
            token_list.push(UnparsedToken {
                variant: curr_token_type,
                string: ch.to_string(),
            });

            prev_token_type = curr_token_type
        } else {
            // add to last token
            token_list
                .last_mut()
                .expect("List is not empty")
                .string
                .push(ch);
        }
    }

    Ok(token_list)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn separate_time_unit() {
        assert_eq!(
            build_unparsed_tokens("1d"),
            Ok(vec![
                UnparsedToken {
                    variant: UnparsedTokenType::Number,
                    string: "1".to_string()
                },
                UnparsedToken {
                    variant: UnparsedTokenType::Text,
                    string: "d".to_string()
                },
            ])
        );
        assert_eq!(
            build_unparsed_tokens("1.3 h"),
            Ok(vec![
                UnparsedToken {
                    variant: UnparsedTokenType::Number,
                    string: "1.3".to_string()
                },
                UnparsedToken {
                    variant: UnparsedTokenType::Text,
                    string: "h".to_string()
                },
            ])
        );
        assert_eq!(
            build_unparsed_tokens("3M "),
            Ok(vec![
                UnparsedToken {
                    variant: UnparsedTokenType::Number,
                    string: "3".to_string()
                },
                UnparsedToken {
                    variant: UnparsedTokenType::Text,
                    string: "m".to_string()
                },
            ])
        );
        assert_eq!(
            build_unparsed_tokens("94 ms"),
            Ok(vec![
                UnparsedToken {
                    variant: UnparsedTokenType::Number,
                    string: "94".to_string()
                },
                UnparsedToken {
                    variant: UnparsedTokenType::Text,
                    string: "ms".to_string()
                },
            ])
        );
    }

    #[test]
    fn separate_multiple_time_unit() {
        assert_eq!(
            build_unparsed_tokens("1d3h"),
            Ok(vec![
                UnparsedToken {
                    variant: UnparsedTokenType::Number,
                    string: "1".to_string()
                },
                UnparsedToken {
                    variant: UnparsedTokenType::Text,
                    string: "d".to_string()
                },
                UnparsedToken {
                    variant: UnparsedTokenType::Number,
                    string: "3".to_string()
                },
                UnparsedToken {
                    variant: UnparsedTokenType::Text,
                    string: "h".to_string()
                },
            ])
        );
        assert_eq!(
            build_unparsed_tokens("5h 92m 1ms"),
            Ok(vec![
                UnparsedToken {
                    variant: UnparsedTokenType::Number,
                    string: "5".to_string()
                },
                UnparsedToken {
                    variant: UnparsedTokenType::Text,
                    string: "h".to_string()
                },
                UnparsedToken {
                    variant: UnparsedTokenType::Number,
                    string: "92".to_string()
                },
                UnparsedToken {
                    variant: UnparsedTokenType::Text,
                    string: "m".to_string()
                },
                UnparsedToken {
                    variant: UnparsedTokenType::Number,
                    string: "1".to_string()
                },
                UnparsedToken {
                    variant: UnparsedTokenType::Text,
                    string: "ms".to_string()
                },
            ])
        );
    }

    #[test]
    fn separate_separators() {
        assert_eq!(
            build_unparsed_tokens("3:4:7"),
            Ok(vec![
                UnparsedToken {
                    variant: UnparsedTokenType::Number,
                    string: "3".to_string()
                },
                UnparsedToken {
                    variant: UnparsedTokenType::Separator,
                    string: ":".to_string()
                },
                UnparsedToken {
                    variant: UnparsedTokenType::Number,
                    string: "4".to_string()
                },
                UnparsedToken {
                    variant: UnparsedTokenType::Separator,
                    string: ":".to_string()
                },
                UnparsedToken {
                    variant: UnparsedTokenType::Number,
                    string: "7".to_string()
                },
            ])
        );
        assert_eq!(
            build_unparsed_tokens("1::2"),
            Ok(vec![
                UnparsedToken {
                    variant: UnparsedTokenType::Number,
                    string: "1".to_string()
                },
                UnparsedToken {
                    variant: UnparsedTokenType::Separator,
                    string: ":".to_string()
                },
                UnparsedToken {
                    variant: UnparsedTokenType::Separator,
                    string: ":".to_string()
                },
                UnparsedToken {
                    variant: UnparsedTokenType::Number,
                    string: "2".to_string()
                },
            ])
        );
    }
}
