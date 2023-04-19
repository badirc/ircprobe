use anyhow::Result;

pub fn parse(input: String) -> Result<Vec<String>> {
    let results = input
        .split("AND")
        .map(|s| s.trim())
        .map(|s| s.to_owned() + "\r\n")
        .collect::<Vec<String>>();

    Ok(results)
}
