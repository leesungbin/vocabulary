extern crate csv;
extern crate reqwest;
extern crate tokio;

use std::collections::HashMap;
use std::error::Error;

mod voca;
use crate::voca::Word;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut dictionary = HashMap::new();
    let mut rdr = csv::Reader::from_path("test.csv")?;

    for result in rdr.records() {
        let record = result?;

        let spelling = String::from(&record[0]);
        let means = String::from(&record[1]);
        let w: Word;
        if !means.is_empty() {
            let means: Vec<&str> = means.split(",").collect();
            let means = {
                let mut v: Vec<String> = Vec::new();
                for mean in means {
                    v.push(String::from(mean.trim()));
                }
                v
            };
            let means = vec![means];
            w = Word { pos: None, means };
        } else {
            w = voca::search::naver(&spelling).await?;
        }
        dictionary.insert(spelling, w);
    }

    println!("{:#?}", dictionary);
    Ok(())
}
