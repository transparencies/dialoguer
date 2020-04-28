use std::{
    fmt::{Debug, Display},
    io,
    str::FromStr,
};

use crate::{
    theme::{SimpleTheme, TermThemeRenderer, Theme},
    validate::Validator,
};

use console::Term;

/// Renders a simple confirmation prompt.
///
/// ## Example usage
///
/// ```rust,no_run
/// # fn test() -> Result<(), Box<std::error::Error>> {
/// use dialoguer::Confirmation;
///
/// if Confirmation::new().with_text("Do you want to continue?").interact()? {
///     println!("Looks like you want to continue");
/// } else {
///     println!("nevermind then :(");
/// }
/// # Ok(()) } fn main() { test().unwrap(); }
/// ```
pub struct Confirmation<'a> {
    prompt: String,
    default: bool,
    show_default: bool,
    theme: &'a dyn Theme,
}

/// Renders a simple input prompt.
///
/// ## Example usage
///
/// ```rust,no_run
/// # fn test() -> Result<(), Box<std::error::Error>> {
/// use dialoguer::Input;
///
/// let name = Input::<String>::new().with_prompt("Your name").interact()?;
/// println!("Name: {}", name);
/// # Ok(()) } fn main() { test().unwrap(); }
/// ```
pub struct Input<'a, T> {
    prompt: String,
    default: Option<T>,
    show_default: bool,
    initial_text: Option<String>,
    theme: &'a dyn Theme,
    permit_empty: bool,
    validator: Option<Box<dyn Fn(&str) -> Option<String>>>,
}

/// Renders a password input prompt.
///
/// ## Example usage
///
/// ```rust,no_run
/// # fn test() -> Result<(), Box<std::error::Error>> {
/// use dialoguer::PasswordInput;
///
/// let password = PasswordInput::new().with_prompt("New Password")
///     .with_confirmation("Confirm password", "Passwords mismatching")
///     .interact()?;
/// println!("Length of the password is: {}", password.len());
/// # Ok(()) } fn main() { test().unwrap(); }
/// ```
pub struct PasswordInput<'a> {
    prompt: String,
    theme: &'a dyn Theme,
    allow_empty_password: bool,
    confirmation_prompt: Option<(String, String)>,
}

impl<'a> Default for Confirmation<'a> {
    fn default() -> Confirmation<'a> {
        Confirmation::new()
    }
}

impl<'a> Confirmation<'a> {
    /// Creates the prompt with a specific text.
    pub fn new() -> Confirmation<'static> {
        Confirmation::with_theme(&SimpleTheme)
    }

    /// Sets a theme other than the default one.
    pub fn with_theme(theme: &'a dyn Theme) -> Confirmation<'a> {
        Confirmation {
            prompt: "".into(),
            default: true,
            show_default: true,
            theme,
        }
    }

    /// Sets the confirmation text.
    pub fn with_prompt<S: Into<String>>(&mut self, prompt: S) -> &mut Confirmation<'a> {
        self.prompt = prompt.into();
        self
    }

    /// Sets the confirmation text.
    #[deprecated(note = "Use with_prompt() instead", since = "0.6.0")]
    #[inline]
    pub fn with_text(&mut self, text: &str) -> &mut Confirmation<'a> {
        self.with_prompt(text)
    }

    /// Overrides the default.
    pub fn default(&mut self, val: bool) -> &mut Confirmation<'a> {
        self.default = val;
        self
    }

    /// Disables or enables the default value display.
    ///
    /// The default is to append `[y/n]` to the prompt to tell the
    /// user which keys to press. This also renders the default choice
    /// in uppercase. The default is selected on enter.
    pub fn show_default(&mut self, val: bool) -> &mut Confirmation<'a> {
        self.show_default = val;
        self
    }

    /// Enables user interaction and returns the result.
    ///
    /// If the user confirms the result is `true`, `false` otherwise.
    /// The dialog is rendered on stderr.
    pub fn interact(&self) -> io::Result<bool> {
        self.interact_on(&Term::stderr())
    }

    /// Like `interact` but allows a specific terminal to be set.
    pub fn interact_on(&self, term: &Term) -> io::Result<bool> {
        let mut render = TermThemeRenderer::new(term, self.theme);

        render.confirmation_prompt(
            &self.prompt,
            if self.show_default {
                Some(self.default)
            } else {
                None
            },
        )?;

        term.hide_cursor()?;
        term.flush()?;

        loop {
            let input = term.read_char()?;
            let rv = match input {
                'y' | 'Y' => true,
                'n' | 'N' => false,
                '\n' | '\r' => self.default,
                _ => {
                    continue;
                }
            };

            term.clear_line()?;
            render.confirmation_prompt_selection(&self.prompt, rv)?;
            term.show_cursor()?;
            term.flush()?;

            return Ok(rv);
        }
    }
}

