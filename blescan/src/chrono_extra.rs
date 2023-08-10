use chrono::Duration;

pub trait Truncate {
    fn truncate_to_seconds(&self) -> Duration;
}

impl Truncate for Duration {
    fn truncate_to_seconds(&self) -> Duration {
        Duration::seconds(self.num_seconds())
    }
}

#[cfg(test)]
mod test {
    use chrono::Duration;

    use super::Truncate;

    #[test]
    fn truncate_to_seconds() {
        let d = Duration::milliseconds(1234);
        let expected = Duration::seconds(1);
        let actual = d.truncate_to_seconds();
        assert_eq!(actual, expected);
    }
}