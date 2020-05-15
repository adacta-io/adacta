use std::collections::{HashMap, HashSet};
use std::iter::Sum;
use std::ops::{Add, AddAssign, Mul};
use std::path::{Path, PathBuf};

use anyhow::Result;
use serde::{Deserialize, Serialize};

use async_trait::async_trait;

use crate::config::BayesicPigeonhole as Config;

use crate::model::Label;

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, Deserialize, Serialize)]
struct Counter {
    pub pro: u64,
    pub con: u64,
}

impl Counter {
    pub fn with(pro: u64, con: u64) -> Self { return Self { pro, con }; }

    pub fn neutral() -> Self { return Self::with(0, 0); }

    pub fn pro() -> Self { return Self::with(1, 0); }

    pub fn con() -> Self { return Self::with(0, 1); }
}

impl Add<&Counter> for Counter {
    type Output = Counter;

    fn add(self, rhs: &Counter) -> Self::Output {
        return Self {
            pro: self.pro + rhs.pro,
            con: self.con + rhs.con,
        };
    }
}

impl AddAssign<Counter> for Counter {
    fn add_assign(&mut self, rhs: Counter) {
        self.pro += rhs.pro;
        self.con += rhs.con;
    }
}

impl Mul<u64> for Counter {
    type Output = Counter;

    fn mul(self, rhs: u64) -> Self::Output {
        return Self {
            pro: self.pro * rhs,
            con: self.con * rhs,
        };
    }
}

impl<'a> Sum<&'a Counter> for Counter {
    fn sum<I: Iterator<Item = &'a Counter>>(iter: I) -> Self {
        return iter.fold(Counter::neutral(), |acc, v| acc + v);
    }
}

impl Default for Counter {
    fn default() -> Self { return Self::neutral(); }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct Classifier {
    data: HashMap<String, Counter>,
}

impl Classifier {
    pub fn new() -> Self {
        return Self {
            data: HashMap::new(),
        };
    }

    pub fn train_pro(&mut self, tokens: &HashMap<String, u64>) {
        self.train(tokens, Counter::pro());
    }

    pub fn train_con(&mut self, tokens: &HashMap<String, u64>) {
        self.train(tokens, Counter::con());
    }

    fn train(&mut self, tokens: &HashMap<String, u64>, counter: Counter) {
        for (token, &count) in tokens {
            // TODO: Avoid cloning if not required
            *self.data.entry(token.to_string()).or_default() += counter * count;
        }

        println!("{:?}", self.data);
    }

    pub fn classify(&self, tokens: &HashMap<String, u64>) -> f64 {
        // TODO: Cache this
        let tally = self.data.values().sum::<Counter>();

        let mut pro = 0.0;
        let mut con = 0.0;
        for (token, &count) in tokens {
            if let Some(counter) = self.data.get(token) {
                if counter.pro != 0 {
                    pro += count as f64 * (counter.pro as f64 / tally.pro as f64).ln();
                }

                if counter.con != 0 {
                    con += count as f64 * (counter.con as f64 / tally.con as f64).ln();
                }
            };
        }

        let total = pro + con;
        if total == 0.0 {
            return 0.0;
        }

        return pro / total;
    }
}

pub struct Pigeonhole {
    path: PathBuf,
    certainty: f64,

    classifiers: HashMap<Label, Classifier>,
}

impl Pigeonhole {
    pub async fn from_config(config: Config) -> Result<Self> {
        let path = PathBuf::from(config.path);

        let classifiers = Self::load(&path).await?;

        return Ok(Self {
            path,
            certainty: config.certainty,
            classifiers,
        });
    }

    async fn load(path: impl AsRef<Path>) -> Result<HashMap<Label, Classifier>> {
        match tokio::fs::read(path).await {
            Ok(data) => {
                return Ok(bincode::deserialize(&data)?);
            }
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
                return Ok(HashMap::new());
            }
            Err(err) => {
                return Err(err.into());
            }
        };
    }

