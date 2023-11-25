use crate::sketchbook::layout::LayoutId;
use crate::sketchbook::Identifier;
use std::fmt::{Display, Error, Formatter};
use std::str::FromStr;

impl LayoutId {
    /// Construct new instances of `LayoutId`s from string.
    ///
    /// Returns `Err` if the string is not a valid identifier (it must be a C-like identifier).
    ///
    /// This does not ensure that the generated ID is unique and usable for given context.
    /// Parent classes (like `RegulationsState`) allow to generate unique `VarIds` safely.
    pub(crate) fn new(identifier: &str) -> Result<LayoutId, String> {
        Ok(LayoutId {
            id: Identifier::new(identifier)?,
        })
    }

    /// Access the identifier as a string slice.
    pub fn as_str(&self) -> &str {
        self.id.as_str()
    }
}

impl Display for LayoutId {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "{}", self.id)
    }
}

impl FromStr for LayoutId {
    type Err = String;

    fn from_str(s: &str) -> Result<LayoutId, <LayoutId as FromStr>::Err> {
        LayoutId::new(s)
    }
}

#[cfg(test)]
mod tests {
    use crate::sketchbook::layout::LayoutId;
    use std::str::FromStr;

    #[test]
    fn test_valid_layout_id() {
        let layout_string = "layout";
        let layout_id = LayoutId::new(layout_string).unwrap();
        let same_layout_id = LayoutId::from_str(layout_string).unwrap();
        assert_eq!(layout_id, same_layout_id);

        assert_eq!(layout_id.as_str(), layout_string);
        assert_eq!(same_layout_id.to_string(), layout_string.to_string());
    }

    #[test]
    fn test_invalid_layout_id() {
        let layout_string = "invalid %%% id";
        let layout_id = LayoutId::new(layout_string);
        assert!(layout_id.is_err());

        let layout_id = LayoutId::from_str(layout_string);
        assert!(layout_id.is_err());
    }
}
