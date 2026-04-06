use anyhow::Result;
use dialoguer::{Select, theme::ColorfulTheme};

pub fn select_desk(desks: &[String]) -> Result<Option<String>> {
    if desks.is_empty() {
        return Ok(None);
    }

    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Chọn deck để import")
        .default(0)
        .items(desks)
        .interact_opt()?;

    Ok(selection.map(|idx| desks[idx].clone()))
}
