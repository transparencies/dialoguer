extern crate console;
extern crate dialoguer;

use console::Term;
use dialoguer::{theme::ColorfulTheme, Confirm, Input, MultiSelect, Password, Select, Sort};

fn main() {
    let items = &[
        "Ice Cream",
        "Vanilla Cupcake",
        "Chocolate Muffin",
        "A Pile of sweet, sweet mustard",
    ];
    let term = Term::buffered_stderr();
    let theme = ColorfulTheme::default();

    println!("All the following controls are run in a buffered terminal");
    Confirm::with_theme(&theme)
        .with_prompt("Do you want to continue?")
        .interact_on(&term)
        .unwrap();

    let _: String = Input::with_theme(&theme)
        .with_prompt("Your name")
        .interact_on(&term)
        .unwrap();

    Password::with_theme(&theme)
        .with_prompt("Your password")
        .with_confirmation("Confirm", "Passwords do not match")
        .interact_on(&term)
        .unwrap();

    Select::with_theme(&theme)
        .with_prompt("Pick an item")
        .items(items)
        .interact_on(&term)
        .unwrap();

    MultiSelect::with_theme(&theme)
        .with_prompt("Pick some items")
        .items(items)
        .interact_on(&term)
        .unwrap();

    Sort::with_theme(&theme)
        .with_prompt("Order these items")
        .items(items)
        .interact_on(&term)
        .unwrap();
}