impl<'a, T> Default for Input<'a, T>
where
    T: Clone + FromStr + Display,
    T::Err: Display + Debug,
{
    fn default() -> Input<'a, T> {
        Input::new()
    }
}

impl<'a, T> Input<'a, T>
where
    T: Clone + FromStr + Display,
    T::Err: Display + Debug,
{
    /// Creates a new input prompt.
    pub fn new() -> Input<'static, T> {
        Input::with_theme(&SimpleTheme)
    }

    /// Creates an input with a specific theme.
    pub fn with_theme(theme: &'a dyn Theme) -> Input<'a, T> {
        Input {
            prompt: "".into(),
            default: None,
            show_default: true,
            initial_text: None,
            theme,
            permit_empty: false,
            validator: None,
        }
    }

    /// Sets the input prompt.
    pub fn with_prompt<S: Into<String>>(&mut self, prompt: S) -> &mut Input<'a, T> {
        self.prompt = prompt.into();
        self
    }

    /// Sets whether the default can be editable.
    pub fn with_initial_text<S: Into<String>>(&mut self, val: S) -> &mut Input<'a, T> {
        self.initial_text = Some(val.into());
        self
    }

    /// Sets a default.
    ///
    /// Out of the box the prompt does not have a default and will continue
    /// to display until the user hit enter.  If a default is set the user
    /// can instead accept the default with enter.
    pub fn default(&mut self, value: T) -> &mut Input<'a, T> {
        self.default = Some(value);
        self
    }

    /// Enables or disables an empty input
    ///
    /// By default, if there is no default value set for the input, the user must input a non-empty string.
    pub fn allow_empty(&mut self, val: bool) -> &mut Input<'a, T> {
        self.permit_empty = val;
        self
    }

    /// Disables or enables the default value display.
    ///
    /// The default is to append `[default]` to the prompt to tell the
    /// user that a default is acceptable.
    pub fn show_default(&mut self, val: bool) -> &mut Input<'a, T> {
        self.show_default = val;
        self
    }

    /// Registers a validator.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use dialoguer::Input;
    /// let mail: String = Input::new()
    ///     .with_prompt("Enter email")
    ///     .validate_with(|input: &str| -> Result<(), &str> {
    ///         if input.contains('@') {
    ///             Ok(())
    ///         } else {
    ///             Err("This is not a mail address")
    ///         }
    ///     })
    ///     .interact()
    ///     .unwrap();
    /// ```
    pub fn validate_with<V: Validator + 'static>(&mut self, validator: V) -> &mut Input<'a, T> {
        let old_validator_func = self.validator.take();

        self.validator = Some(Box::new(move |value: &str| -> Option<String> {
            if let Some(old) = old_validator_func.as_ref() {
                if let Some(err) = old(value) {
                    return Some(err);
                }
            }

            match validator.validate(value) {
                Ok(()) => None,
                Err(err) => Some(err.to_string()),
            }
        }));

        self
    }

    /// Enables user interaction and returns the result.
    ///
    /// If the user confirms the result is `true`, `false` otherwise.
    /// The dialog is rendered on stderr.
    pub fn interact(&self) -> io::Result<T> {
        self.interact_on(&Term::stderr())
    }

    /// Like `interact` but allows a specific terminal to be set.
    pub fn interact_on(&self, term: &Term) -> io::Result<T> {
        let mut render = TermThemeRenderer::new(term, self.theme);

        loop {
            let default_string = self.default.as_ref().map(|x| x.to_string());

            render.input_prompt(
                &self.prompt,
                if self.show_default {
                    default_string.as_ref().map(|x| x.as_str())
                } else {
                    None
                },
            )?;
            term.flush()?;

            let input = if let Some(initial_text) = self.initial_text.as_ref() {
                term.read_line_initial_text(initial_text)?
            } else {
                term.read_line()?
            };

            render.add_line();
            term.clear_line()?;
            render.clear()?;

            if input.is_empty() {
                if let Some(ref default) = self.default {
                    render.input_prompt_selection(&self.prompt, &default.to_string())?;
                    term.flush()?;
                    return Ok(default.clone());
                } else if !self.permit_empty {
                    continue;
                }
            }

            match input.parse::<T>() {
                Ok(value) => {
                    if let Some(ref validator) = self.validator {
                        if let Some(err) = validator(&input) {
                            render.error(&err)?;
                            continue;
                        }
                    }

                    render.input_prompt_selection(&self.prompt, &input)?;
                    term.flush()?;

                    return Ok(value);
                }
                Err(err) => {
                    render.error(&err.to_string())?;
                    continue;
                }
            }
        }
    }
}

