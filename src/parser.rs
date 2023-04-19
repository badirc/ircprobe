use anyhow::Result;

pub fn parse(input: String) -> Result<Vec<String>> {
    let results = input
        .split("AND")
        .map(|s| s.trim())
        .map(|s| s.to_owned() + "\r\n")
        .collect::<Vec<String>>();

    Ok(results)
}

#[test]
fn test_parser() {
    assert_eq!(parse("CAP LS".into()).unwrap(), vec!["CAP LS\r\n"]);
    assert_eq!(
        parse("CAP LS AND NICK liv".into()).unwrap(),
        vec!["CAP LS\r\n", "NICK liv\r\n"]
    );
}
