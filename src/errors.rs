use std::{error::Error, panic};

use color_eyre::{config::HookBuilder, eyre::set_hook};

use crate::tui::restore;

pub fn install_hooks() -> color_eyre::Result<()> {
    let (panic_hook, eyre_hook) = HookBuilder::default().into_hooks();
    
    let panic_hook = panic_hook.into_panic_hook();
    panic::set_hook(Box::new(move |panic_info| {
        restore().unwrap();
        panic_hook(panic_info);
    }));

    let eyre_hook = eyre_hook.into_eyre_hook();
    set_hook(Box::new(
        move |error: &(dyn Error + 'static)| {
            restore().unwrap();
            eyre_hook(error)
        },
    ))?;

    Ok(())
}