impl<'a> Default for PasswordInput<'a> {
    fn default() -> PasswordInput<'a> {
        PasswordInput::new()
    }
}

impl<'a> PasswordInput<'a> {
    /// Creates a new input prompt.
    pub fn new() -> PasswordInput<'static> {
        PasswordInput::with_theme(&SimpleTheme)
    }

    /// Creates the password input with a specific theme.
    pub fn with_theme(theme: &'a dyn Theme) -> PasswordInput<'a> {
        PasswordInput {
            prompt: "".into(),
            theme,
            allow_empty_password: false,
            confirmation_prompt: None,
        }
    }

    /// Sets the prompt.
    pub fn with_prompt<S: Into<String>>(&mut self, prompt: S) -> &mut PasswordInput<'a> {
        self.prompt = prompt.into();
        self
    }

    /// Enables confirmation prompting.
    pub fn with_confirmation<A, B>(&mut self, prompt: A, mismatch_err: B) -> &mut PasswordInput<'a>
    where
        A: Into<String>,
        B: Into<String>,
    {
        self.confirmation_prompt = Some((prompt.into(), mismatch_err.into()));
        self
    }

    /// Allows/Disables empty password.
    ///
    /// By default this setting is set to false (i.e. password is not empty).
    pub fn allow_empty_password(&mut self, allow_empty_password: bool) -> &mut PasswordInput<'a> {
        self.allow_empty_password = allow_empty_password;
        self
    }

    /// Enables user interaction and returns the result.
    ///
    /// If the user confirms the result is `true`, `false` otherwise.
    /// The dialog is rendered on stderr.
    pub fn interact(&self) -> io::Result<String> {
        self.interact_on(&Term::stderr())
    }

    /// Like `interact` but allows a specific terminal to be set.
    pub fn interact_on(&self, term: &Term) -> io::Result<String> {
        let mut render = TermThemeRenderer::new(term, self.theme);
        render.set_prompts_reset_height(false);

        loop {
            let password = self.prompt_password(&mut render, &self.prompt)?;

            if let Some((ref prompt, ref err)) = self.confirmation_prompt {
                let pw2 = self.prompt_password(&mut render, &prompt)?;

                if password == pw2 {
                    render.clear()?;
                    render.password_prompt_selection(&self.prompt)?;
                    term.flush()?;
                    return Ok(password);
                }

                render.error(err)?;
            } else {
                render.clear()?;
                render.password_prompt_selection(&self.prompt)?;
                term.flush()?;

                return Ok(password);
            }
        }
    }

    fn prompt_password(&self, render: &mut TermThemeRenderer, prompt: &str) -> io::Result<String> {
        loop {
            render.password_prompt(prompt)?;
            render.term().flush()?;

            let input = render.term().read_secure_line()?;

            render.add_line();

            if !input.is_empty() || self.allow_empty_password {
                return Ok(input);
            }
        }
    }
}