    async fn save(path: impl AsRef<Path>, classifiers: &HashMap<Label, Classifier>) -> Result<()> {
        let data = bincode::serialize(classifiers)?;
        tokio::fs::write(path, &data).await?;

        return Ok(());
    }

    fn tokenize(text: &str) -> HashMap<String, u64> {
        let stream = text
            .split_whitespace()
            .map(|s| s.replace(|c: char| c.is_ascii_punctuation(), ""))
            .map(|s| s.to_lowercase());

        let mut tokens = HashMap::new();
        for token in stream {
            *tokens.entry(token).or_insert(0) += 1;
        }
        return tokens;
    }
}

#[async_trait]
impl super::Pigeonhole for Pigeonhole {
    fn labels(&self) -> HashSet<Label> { return self.classifiers.keys().cloned().collect(); }

    async fn guess(&self, text: &str) -> Result<HashSet<Label>> {
        let tokens = Self::tokenize(text);

        return Ok(self
            .classifiers
            .iter()
            .filter_map(|(label, classifier)| {
                return (classifier.classify(&tokens) >= self.certainty).then(|| label.clone());
            })
            .collect());
    }

    async fn train(&mut self, text: &str, labels: HashSet<Label>) -> Result<()> {
        let tokens = Self::tokenize(text);

        for label in &labels {
            self.classifiers.entry(label.clone()).or_insert_with(|| {
                println!("New label: {:?}", label);
                Classifier::new()
            });
        }

        for (label, classifier) in &mut self.classifiers {
            if labels.contains(label) {
                println!("Train pro: {:?}: {:?}", label, tokens);
                classifier.train_pro(&tokens);
            } else {
                println!("Train con: {:?}: {:?}", label, tokens);
                classifier.train_con(&tokens);
            }
        }

        Self::save(&self.path, &self.classifiers).await?;

        return Ok(());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pigeonhole::Pigeonhole as _;

    #[tokio::test]
    async fn classify() {
        let meat = [
            "sirloin meatloaf ham hock sausage meatball tongue prosciutto picanha turkey ball tip pastrami. ribeye chicken sausage, ham hock landjaeger pork belly pancetta ball tip tenderloin leberkas shank shankle rump. cupim short ribs ground round biltong tenderloin ribeye drumstick landjaeger short loin doner chicken shoulder spare ribs fatback boudin. pork chop shank shoulder, t-bone beef ribs drumstick landjaeger meatball.",
            "sirloin porchetta drumstick, pastrami bresaola landjaeger turducken kevin ham capicola corned beef. pork cow capicola, pancetta turkey tri-tip doner ball tip salami. fatback pastrami rump pancetta landjaeger. doner porchetta meatloaf short ribs cow chuck jerky pork chop landjaeger picanha tail.",
        ];
        let vegg = [
            "beetroot water spinach okra water chestnut ricebean pea catsear courgette summer purslane. water spinach arugula pea tatsoi aubergine spring onion bush tomato kale radicchio turnip chicory salsify pea sprouts fava bean. dandelion zucchini burdock yarrow chickpea dandelion sorrel courgette turnip greens tigernut soybean radish artichoke wattle seed endive groundnut broccoli arugula.",
            "pea horseradish azuki bean lettuce avocado asparagus okra. kohlrabi radish okra azuki bean corn fava bean mustard tigernut jã­cama green bean celtuce collard greens avocado quandong fennel gumbo black-eyed pea. grape silver beet watercress potato tigernut corn groundnut. chickweed okra pea winter purslane coriander yarrow sweet pepper radish garlic brussels sprout groundnut summer purslane earthnut pea tomato spring onion azuki bean gourd. gumbo kakadu plum komatsuna black-eyed pea green bean zucchini gourd winter purslane silver beet rock melon radish asparagus spinach.",
        ];

        let mut pigeonhole = Pigeonhole::from_config(Config {
            path: String::from("bayesian"),
            certainty: 0.1,
        })
        .await
        .unwrap();

        pigeonhole
            .train(
                &meat[0],
                vec!["Meat".into(), "Foo".into()].into_iter().collect(),
            )
            .await
            .unwrap();
        assert_eq!(
            pigeonhole
                .classifiers
                .get("Meat")
                .unwrap()
                .data
                .get("sirloin")
                .unwrap(),
            &Counter::with(1, 0)
        );
        assert_eq!(
            pigeonhole
                .classifiers
                .get("Foo")
                .unwrap()
                .data
                .get("sirloin")
                .unwrap(),
            &Counter::with(1, 0)
        );
        assert_eq!(
            pigeonhole
                .classifiers
                .get("Meat")
                .unwrap()
                .data
                .get("prosciutto")
                .unwrap(),
            &Counter::with(1, 0)
        );
        assert_eq!(
            pigeonhole
                .classifiers
                .get("Foo")
                .unwrap()
                .data
                .get("prosciutto")
                .unwrap(),
            &Counter::with(1, 0)
        );

        pigeonhole
            .train(
                &meat[1],
                vec!["Meat".into(), "Bar".into()].into_iter().collect(),
            )
            .await
            .unwrap();
        assert_eq!(
            pigeonhole
                .classifiers
                .get("Meat")
                .unwrap()
                .data
                .get("sirloin")
                .unwrap(),
            &Counter::with(2, 0)
        );
        assert_eq!(
            pigeonhole
                .classifiers
                .get("Foo")
                .unwrap()
                .data
                .get("sirloin")
                .unwrap(),
            &Counter::with(1, 1)
        );
        assert_eq!(
            pigeonhole
                .classifiers
                .get("Bar")
                .unwrap()
                .data
                .get("sirloin")
                .unwrap(),
            &Counter::with(1, 0)
        );
        assert_eq!(
            pigeonhole
                .classifiers
                .get("Meat")
                .unwrap()
                .data
                .get("prosciutto")
                .unwrap(),
            &Counter::with(1, 0)
        );
        assert_eq!(
            pigeonhole
                .classifiers
                .get("Foo")
                .unwrap()
                .data
                .get("prosciutto")
                .unwrap(),
            &Counter::with(1, 0)
        );
        assert_eq!(
            pigeonhole
                .classifiers
                .get("Meat")
                .unwrap()
                .data
                .get("bresaola")
                .unwrap(),
            &Counter::with(1, 0)
        );
        assert_eq!(
            pigeonhole
                .classifiers
                .get("Foo")
                .unwrap()
                .data
                .get("bresaola")
                .unwrap(),
            &Counter::with(0, 1)
        );
        assert_eq!(
            pigeonhole
                .classifiers
                .get("Bar")
                .unwrap()
                .data
                .get("bresaola")
                .unwrap(),
            &Counter::with(1, 0)
        );

        pigeonhole
            .train(
                &vegg[0],
                vec!["Vegg".into(), "Foo".into()].into_iter().collect(),
            )
            .await
            .unwrap();
        pigeonhole
            .train(
                &vegg[1],
                vec!["Vegg".into(), "Bar".into()].into_iter().collect(),
            )
            .await
            .unwrap();

        assert_eq!(
            pigeonhole
                .classifiers
                .get("Meat")
                .unwrap()
                .classify(&Pigeonhole::tokenize("salami")),
            1.0f64
        );
        assert_eq!(
            pigeonhole
                .classifiers
                .get("Foo")
                .unwrap()
                .classify(&Pigeonhole::tokenize("salami")),
            0.0f64
        );
        assert_eq!(
            pigeonhole
                .classifiers
                .get("Bar")
                .unwrap()
                .classify(&Pigeonhole::tokenize("salami")),
            1.0f64
        );

        assert_eq!(
            pigeonhole.guess("salami pancetta beef ribs").await.unwrap(),
            vec!["Meat".into(), "Foo".into(), "Bar".into()]
                .into_iter()
                .collect()
        );
    }
}
