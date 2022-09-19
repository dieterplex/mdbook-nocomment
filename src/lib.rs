//! A simple [mdbook](https://rust-lang.github.io/mdBook/index.html) preprocessors that clean up html comments.

use anyhow::Result;
use itertools::Itertools;
use log;
use mdbook::{
    book::Book,
    preprocess::{Preprocessor, PreprocessorContext},
    BookItem,
};
use pulldown_cmark::{Event, Parser};
use pulldown_cmark_to_cmark::cmark;

pub struct NoCommentPreprocessor;

impl Preprocessor for NoCommentPreprocessor {
    fn name(&self) -> &str {
        "nocomment-preprocessor"
    }

    fn run(&self, _ctx: &PreprocessorContext, mut book: Book) -> Result<Book> {
        book.for_each_mut(|item: &mut BookItem| {
            if let BookItem::Chapter(ref mut chapter) = *item {
                let content_events =
                    Parser::new_ext(&chapter.content, pulldown_cmark::Options::empty());
                let events = remove_comment(content_events);
                let mut buf = String::with_capacity(chapter.content.len());
                cmark(events, &mut buf).unwrap();
                chapter.content = buf;
            }
        });
        Ok(book)
    }

    fn supports_renderer(&self, renderer: &str) -> bool {
        renderer != "not-supported"
    }
}

fn remove_comment<'a>(events: impl Iterator<Item = Event<'a>>) -> impl Iterator<Item = Event<'a>> {
    const COMMENT_START: &str = "<!--";
    const COMMENT_END: &str = "-->";
    let mut filtered = vec![];
    let mut mp = events.multipeek();
    loop {
        let current_event = match mp.next() {
            Some(e) => e,
            None => break,
        };
        match current_event {
            Event::Text(ref t1) if t1.as_ref().eq("<") => {
                let next = mp.peek();
                match next {
                    Some(Event::Text(ref t2)) if t2.starts_with("!--") => {
                        let mut removal = t1.to_string();
                        removal.push_str(t2);
                        // Ended at current event
                        if t2.trim_end().ends_with(COMMENT_END) {
                            mp.next();
                            log::debug!("Comment: {}", removal);
                            continue;
                        }
                        // Peek text event for COMMENT_END
                        let mut found = false;
                        let mut count = 0;
                        loop {
                            let nn = mp.peek();
                            match nn {
                                Some(Event::Text(ref c)) => {
                                    removal.push_str(c);
                                    count += 1;
                                    if c.trim_end().ends_with(COMMENT_END) {
                                        found = true;
                                        break;
                                    }
                                }
                                None => break,
                                // May across paragraph
                                _ => {
                                    count += 1;
                                    continue;
                                }
                            }
                        }
                        if found {
                            // Skip comment events
                            for _ in 0..=count {
                                mp.next();
                            }
                            log::debug!("Comment: {}", removal);
                        } else {
                            filtered.push(current_event)
                        }
                    }
                    _ => filtered.push(current_event),
                };
            }
            Event::Html(ref html) if html.starts_with(COMMENT_START) => {
                if html.trim_end().ends_with(COMMENT_END) {
                    // Ended at current event
                    continue;
                }
                let mut removal = vec![html.to_string()];
                let mut found = false;
                let mut cnt = 0;
                loop {
                    let next = mp.peek();
                    match next {
                        Some(Event::Html(ref h)) => {
                            removal.push(h.to_string());
                            cnt += 1;
                            if h.trim_end().ends_with(COMMENT_END) {
                                found = true;
                                for _ in 0..cnt {
                                    mp.next();
                                }
                                log::debug!("{}", removal.join("\n"));
                                continue;
                            }
                        }
                        _ => break,
                    }
                }
                if !found {
                    filtered.push(current_event)
                }
            }
            // Not a comment event, push it as is.
            _ => filtered.push(current_event),
        }
    }
    filtered.into_iter()
}

#[cfg(test)]
mod test {
    #[test]
    fn remove_comments() {
        // oneline comment (one Html event)
        assert_comment_removal("<!-- double-hyphen -->");

        // oneline invalid comment (one Html event)
        assert_comment_removal("<!-- --double-hyphen -->");

        // multiline invalid comment (multi html events)
        assert_comment_removal(
            "<!-- \n\
            --double-hyphen \n\
            -->\n",
        );

        // oneline comment in a paragraph (one Html event)
        assert_comment_removal("text <!-- double-hyphen -->");

        // oneline invalid comment in paragraph (multi Text event)
        assert_comment_removal("text <!-- --double-hyphen -->");

        // multiline invalid comment in a paragraph (multi Text event)
        assert_comment_removal(
            "text <!-- \n\
            --double-hyphen \n\
            \n-->",
        );

        // multiline invalid comment across multi paragraph (multi Text event)
        assert_comment_removal(
            "text <!-- \n\n\
            --double-hyphen \n\n\
            \n-->",
        );
    }

    fn assert_comment_removal(s: &str) {
        let parser = mdbook::utils::new_cmark_parser(s, false);

        let events = crate::remove_comment(parser);
        let mut buf = String::new();
        pulldown_cmark::html::push_html(&mut buf, events);

        log::debug!("RENDERED: {buf}");
        assert!(!buf.contains("double-hyphen"));
        assert!(!buf.contains("--"));
    }
}