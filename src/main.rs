use ego_tree::NodeRef;
use scraper::{node::Text, ElementRef, Node};

enum ElementType<'a> {
    String(&'a Text),
    Span(Span<'a>),
    Anchor(Anchor<'a>),
    Paragraph(Paragraph<'a>),
    Ruby(Ruby<'a>),
}

struct Ruby<'a> {
    node: NodeRef<'a, Node>,
}
struct Anchor<'a> {
    node: NodeRef<'a, Node>,
}
struct Span<'a> {
    node: NodeRef<'a, Node>,
}
struct Paragraph<'a> {
    node: NodeRef<'a, Node>,
}

fn main() {
    let response = reqwest::blocking::get(
        "https://www3.nhk.or.jp/news/easy/k10014254911000/k10014254911000.html",
    )
    .unwrap()
    .text()
    .unwrap();
    let document = scraper::Html::parse_document(&response);

    let article_selector = scraper::Selector::parse("article.article-main").unwrap();
    let title_selector = scraper::Selector::parse("h1.article-main__title").unwrap();
    let article_date_selector = scraper::Selector::parse("js-article-date").unwrap();
    let article_body_selector = scraper::Selector::parse("#js-article-body").unwrap();

    let article = document.select(&article_selector).next().unwrap();
    let title = article.select(&title_selector).next().unwrap();
    let article_body = article.select(&article_body_selector).next().unwrap();

    let title_text = title.text().collect::<Vec<_>>().join(";");

    let title_snippets = parse_japanese_content(title);
    print_nhk_text(&title_snippets);

    let article_body_text = article_body.text().collect::<Vec<_>>().join(";");
    let article_snippets = parse_article_body(article_body);
    print_nhk_text(&article_snippets);
}

fn classify_node(node: NodeRef<'_, Node>) -> Option<ElementType> {
    if let Some(text_node) = node.value().as_text() {
        return Some(ElementType::String(text_node));
    }

    let Some(element) = node.value().as_element() else {
        panic!("Not implemented element type");
    };

    return match element.name() {
        "p" => Some(ElementType::Paragraph(Paragraph { node: node })),
        "a" => Some(ElementType::Anchor(Anchor { node: node })),
        "span" => Some(ElementType::Span(Span { node: node })),
        "ruby" => Some(ElementType::Ruby(Ruby { node: node })),
        name => None, 
    };
}

fn handle_classified_element(element: ElementType) -> Vec<NhkText> {
    return match element {
        ElementType::String(t) => handle_text(t),
        ElementType::Span(s) => handle_span(s),
        ElementType::Anchor(a) => handle_anchor(a),
        ElementType::Paragraph(p) => handle_paragraph(p),
        ElementType::Ruby(r) => handle_ruby(r),
    };
}

fn parse_article_body(element: ElementRef<'_>) -> Vec<NhkText> {
    element
        .children()
        .map(|c| classify_node(c))
        .flatten()
        .flat_map(|n| handle_classified_element(n))
        .collect()
}

fn handle_paragraph(element: Paragraph) -> Vec<NhkText> {
    element
        .node
        .children()
        .map(classify_node)
        .flatten()
        .flat_map(handle_classified_element)
        .collect()
}

fn handle_anchor(element: Anchor) -> Vec<NhkText> {
    element
        .node
        .children()
        .map(classify_node)
        .flatten()
        .flat_map(handle_classified_element)
        .collect()
}

fn handle_span(element: Span) -> Vec<NhkText> {
    element
        .node
        .children()
        .map(classify_node)
        .flatten()
        .flat_map(handle_classified_element)
        .collect()
}

fn parse_japanese_content(element: ElementRef<'_>) -> Vec<NhkText> {
    return Vec::new();
    // element.children().map(map_ruby).collect()
}

fn print_nhk_text(text: &[NhkText]) {
    let s = text
        .iter()
        .map(|t| t.to_string())
        .collect::<Vec<_>>()
        .join("");
    println!("{}", s);
}

fn handle_text<'a>(element: &'a Text) -> Vec<NhkText> {
    vec![NhkText {
        content: element.to_string(),
        superscript: None,
    }]
}

struct NhkText {
    content: String,
    superscript: Option<String>,
}

impl NhkText {
    fn to_string(&self) -> String {
        if let Some(furigana) = &self.superscript {
            return format!("{}[{}]", self.content, furigana);
        }

        self.content.clone()
    }
}

fn handle_ruby(element: Ruby) -> Vec<NhkText> {
    let child_value = element.node.first_child().unwrap().value();

    let text = child_value
        .as_text()
        .or_else(|| {
            element
                .node
                .first_child()
                .unwrap()
                .first_child()
                .unwrap()
                .value()
                .as_text()
        })
        .unwrap();

    let furigana = element
        .node
        .children()
        .filter(|c| c.value().as_element().is_some_and(|e| e.name() == "rt"))
        .map(|c| {
            c.first_child()
                .unwrap()
                .value()
                .as_text()
                .unwrap()
                .to_string()
        })
        .collect::<Vec<_>>()
        .first()
        .unwrap()
        .clone();

    let snippets = NhkText {
        content: text.to_string(),
        superscript: Some(furigana),
    };

    vec![snippets]
}
