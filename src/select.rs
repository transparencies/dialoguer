use std::{io, iter::repeat, ops::Rem};

use crate::theme::{SimpleTheme, TermThemeRenderer, Theme};

use console::{Key, Term};

/// Renders a selection menu.
pub struct Select<'a> {
    default: usize,
    items: Vec<String>,
    prompt: Option<String>,
    clear: bool,
    theme: &'a dyn Theme,
    paged: bool,
}

/// Renders a multi select checkbox menu.
pub struct Checkboxes<'a> {
    defaults: Vec<bool>,
    items: Vec<String>,
    prompt: Option<String>,
    clear: bool,
    theme: &'a dyn Theme,
    paged: bool,
}

/// Renders a list to order.
pub struct OrderList<'a> {
    items: Vec<String>,
    prompt: Option<String>,
    clear: bool,
    theme: &'a dyn Theme,
    paged: bool,
}

impl<'a> Default for Select<'a> {
    fn default() -> Select<'a> {
        Select::new()
    }
}

impl<'a> Select<'a> {
    /// Creates the prompt with a specific text.
    pub fn new() -> Select<'static> {
        Select::with_theme(&SimpleTheme)
    }

    /// Same as `new` but with a specific theme.
    pub fn with_theme(theme: &'a dyn Theme) -> Select<'a> {
        Select {
            default: !0,
            items: vec![],
            prompt: None,
            clear: true,
            theme,
            paged: false,
        }
    }

    /// Enables or disables paging
    pub fn paged(&mut self, val: bool) -> &mut Select<'a> {
        self.paged = val;
        self
    }

    /// Sets the clear behavior of the menu.
    ///
    /// The default is to clear the menu.
    pub fn clear(&mut self, val: bool) -> &mut Select<'a> {
        self.clear = val;
        self
    }

    /// Sets a default for the menu
    pub fn default(&mut self, val: usize) -> &mut Select<'a> {
        self.default = val;
        self
    }

    /// Add a single item to the selector.
    pub fn item<T: ToString>(&mut self, item: T) -> &mut Select<'a> {
        self.items.push(item.to_string());
        self
    }

    /// Adds multiple items to the selector.
    pub fn items<T: ToString>(&mut self, items: &[T]) -> &mut Select<'a> {
        for item in items {
            self.items.push(item.to_string());
        }
        self
    }

    /// Prefaces the menu with a prompt.
    ///
    /// When a prompt is set the system also prints out a confirmation after
    /// the selection.
    pub fn with_prompt<S: Into<String>>(&mut self, prompt: S) -> &mut Select<'a> {
        self.prompt = Some(prompt.into());
        self
    }

    /// Enables user interaction and returns the result.
    ///
    /// The index of the selected item.
    /// The dialog is rendered on stderr.
    pub fn interact(&self) -> io::Result<usize> {
        self.interact_on(&Term::stderr())
    }

    /// Enables user interaction and returns the result.
    ///
    /// The index of the selected item. None if the user
    /// cancelled with Esc or 'q'.
    /// The dialog is rendered on stderr.
    pub fn interact_opt(&self) -> io::Result<Option<usize>> {
        self.interact_on_opt(&Term::stderr())
    }

    /// Like `interact` but allows a specific terminal to be set.
    pub fn interact_on(&self, term: &Term) -> io::Result<usize> {
        self._interact_on(term, false)?
            .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "Quit not allowed in this case"))
    }

    /// Like `interact_opt` but allows a specific terminal to be set.
    #[inline]
    pub fn interact_on_opt(&self, term: &Term) -> io::Result<Option<usize>> {
        self._interact_on(term, true)
    }

    /// Like `interact` but allows a specific terminal to be set.
    fn _interact_on(&self, term: &Term, allow_quit: bool) -> io::Result<Option<usize>> {
        let mut page = 0;

        let capacity = if self.paged {
            term.size().0 as usize - 1
        } else {
            self.items.len()
        };

        let pages = (self.items.len() / capacity) + 1;
        let mut render = TermThemeRenderer::new(term, self.theme);
        let mut sel = self.default;

        if let Some(ref prompt) = self.prompt {
            render.select_prompt(prompt)?;
        }

        let mut size_vec = Vec::new();

        for items in self
            .items
            .iter()
            .flat_map(|i| i.split('\n'))
            .collect::<Vec<_>>()
        {
            let size = &items.len();
            size_vec.push(size.clone());
        }

        loop {
            for (idx, item) in self
                .items
                .iter()
                .enumerate()
                .skip(page * capacity)
                .take(capacity)
            {
                render.select_prompt_item(item, sel == idx)?;
            }

            term.hide_cursor()?;
            term.flush()?;

            match term.read_key()? {
                Key::ArrowDown | Key::Char('j') => {
                    if sel == !0 {
                        sel = 0;
                    } else {
                        sel = (sel as u64 + 1).rem(self.items.len() as u64) as usize;
                    }
                }
                Key::Escape | Key::Char('q') => {
                    if allow_quit {
                        if self.clear {
                            term.clear_last_lines(self.items.len())?;
                            term.show_cursor()?;
                            term.flush()?;
                        }

                        return Ok(None);
                    }
                }
                Key::ArrowUp | Key::Char('k') => {
                    if sel == !0 {
                        sel = self.items.len() - 1;
                    } else {
                        sel = ((sel as i64 - 1 + self.items.len() as i64)
                            % (self.items.len() as i64)) as usize;
                    }
                }
                Key::ArrowLeft | Key::Char('h') => {
                    if self.paged {
                        if page == 0 {
                            page = pages - 1;
                        } else {
                            page -= 1;
                        }

                        sel = page * capacity;
                    }
                }
                Key::ArrowRight | Key::Char('l') => {
                    if self.paged {
                        if page == pages - 1 {
                            page = 0;
                        } else {
                            page += 1;
                        }

                        sel = page * capacity;
                    }
                }

                Key::Enter | Key::Char(' ') if sel != !0 => {
                    if self.clear {
                        render.clear()?;
                    }

                    if let Some(ref prompt) = self.prompt {
                        render.select_prompt_selection(prompt, &self.items[sel])?;
                    }

                    term.show_cursor()?;
                    term.flush()?;

                    return Ok(Some(sel));
                }
                _ => {}
            }

            if sel != !0 && (sel < page * capacity || sel >= (page + 1) * capacity) {
                page = sel / capacity;
            }

            render.clear_preserve_prompt(&size_vec)?;
        }
    }
}

