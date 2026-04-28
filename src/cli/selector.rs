use anyhow::Result;
use dialoguer::{Select, theme::ColorfulTheme};

pub fn select_desk(desks: &[String]) -> Result<Option<String>> {
    select_item("Chọn deck để import", desks)
}

pub fn select_item(prompt: &str, items: &[String]) -> Result<Option<String>> {
    if items.is_empty() {
        return Ok(None);
    }

    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt(prompt)
        .default(0)
        .items(items)
        .interact_opt()?;

    Ok(selection.map(|idx| items[idx].clone()))
}
