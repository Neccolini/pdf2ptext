use anyhow::Error;
use pdf::content::Op;
use pdf::content::TextDrawAdjusted;
use pdf::file::File;
use std::path::Path;
use std::{iter::Peekable, marker::PhantomData};

#[derive(Debug, Clone, PartialEq)]
pub struct TextObject {
    pub text: String,
}
#[derive(Debug, Clone)]
struct TextObjectParser<'src> {
    ops: std::slice::Iter<'src, Op>,
}

fn text_objects(ops: &[Op]) -> impl Iterator<Item = TextObject> + '_ {
    TextObjectParser { ops: ops.iter() }
}

impl<'src> Iterator for TextObjectParser<'src> {
    type Item = TextObject;

    fn next(&mut self) -> Option<Self::Item> {
        let mut last_text = String::new();
        for op in self.ops.by_ref() {
            match op {
                Op::BeginText => {
                    continue;
                }
                Op::TextDraw { text } => {
                    last_text += &text.to_string().unwrap();
                    println!("text: {}", last_text);
                }
                Op::TextDrawAdjusted { array } => {
                    for td in array {
                        if let TextDrawAdjusted::Text(text) = td {
                            last_text += &text.to_string().unwrap();
                        }
                    }
                }
                Op::EndText => {
                    return Some(TextObject { text: last_text });
                }
                _ => {
                    continue;
                }
            }
        }
        None
    }
}

pub fn group_by<I, F, K>(iterator: I, grouper: F) -> impl Iterator<Item = Vec<I::Item>>
where
    I: IntoIterator,
    F: FnMut(&I::Item) -> K,
    K: PartialEq,
{
    GroupBy {
        iter: iterator.into_iter().peekable(),
        grouper,
        _key: PhantomData,
    }
}

struct GroupBy<I: Iterator, F, K> {
    iter: Peekable<I>,
    grouper: F,
    _key: PhantomData<fn() -> K>,
}

impl<I, F, K> Iterator for GroupBy<I, F, K>
where
    I: Iterator,
    F: FnMut(&I::Item) -> K,
    K: PartialEq,
{
    type Item = Vec<I::Item>;

    fn next(&mut self) -> Option<Self::Item> {
        let first_item = self.iter.next()?;
        let key = (self.grouper)(&first_item);

        let mut items = vec![first_item];

        while let Some(peek) = self.iter.peek() {
            if (self.grouper)(peek) != key {
                break;
            }

            items.push(
                self.iter
                    .next()
                    .expect("Peek guarantees there is another item"),
            );
        }

        Some(items)
    }
}

pub fn parse(file: File<Vec<u8>>) -> Result<String, Error> {
    let mut result = String::new();
    for page in file.pages() {
        let page = page?;
        if let Some(content) = &page.contents {
            let ops = content.operations(&file)?;
            let text = text_objects(&ops);
            let texts = group_by(text, |t| t.text.clone());
            for page in texts {
                for row in page {
                    result += &row.text;
                }
            }
        }
    }
    Ok(result)
}

pub fn read_pdf(path: &Path) -> String {
    let file = File::<Vec<u8>>::open(path).unwrap();
    parse(file).unwrap()
}
