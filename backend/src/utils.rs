pub trait StrExt: AsRef<str> {
    fn split2(&self, p: char) -> Option<(&str, &str)> {
        let s = self.as_ref();
        let i = s.find(p)?;
        return Some((&s[..i], &s[i + 1..]));
    }
}

impl<S: AsRef<str>> StrExt for S {}
