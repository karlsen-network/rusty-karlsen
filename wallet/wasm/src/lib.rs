use karlsen_cli_lib::karlsen_cli;
use wasm_bindgen::prelude::*;
use workflow_terminal::Options;
use workflow_terminal::Result;

#[wasm_bindgen]
pub async fn load_karlsen_wallet_cli() -> Result<()> {
    let options = Options {
        ..Options::default()
    };
    karlsen_cli(options, None).await?;
    Ok(())
}
