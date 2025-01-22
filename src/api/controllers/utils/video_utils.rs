use anyhow::Error;

pub fn parse_option_string(input: Option<String>) -> Result<Option<Vec<i32>>, Error> {
    if let Some(s) = input {
        println!("{}", s);
        let result: Vec<i32> = s
            .split(',')
            .map(|item| item.trim().parse::<i32>()) // Try to parse each item
            .collect::<Result<Vec<i32>, _>>()?; // Collect into a Vec or return an error

        Ok(Some(result))
    } else {
        Ok(None)
    }
}
