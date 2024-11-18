use crate::app::DynError;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};

/// [AeonError] is an implementation of [Error] which is
/// intended as a general "runtime error" in the AEON application.
pub struct AeonError {
    description: String,
    source: Option<DynError>,
}

impl AeonError {
    /// Create a new instance of [AeonError] with the provided `description` and
    /// an optional `source` [DynError].
    ///
    /// Refer to [Error] regarding recommended error description format.
    ///
    /// ```rust
    /// # use biodivine_sketchbook::app::AeonError;
    /// # use std::error::Error;
    /// let error = AeonError::new("something failed", None);
    /// let other_error = AeonError::new("something else failed", Some(Box::new(error)));
    /// assert_eq!("something else failed", format!("{}", other_error));
    /// assert_eq!("something failed", format!("{}", other_error.source().unwrap()));
    /// ```
    pub fn new(description: impl Into<String>, source: Option<DynError>) -> AeonError {
        AeonError {
            description: description.into(),
            source,
        }
    }

    /// The same as [Self::new], but returns [DynError] instead.
    pub fn dyn_new(description: impl Into<String>) -> DynError {
        Box::new(Self::new(description, None))
    }

    /// Create a new instance of [AeonError], convert it to [DynError] and return it as
    /// the specified [Result] type.
    ///
    /// This function is useful when you want to return an error from a function which
    /// returns some `Result<R, DynError>`, because you don't need to convert the error
    /// into the expected result type.
    ///
    /// See also [AeonError::throw_with_source].
    ///
    /// ```rust
    /// # use biodivine_sketchbook::app::{AeonError, DynError};
    /// fn division(numerator: i32, denominator: i32) -> Result<i32, DynError> {
    ///     if denominator == 0 {
    ///         AeonError::throw("division by zero")
    ///     } else {
    ///         Ok(numerator / denominator)
    ///     }
    /// }
    ///
    /// assert_eq!(5, division(10, 2).unwrap());
    /// assert_eq!("division by zero", format!("{}", division(10, 0).unwrap_err()));
    /// ```
    pub fn throw<R>(description: impl Into<String>) -> Result<R, DynError> {
        Err(Box::new(AeonError::new(description, None)))
    }

    /// The same as [AeonError::throw], but also includes a generic error `source`.
    ///
    /// Note that compared to [AeonError::new], `source` can be any `Into<DynError>` type,
    /// which means you can avoid conversions when they can be performed automatically
    /// (see the example below).
    ///
    /// ```rust
    /// # use biodivine_sketchbook::app::{AeonError, DynError};
    /// fn read_number(num: &str) -> Result<i32, DynError> {
    ///     match num.parse::<i32>() {
    ///         Ok(num) => Ok(num),
    ///         Err(e) => AeonError::throw_with_source("invalid number", e),
    ///     }
    /// }
    ///
    /// assert_eq!(5, read_number("5").unwrap());
    /// assert_eq!("invalid number", format!("{}", read_number("abc").unwrap_err()));
    /// ```
    pub fn throw_with_source<R>(
        description: impl Into<String>,
        source: impl Into<DynError>,
    ) -> Result<R, DynError> {
        Err(Box::new(AeonError::new(description, Some(source.into()))))
    }
}

impl Debug for AeonError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if let Some(source) = self.source.as_ref() {
            write!(
                f,
                "AeonError[description=\"{}\",source=\"{}\"]",
                self.description, source
            )
        } else {
            write!(f, "AeonError[description=\"{}\"]", self.description)
        }
    }
}

impl Display for AeonError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.description)
    }
}

impl Error for AeonError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.source.as_ref().map(|it| it.as_ref())
    }
}
