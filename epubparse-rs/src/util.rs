use std::collections::VecDeque;

// TODO: fix spacing for inline tags such as <i> <b> etc.
pub fn html_to_text(
    full_text: &str,
    start_anchor: Option<&str>,
    stop_anchor: Option<&str>,
) -> Result<String, xmltree::ParseError> {
    let mut text: Vec<String> = Vec::new();
    let root = xmltree::XMLNode::Element(xmltree::Element::parse(full_text.as_bytes())?);
    let mut to_visit: VecDeque<&xmltree::XMLNode> = VecDeque::new();
    to_visit.push_back(&root);
    // do DFS for start node, visiting all nodes before on the way
    while let Some(start) = start_anchor {
        if let Some(xmltree::XMLNode::Element(element)) = to_visit.pop_front() {
            for child in element.children.iter().rev() {
                to_visit.push_front(child);
            }
            if get_named_anchor(element) == Some(start) {
                break;
            }
        }
    }
    // do DFS for all nodes after start until encountering potential stop
    // save all text that is encountered on the way
    while let Some(next_node) = to_visit.pop_front() {
        match next_node {
            xmltree::XMLNode::Element(element) => {
                // if stop anchor encountered, stop
                // println!("{}", element.name);
                if let (Some(stop), Some(name_attr)) = (stop_anchor, get_named_anchor(element)) {
                    if stop == name_attr {
                        break;
                    }
                }
                for child in element.children.iter().rev() {
                    to_visit.push_front(child);
                }
            }
            xmltree::XMLNode::Text(s) => {
                // println!("{}", s);
                text.push(s.trim().to_string());
            }
            _ => {}
        }
    }
    Ok(text.join(" "))
}

#[allow(dead_code)]
pub fn get_all_text(xml_node: &xmltree::XMLNode) -> String {
    let mut text: Vec<String> = Vec::new();
    match xml_node {
        xmltree::XMLNode::Element(element) => {
            for child in &element.children {
                let s = get_all_text(child);
                if !s.is_empty() {
                    text.push(s);
                }
            }
        }
        xmltree::XMLNode::Text(s) => {
            return s.trim().to_string();
        }
        _ => {}
    }
    text.join(" ")
}

pub fn get_named_anchor(element: &xmltree::Element) -> Option<&str> {
    let mut result = element.attributes.get("id").map(String::as_str);
    if let ("a", Some(name_attr)) = (element.name.as_str(), element.attributes.get("name")) {
        result = Some(name_attr.as_str());
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    static PRIDE_PREJUDICE_CHAPTER_HTML: &str =
        include_str!("../../test_resources/pride_prejudice_chapter.html");
    static SIMPLE_CHAPTER_HTML: &str = include_str!("../../test_resources/simple_chapter.html");

    #[test]
    fn get_all_text_simple() {
        let root = xmltree::Element::parse(PRIDE_PREJUDICE_CHAPTER_HTML.as_bytes()).unwrap();
        let all_text = get_all_text(&xmltree::XMLNode::Element(root));
        // assert_eq!(all_text, "lala");
        assert!(all_text
            .starts_with("The Project Gutenberg eBook of Pride and Prejudice, by Jane Austen"));
        assert!(all_text.contains("“I would not be so fastidious as you are,” cried Mr. Bingley, "));
        assert!(all_text.contains("You are dancing with the only handsome girl in the room,” said Mr. Darcy, looking at the eldest Miss Bennet."));
        assert!(all_text.contains("and the Boulanger —” “If he had had any compassion for me ,” cried her husband impatiently,"));
    }

    #[test]
    fn html_to_text_no_anchors() {
        let all_text = html_to_text(PRIDE_PREJUDICE_CHAPTER_HTML, None, None).unwrap();

        // assert_eq!(all_text, "lala");
        assert!(all_text
            .starts_with("The Project Gutenberg eBook of Pride and Prejudice, by Jane Austen"));
        assert!(all_text.contains("“I would not be so fastidious as you are,” cried Mr. Bingley, "));
        assert!(all_text.contains("You are dancing with the only handsome girl in the room,” said Mr. Darcy, looking at the eldest Miss Bennet."));
        assert!(all_text.contains("and the Boulanger —” “If he had had any compassion for me ,” cried her husband impatiently,"));
    }

    #[test]
    fn html_to_text_with_anchors_simple() {
        let all_text =
            html_to_text(PRIDE_PREJUDICE_CHAPTER_HTML, Some("start"), Some("end")).unwrap();
        assert!(all_text
            .starts_with("Mr. Bingley had soon made himself acquainted with all the principal people in the room;"));
        assert!(all_text.ends_with("and the Boulanger —”"));
    }

    #[test]
    fn html_to_text_with_start_anchor_only_pp() {
        let all_text = html_to_text(PRIDE_PREJUDICE_CHAPTER_HTML, Some("start"), None).unwrap();
        assert!(all_text
            .starts_with("Mr. Bingley had soon made himself acquainted with all the principal people in the room;"));
        assert!(all_text.ends_with("I quite detest the man.”"));
    }

    #[test]
    fn html_to_text_with_start_anchor_only_simple() {
        let all_text = html_to_text(SIMPLE_CHAPTER_HTML, Some("卷一-考城隍"), None).unwrap();
        assert!(all_text.starts_with("卷一 考城隍 我姐夫的祖父，名叫宋焘，是本县的廪生。"));
        assert!(all_text.ends_with("这里的记载只是个大概而已。"));
    }

    #[test]
    fn html_to_text_with_stop_anchor_only() {
        let all_text = html_to_text(PRIDE_PREJUDICE_CHAPTER_HTML, None, Some("end")).unwrap();
        assert!(all_text
            .starts_with("The Project Gutenberg eBook of Pride and Prejudice, by Jane Austen"));
        assert!(all_text.ends_with("and the Boulanger —”"));
    }

    #[test]
    fn html_to_text_get_all_text_equal() {
        let root = xmltree::Element::parse(PRIDE_PREJUDICE_CHAPTER_HTML.as_bytes()).unwrap();
        let all_text1 = get_all_text(&xmltree::XMLNode::Element(root));
        let all_text2 = html_to_text(PRIDE_PREJUDICE_CHAPTER_HTML, None, None).unwrap();
        assert_eq!(all_text1, all_text2);
    }
}
