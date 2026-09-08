use anyhow::{Context, Result, bail};
use std::{fs, path::Path};

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let check = match args
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>()
        .as_slice()
    {
        ["openapi"] => false,
        ["openapi", "--check"] => true,
        _ => bail!("Usage: cargo xtask openapi [--check]"),
    };
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .context("Missing workspace root")?
        .join("swagger.json");
    let schema = format!(
        "{}\n",
        server::api::openapi::ApiDoc::merged().to_pretty_json()?
    );
    if check {
        let existing =
            fs::read_to_string(&path).context("Missing swagger.json; run cargo xtask openapi")?;
        if existing.replace("\r\n", "\n") != schema {
            bail!("swagger.json is stale; run cargo xtask openapi");
        }
        println!("OpenAPI schema is current");
    } else {
        fs::write(path, schema)?;
        println!("Exported swagger.json");
    }
    Ok(())
}
