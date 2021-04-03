#[derive(Debug)]
pub struct Word {
  pub pos: Option<Vec<String>>,
  pub means: Vec<Vec<String>>,
}

impl Word {
  pub fn new(pos: Vec<String>, means: Vec<Vec<String>>) -> Word {
    Word {
      pos: Some(pos),
      means,
    }
  }
}

impl PartialEq for Word {
  fn eq(&self, other: &Self) -> bool {
    self.pos == other.pos && self.means == other.means
  }
}

const NAVER: &'static str = "https://en.dict.naver.com/api3/enko/search?lang=en&query=";

pub mod search {
  use super::*;
  use regex::Regex;
  use serde_json::Value;
  use std::error::Error;

  pub fn escape_span_of_meaning(mean: &str) -> String {
    let s: Vec<&str> = mean.split("(=").collect();
    let s: Vec<&str> = s[0].split("(→").collect();
    let mut s: Vec<&str> = s[0].split("(↔").collect();
    s.truncate(1);

    let re = Regex::new(r#"<(?:"[^"]*"['"]*|'[^']*'['"]*|[^'">])+>"#).unwrap();
    let fields: Vec<&str> = re.split(s[0].trim()).collect();
    fields.join("")
  }

  pub async fn naver(word: &str) -> Result<Word, Box<dyn Error>> {
    let url = format!("{}{}", NAVER, word);
    let body = reqwest::get(url).await?.text().await?;
    let data: Value = serde_json::from_str(&body)?;

    let part_of_sppech =
      &data["searchResultMap"]["searchResultListMap"]["WORD"]["items"][0]["meansCollector"];

    if let Value::Array(arr) = part_of_sppech {
      let mut pos: Vec<String> = Vec::new();
      let mut meanings: Vec<Vec<String>> = Vec::new();

      let word = {
        for part in arr {
          if let Value::String(p) = &part["partOfSpeech"] {
            pos.push(p.to_string());
          }
          let submeans = {
            let mut subs: Vec<String> = Vec::new();
            if let Value::Array(means) = &part["means"] {
              for mean in means {
                if let Value::String(m) = &mean["value"] {
                  subs.push(escape_span_of_meaning(m));
                }
              }
            }
            subs
          };
          meanings.push(submeans);
        }
        Word::new(pos, meanings)
      };

      Ok(word)
    } else {
      panic!("invalid search")
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::error::Error;

  #[tokio::test]
  async fn get_devour() -> Result<(), Box<dyn Error>> {
    let word = search::naver("devour").await?;

    let pos = vec![String::from("동사")];
    let means = vec![vec![
      String::from("(특히 몹시 배가 고파서) 걸신 들린 듯 먹다"),
      String::from("(엄청난 관심과 열의로) 집어삼킬듯이[빨아들이듯이] 읽다[보다]"),
      String::from("집어삼키다, 파괴하다"),
    ]];
    let devour = Word {
      pos: Some(pos),
      means,
    };

    assert_eq!(devour, word);
    Ok(())
  }

  #[test]
  fn escape_span() {
    let t = "(Abbr.) <span class=\'related_word\' lang=\'en\' >a/c</span>. 계좌";
    let result = search::escape_span_of_meaning(t);
    assert_eq!(String::from("(Abbr.) a/c. 계좌"), result);
  }
}