impl<'a> Default for Checkboxes<'a> {
    fn default() -> Checkboxes<'a> {
        Checkboxes::new()
    }
}

impl<'a> Checkboxes<'a> {
    /// Creates a new checkbox object.
    pub fn new() -> Checkboxes<'static> {
        Checkboxes::with_theme(&SimpleTheme)
    }

    /// Sets a theme other than the default one.
    pub fn with_theme(theme: &'a dyn Theme) -> Checkboxes<'a> {
        Checkboxes {
            items: vec![],
            defaults: vec![],
            clear: true,
            prompt: None,
            theme,
            paged: false,
        }
    }

    /// Enables or disables paging
    pub fn paged(&mut self, val: bool) -> &mut Checkboxes<'a> {
        self.paged = val;
        self
    }

    /// Sets the clear behavior of the checkbox menu.
    ///
    /// The default is to clear the checkbox menu.
    pub fn clear(&mut self, val: bool) -> &mut Checkboxes<'a> {
        self.clear = val;
        self
    }

    /// Sets a defaults for the menu
    pub fn defaults(&mut self, val: &[bool]) -> &mut Checkboxes<'a> {
        self.defaults = val
            .to_vec()
            .iter()
            .cloned()
            .chain(repeat(false))
            .take(self.items.len())
            .collect();
        self
    }

    /// Add a single item to the selector.
    #[inline]
    pub fn item<T: ToString>(&mut self, item: T) -> &mut Checkboxes<'a> {
        self.item_checked(item, false)
    }

    /// Add a single item to the selector with a default checked state.
    pub fn item_checked<T: ToString>(&mut self, item: T, checked: bool) -> &mut Checkboxes<'a> {
        self.items.push(item.to_string());
        self.defaults.push(checked);
        self
    }

    /// Adds multiple items to the selector.
    pub fn items<T: ToString>(&mut self, items: &[T]) -> &mut Checkboxes<'a> {
        for item in items {
            self.items.push(item.to_string());
            self.defaults.push(false);
        }
        self
    }

    /// Adds multiple items to the selector with checked state
    pub fn items_checked<T: ToString>(&mut self, items: &[(T, bool)]) -> &mut Checkboxes<'a> {
        for &(ref item, checked) in items {
            self.items.push(item.to_string());
            self.defaults.push(checked);
        }
        self
    }

    /// Prefaces the menu with a prompt.
    ///
    /// When a prompt is set the system also prints out a confirmation after
    /// the selection.
    pub fn with_prompt<S: Into<String>>(&mut self, prompt: S) -> &mut Checkboxes<'a> {
        self.prompt = Some(prompt.into());
        self
    }

    /// Enables user interaction and returns the result.
    ///
    /// The user can select the items with the space bar and on enter
    /// the selected items will be returned.
    pub fn interact(&self) -> io::Result<Vec<usize>> {
        self.interact_on(&Term::stderr())
    }

    /// Like `interact` but allows a specific terminal to be set.
    pub fn interact_on(&self, term: &Term) -> io::Result<Vec<usize>> {
        let mut page = 0;

        let capacity = if self.paged {
            term.size().0 as usize - 1
        } else {
            self.items.len()
        };

        let pages = (self.items.len() / capacity) + 1;
        let mut render = TermThemeRenderer::new(term, self.theme);
        let mut sel = 0;

        if let Some(ref prompt) = self.prompt {
            render.multiselect_prompt(prompt)?;
        }

        let mut size_vec = Vec::new();

        for items in self
            .items
            .iter()
            .flat_map(|i| i.split('\n'))
            .collect::<Vec<_>>()
        {
            let size = &items.len();
            size_vec.push(size.clone());
        }

        let mut checked: Vec<bool> = self.defaults.clone();

        loop {
            for (idx, item) in self
                .items
                .iter()
                .enumerate()
                .skip(page * capacity)
                .take(capacity)
            {
                render.multiselect_prompt_item(item, checked[idx], sel == idx)?;
            }

            term.hide_cursor()?;
            term.flush()?;

            match term.read_key()? {
                Key::ArrowDown | Key::Char('j') => {
                    if sel == !0 {
                        sel = 0;
                    } else {
                        sel = (sel as u64 + 1).rem(self.items.len() as u64) as usize;
                    }
                }
                Key::ArrowUp | Key::Char('k') => {
                    if sel == !0 {
                        sel = self.items.len() - 1;
                    } else {
                        sel = ((sel as i64 - 1 + self.items.len() as i64)
                            % (self.items.len() as i64)) as usize;
                    }
                }
                Key::ArrowLeft | Key::Char('h') => {
                    if self.paged {
                        if page == 0 {
                            page = pages - 1;
                        } else {
                            page -= 1;
                        }

                        sel = page * capacity;
                    }
                }
                Key::ArrowRight | Key::Char('l') => {
                    if self.paged {
                        if page == pages - 1 {
                            page = 0;
                        } else {
                            page += 1;
                        }

                        sel = page * capacity;
                    }
                }
                Key::Char(' ') => {
                    checked[sel] = !checked[sel];
                }
                Key::Escape => {
                    if self.clear {
                        render.clear()?;
                    }

                    if let Some(ref prompt) = self.prompt {
                        render.multiselect_prompt_selection(prompt, &[][..])?;
                    }

                    term.show_cursor()?;
                    term.flush()?;

                    return Ok(self
                        .defaults
                        .clone()
                        .into_iter()
                        .enumerate()
                        .filter_map(|(idx, checked)| if checked { Some(idx) } else { None })
                        .collect());
                }
                Key::Enter => {
                    if self.clear {
                        render.clear()?;
                    }

                    if let Some(ref prompt) = self.prompt {
                        let selections: Vec<_> = checked
                            .iter()
                            .enumerate()
                            .filter_map(|(idx, &checked)| {
                                if checked {
                                    Some(self.items[idx].as_str())
                                } else {
                                    None
                                }
                            })
                            .collect();

                        render.multiselect_prompt_selection(prompt, &selections[..])?;
                    }

                    term.show_cursor()?;
                    term.flush()?;

                    return Ok(checked
                        .into_iter()
                        .enumerate()
                        .filter_map(|(idx, checked)| if checked { Some(idx) } else { None })
                        .collect());
                }
                _ => {}
            }

            if sel < page * capacity || sel >= (page + 1) * capacity {
                page = sel / capacity;
            }

            render.clear_preserve_prompt(&size_vec)?;
        }
    }
}

