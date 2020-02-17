
fn there_is_no_null(numerator: u8, denominator: u8) -> Option<u8> {
    if denominator == 0 {
        None
    } else {
        Some(numerator / denominator)
    }
}

fn there_is_no_exception(numerator: u8, denominator: u8) -> Result<u8, &'static str> {
    if denominator == 0 {
        Err("You can't divide by zero")
    } else {
        Ok(numerator / denominator)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_there_is_no_null() {
        assert_eq!(there_is_no_null(30, 0), None);
        assert_eq!(there_is_no_null(30, 2), Some(15));
    }

    #[test]
    fn test_there_is_no_exception() {
        assert_eq!(there_is_no_exception(30, 0), Err("You can't divide by zero"));
        assert_eq!(there_is_no_exception(30, 2), Ok(15));
    }
}