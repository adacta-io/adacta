use std::collections::{HashMap, HashSet};
use std::iter::Sum;
use std::ops::{Add, AddAssign, Mul};
use std::path::{Path, PathBuf};

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use crate::config::BayesicSuggester as Config;
use crate::model::Label;

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, Deserialize, Serialize)]
struct Counter {
    pub pro: u64,
    pub con: u64,
}

impl Counter {
    pub fn with(pro: u64, con: u64) -> Self { Self { pro, con } }

    pub fn neutral() -> Self { Self::with(0, 0) }

    pub fn pro() -> Self { Self::with(1, 0) }

    pub fn con() -> Self { Self::with(0, 1) }
}

impl Add<&Counter> for Counter {
    type Output = Counter;

    fn add(self, rhs: &Counter) -> Self::Output {
        Self {
            pro: self.pro + rhs.pro,
            con: self.con + rhs.con,
        }
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
        Self {
            pro: self.pro * rhs,
            con: self.con * rhs,
        }
    }
}

impl<'a> Sum<&'a Counter> for Counter {
    fn sum<I: Iterator<Item=&'a Counter>>(iter: I) -> Self {
        iter.fold(Counter::neutral(), |acc, v| acc + v)
    }
}

impl Default for Counter {
    fn default() -> Self { Self::neutral() }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct Classifier {
    data: HashMap<String, Counter>,
}

impl Classifier {
    pub fn new() -> Self {
        Self { data: HashMap::new() }
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

        pro / total
    }
}

pub struct Suggester {
    path: PathBuf,
    certainty: f64,

    classifiers: RwLock<HashMap<Label, Classifier>>,
}

impl Suggester {
    pub async fn from_config(config: Config) -> Result<Self> {
        let path = PathBuf::from(config.path);

        let classifiers = Self::load(&path).await?;

        Ok(Self {
            path,
            certainty: config.certainty,
            classifiers: RwLock::new(classifiers),
        })
    }

    async fn load(path: impl AsRef<Path>) -> Result<HashMap<Label, Classifier>> {
        match tokio::fs::read(path).await {
            Ok(data) => Ok(bincode::deserialize(&data)?),
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(HashMap::new()),
            Err(err) => Err(err.into()),
        }
    }

    async fn save(path: impl AsRef<Path>, classifiers: &HashMap<Label, Classifier>) -> Result<()> {
        let data = bincode::serialize(classifiers)?;
        tokio::fs::write(path, &data).await?;

        Ok(())
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

        tokens
    }

    async fn suggest(&self, tokens: &HashMap<String, u64>) -> HashSet<Label> {
        let classifiers = self.classifiers.read().await;

        return classifiers.iter()
            .filter_map(|(label, classifier)| (classifier.classify(&tokens) >= self.certainty).then(|| label.clone()))
            .collect();
    }
}

#[async_trait]
impl super::Suggester for Suggester {
    async fn labels(&self) -> HashSet<Label> {
        let classifiers = self.classifiers.read().await;

        return classifiers.keys().cloned().collect();
    }

    async fn guess(&self, text: &str) -> Result<HashSet<Label>> {
        let tokens = Self::tokenize(text);

        return Ok(self.suggest(&tokens).await);
    }