impl<'a> Default for OrderList<'a> {
    fn default() -> OrderList<'a> {
        OrderList::new()
    }
}

impl<'a> OrderList<'a> {
    /// Creates a new orderlist object.
    pub fn new() -> OrderList<'static> {
        OrderList::with_theme(&SimpleTheme)
    }

    /// Sets a theme other than the default one.
    pub fn with_theme(theme: &'a dyn Theme) -> OrderList<'a> {
        OrderList {
            items: vec![],
            clear: true,
            prompt: None,
            theme,
            paged: false,
        }
    }

    /// Enables or disables paging
    pub fn paged(&mut self, val: bool) -> &mut OrderList<'a> {
        self.paged = val;
        self
    }

    /// Sets the clear behavior of the checkbox menu.
    ///
    /// The default is to clear the checkbox menu.
    pub fn clear(&mut self, val: bool) -> &mut OrderList<'a> {
        self.clear = val;
        self
    }

    /// Add a single item to the selector.
    pub fn item<T: ToString>(&mut self, item: T) -> &mut OrderList<'a> {
        self.items.push(item.to_string());
        self
    }

    /// Adds multiple items to the selector.
    pub fn items<T: ToString>(&mut self, items: &[T]) -> &mut OrderList<'a> {
        for item in items {
            self.items.push(item.to_string());
        }
        self
    }

    /// Prefaces the menu with a prompt.
    ///
    /// When a prompt is set the system also prints out a confirmation after
    /// the selection.
    pub fn with_prompt<S: Into<String>>(&mut self, prompt: S) -> &mut OrderList<'a> {
        self.prompt = Some(prompt.into());
        self
    }

    /// Enables user interaction and returns the result.
    ///
    /// The user can order the items with the space bar and the arrows.
    /// On enter the ordered list will be returned.
    pub fn interact(&self) -> io::Result<Vec<usize>> {
        self.interact_on(&Term::stderr())
    }

    /// Like `interact` but allows a specific terminal to be set.
    pub fn interact_on(&self, term: &Term) -> io::Result<Vec<usize>> {
        let mut page = 0;

        let capacity = if self.paged {
            term.size().0 as usize - 1
        } else {
            self.items.len()
        };

        let pages = (self.items.len() as f64 / capacity as f64).ceil() as usize;
        let mut render = TermThemeRenderer::new(term, self.theme);
        let mut sel = 0;

        if let Some(ref prompt) = self.prompt {
            render.sort_prompt(prompt)?;
        }

        let mut size_vec = Vec::new();

        for items in self.items.iter().as_slice() {
            let size = &items.len();
            size_vec.push(size.clone());
        }

        let mut order: Vec<_> = (0..self.items.len()).collect();
        let mut checked: bool = false;

        loop {
            for (idx, item) in order
                .iter()
                .enumerate()
                .skip(page * capacity)
                .take(capacity)
            {
                render.sort_prompt_item(&self.items[*item], checked, sel == idx)?;
            }

            term.hide_cursor()?;
            term.flush()?;

            match term.read_key()? {
                Key::ArrowDown | Key::Char('j') => {
                    let old_sel = sel;

                    if sel == !0 {
                        sel = 0;
                    } else {
                        sel = (sel as u64 + 1).rem(self.items.len() as u64) as usize;
                    }

                    if checked && old_sel != sel {
                        order.swap(old_sel, sel);
                    }
                }
                Key::ArrowUp | Key::Char('k') => {
                    let old_sel = sel;

                    if sel == !0 {
                        sel = self.items.len() - 1;
                    } else {
                        sel = ((sel as i64 - 1 + self.items.len() as i64)
                            % (self.items.len() as i64)) as usize;
                    }

                    if checked && old_sel != sel {
                        order.swap(old_sel, sel);
                    }
                }
                Key::ArrowLeft | Key::Char('h') => {
                    if self.paged {
                        let old_sel = sel;
                        let old_page = page;

                        if page == 0 {
                            page = pages - 1;
                        } else {
                            page -= 1;
                        }

                        sel = page * capacity;

                        if checked {
                            let indexes: Vec<_> = if old_page == 0 {
                                let indexes1: Vec<_> = (0..=old_sel).rev().collect();
                                let indexes2: Vec<_> = (sel..self.items.len()).rev().collect();
                                [indexes1, indexes2].concat()
                            } else {
                                (sel..=old_sel).rev().collect()
                            };

                            for index in 0..(indexes.len() - 1) {
                                order.swap(indexes[index], indexes[index + 1]);
                            }
                        }
                    }
                }
                Key::ArrowRight | Key::Char('l') => {
                    if self.paged {
                        let old_sel = sel;
                        let old_page = page;

                        if page == pages - 1 {
                            page = 0;
                        } else {
                            page += 1;
                        }

                        sel = page * capacity;

                        if checked {
                            let indexes: Vec<_> = if old_page == pages - 1 {
                                let indexes1: Vec<_> = (old_sel..self.items.len()).collect();
                                let indexes2: Vec<_> = vec![0];
                                [indexes1, indexes2].concat()
                            } else {
                                (old_sel..=sel).collect()
                            };

                            for index in 0..(indexes.len() - 1) {
                                order.swap(indexes[index], indexes[index + 1]);
                            }
                        }
                    }
                }
                Key::Char(' ') => {
                    checked = !checked;
                }
                // TODO: Key::Escape
                Key::Enter => {
                    if self.clear {
                        render.clear()?;
                    }

                    if let Some(ref prompt) = self.prompt {
                        let list: Vec<_> = order
                            .iter()
                            .enumerate()
                            .map(|(_, item)| self.items[*item].as_str())
                            .collect();
                        render.sort_prompt_selection(prompt, &list[..])?;
                    }

                    term.show_cursor()?;
                    term.flush()?;

                    return Ok(order);
                }
                _ => {}
            }

            if sel < page * capacity || sel >= (page + 1) * capacity {
                page = sel / capacity;
            }

            render.clear_preserve_prompt(&size_vec)?;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_str() {
        let selections = &[
            "Ice Cream",
            "Vanilla Cupcake",
            "Chocolate Muffin",
            "A Pile of sweet, sweet mustard",
        ];

        assert_eq!(
            Select::new().default(0).items(&selections[..]).items,
            selections
        );
    }

    #[test]
    fn test_string() {
        let selections = vec!["a".to_string(), "b".to_string()];

        assert_eq!(
            Select::new().default(0).items(&selections[..]).items,
            selections
        );
    }

    #[test]
    fn test_ref_str() {
        let a = "a";
        let b = "b";

        let selections = &[a, b];

        assert_eq!(
            Select::new().default(0).items(&selections[..]).items,
            selections
        );
    }
}
