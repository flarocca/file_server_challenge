use std::path::PathBuf;

pub fn get_path_from_str(input: &str) -> anyhow::Result<PathBuf> {
    let path = if input.starts_with("~") {
        let home = home::home_dir().ok_or_else(|| anyhow::anyhow!("Cannot find home dir"))?;
        PathBuf::from(input.replacen("~", home.to_str().unwrap(), 1))
    } else {
        PathBuf::from(input)
    };

    Ok(path)
}