    async fn train(&self, text: &str, expected_labels: &HashSet<Label>) -> Result<()> {
        let tokens = Self::tokenize(text);

        // Re-calculate which labels have been proposed before
        let proposed_labels = self.suggest(&tokens).await;

        let mut classifiers = self.classifiers.write().await;

        // Calculate added and removed labels and train classifiers accordingly
        for label in proposed_labels.difference(&expected_labels) {
            let classifier = classifiers.entry(label.clone()).or_insert_with(Classifier::new);
            classifier.train_con(&tokens);
        }

        for label in expected_labels.difference(&proposed_labels) {
            let classifier = classifiers.entry(label.clone()).or_insert_with(Classifier::new);
            classifier.train_pro(&tokens);
        }

        Self::save(&self.path, &classifiers).await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::suggester::Suggester as _;

    use super::*;

    #[tokio::test]
    async fn classify() {
        let tmp = tempfile::tempdir().unwrap();

        let meat = [
            "sirloin meatloaf ham hock sausage meatball tongue prosciutto picanha turkey ball tip pastrami. ribeye chicken sausage, ham hock landjaeger pork belly pancetta ball tip tenderloin leberkas shank shankle rump. cupim short ribs ground round biltong tenderloin ribeye drumstick landjaeger short loin doner chicken shoulder spare ribs fatback boudin. pork chop shank shoulder, t-bone beef ribs drumstick landjaeger meatball.",
            "sirloin porchetta drumstick, pastrami bresaola landjaeger turducken kevin ham capicola corned beef. pork cow capicola, pancetta turkey tri-tip doner ball tip salami. fatback pastrami rump pancetta landjaeger. doner porchetta meatloaf short ribs cow chuck jerky pork chop landjaeger picanha tail.",
        ];
        let vegg = [
            "beetroot water spinach okra water chestnut ricebean pea catsear courgette summer purslane. water spinach arugula pea tatsoi aubergine spring onion bush tomato kale radicchio turnip chicory salsify pea sprouts fava bean. dandelion zucchini burdock yarrow chickpea dandelion sorrel courgette turnip greens tigernut soybean radish artichoke wattle seed endive groundnut broccoli arugula.",
            "pea horseradish azuki bean lettuce avocado asparagus okra. kohlrabi radish okra azuki bean corn fava bean mustard tigernut jã­cama green bean celtuce collard greens avocado quandong fennel gumbo black-eyed pea. grape silver beet watercress potato tigernut corn groundnut. chickweed okra pea winter purslane coriander yarrow sweet pepper radish garlic brussels sprout groundnut summer purslane earthnut pea tomato spring onion azuki bean gourd. gumbo kakadu plum komatsuna black-eyed pea green bean zucchini gourd winter purslane silver beet rock melon radish asparagus spinach.",
        ];

        let suggester = Suggester::from_config(Config {
            path: tmp.path().join("bayesian").display().to_string(),
            certainty: 0.1,
        }).await.unwrap();

        suggester.train(
            &meat[0],
            &vec![Label::from("Meat"), Label::from("Foo")].into_iter().collect(),
        ).await.unwrap();

        assert_eq!(suggester.classifiers.read().await.get("Meat").unwrap()
                       .data.get("sirloin").unwrap(),
                   &Counter::with(1, 0));
        assert_eq!(suggester.classifiers.read().await.get("Foo").unwrap()
                       .data.get("sirloin").unwrap(),
                   &Counter::with(1, 0));
        assert_eq!(suggester.classifiers.read().await.get("Meat").unwrap()
                       .data.get("prosciutto").unwrap(),
                   &Counter::with(1, 0));
        assert_eq!(suggester.classifiers.read().await.get("Foo").unwrap()
                       .data.get("prosciutto").unwrap(),
                   &Counter::with(1, 0));

        suggester.train(
            &meat[1],
            &vec![Label::from("Meat"), Label::from("Bar")].into_iter().collect(),
        ).await.unwrap();

        assert_eq!(suggester.classifiers.read().await.get("Meat").unwrap()
                       .data.get("sirloin").unwrap(),
                   &Counter::with(2, 0));
        assert_eq!(suggester.classifiers.read().await.get("Foo").unwrap()
                       .data.get("sirloin").unwrap(),
                   &Counter::with(1, 1));
        assert_eq!(suggester.classifiers.read().await.get("Bar").unwrap()
                       .data.get("sirloin").unwrap(),
                   &Counter::with(1, 0));
        assert_eq!(suggester.classifiers.read().await.get("Meat").unwrap()
                       .data.get("prosciutto").unwrap(),
                   &Counter::with(1, 0));
        assert_eq!(suggester.classifiers.read().await.get("Foo").unwrap()
                       .data.get("prosciutto").unwrap(),
                   &Counter::with(1, 0));
        assert_eq!(suggester.classifiers.read().await.get("Meat").unwrap()
                       .data.get("bresaola").unwrap(),
                   &Counter::with(1, 0));
        assert_eq!(suggester.classifiers.read().await.get("Foo").unwrap()
                       .data.get("bresaola").unwrap(),
                   &Counter::with(0, 1));
        assert_eq!(suggester.classifiers.read().await.get("Bar").unwrap()
                       .data.get("bresaola").unwrap(),
                   &Counter::with(1, 0));

        suggester.train(
            &vegg[0],
            &vec!["Vegg".into(), "Foo".into()].into_iter().collect(),
        ).await.unwrap();

        suggester.train(
            &vegg[1],
            &vec![Label::from("Vegg"), Label::from("Bar")].into_iter().collect(),
        ).await.unwrap();

        assert_eq!(suggester.classifiers.read().await.get("Meat").unwrap()
                       .classify(&Suggester::tokenize("salami")),
                   1.0f64);
        assert_eq!(suggester.classifiers.read().await.get("Foo").unwrap()
                       .classify(&Suggester::tokenize("salami")),
                   0.0f64);
        assert_eq!(suggester.classifiers.read().await.get("Bar").unwrap()
                       .classify(&Suggester::tokenize("salami")),
                   1.0f64);

        assert_eq!(suggester.guess("salami pancetta beef ribs").await.unwrap(),
                   vec![Label::from("Meat"), Label::from("Foo"), Label::from("Bar")].into_iter().collect());
    }
}
