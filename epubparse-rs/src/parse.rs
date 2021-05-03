use std::io::Read;
use std::{collections::HashMap, io, path::PathBuf};

use io::Cursor;
use regex::Regex;
use xmltree::Element;
use zip::{result::ZipError, ZipArchive};

use crate::{
    errors::{MalformattedEpubError, ParseError},
    types::{Book, Chapter},
    util,
};

struct ZipArchiveWrapper<'a> {
    zip_archive: ZipArchive<Cursor<&'a [u8]>>,
}

pub struct ManifestItem {
    id: String,
    href: String,
    media_type: String,
    properties: Option<String>,
}

pub type ItemId = String;
pub type Manifest = HashMap<ItemId, ManifestItem>;
pub type Spine = Vec<ItemId>;

pub struct ContentOPF {
    pub title: String,
    pub author: Option<String>,
    pub language: String,
    pub manifest: Manifest,
    pub spine: Spine,
}

#[derive(PartialEq, Eq, Hash)]
pub struct NavPoint {
    pub id: String,
    pub label: Option<String>,
    pub play_order: Option<usize>,
    pub level: usize,
    pub src: String,
    pub children: Vec<NavPoint>,
}

struct FlattenedChapter {
    pub title: String,
    pub text: String,
    pub level: u64,
    pub resource_id: ItemId,
}

pub struct TocNcx {
    // maximum of 4 is allowed
    pub depth: usize,
    // ordered list of top-level nav points
    pub nav_points: Vec<NavPoint>,
}

impl TocNcx {
    pub fn get_flattened_nav_points(&self) -> Vec<&NavPoint> {
        let mut result = Vec::new();
        for nav_point in &self.nav_points {
            self.add_dfs(nav_point, &mut result);
        }
        result
    }

    fn add_dfs<'a>(&self, nav_point: &'a NavPoint, result: &mut Vec<&'a NavPoint>) {
        result.push(nav_point);
        for child in &nav_point.children {
            self.add_dfs(child, result);
        }
    }
}

pub struct EpubArchive<'a> {
    zip: ZipArchiveWrapper<'a>,
    manifest_html_files: HashMap<String, String>,
    pub content_opf_dir: PathBuf,
    pub content_opf: ContentOPF,
    pub navigation: TocNcx,
}

impl<'a> ZipArchiveWrapper<'a> {
    fn new(reader: Cursor<&'a [u8]>) -> Result<Self, ZipError> {
        let zip_archive = ZipArchive::new(reader)?;
        Ok(ZipArchiveWrapper { zip_archive })
    }

    fn get_file_content(&mut self, filepath: &str) -> Result<String, ZipError> {
        let mut file = self.zip_archive.by_name(filepath)?;
        let mut buffer = String::new();
        file.read_to_string(&mut buffer)?;
        Ok(buffer)
    }

    fn get_filenames(&self) -> Vec<String> {
        self.zip_archive
            .file_names()
            .into_iter()
            .map(|s| s.to_string())
            .collect()
    }
}

impl<'a> EpubArchive<'a> {
    pub fn new(bytes: &'a [u8]) -> Result<Self, ParseError> {
        let mut zip = ZipArchiveWrapper::new(Cursor::new(bytes))?;
        let container_text = zip.get_file_content("META-INF/container.xml")?;
        // TODO: make this more robust
        let content_opf_re = Regex::new(r#"rootfile full-path="(\S*)""#).unwrap();

        let content_opf_path = match content_opf_re.captures(&container_text) {
            Some(captures) => captures.get(1).unwrap().as_str().to_string(),
            None => {
                return Err(ParseError::EpubError(
                    MalformattedEpubError::MalformattedContainer,
                ));
            }
        };
        let content_opf_dir = match PathBuf::from(&content_opf_path).parent() {
            Some(p) => p.to_path_buf(),
            None => PathBuf::new(),
        };
        let content_opf_text = zip.get_file_content(&content_opf_path)?;
        let content_opf = parse_content_opf(&content_opf_text)
            .ok_or(MalformattedEpubError::MalformattedContentOpf)?;

        let mut nxc_path = content_opf_dir.clone();
        nxc_path.push(
            &content_opf
                .manifest
                .get("ncx")
                .ok_or(MalformattedEpubError::MalformattedContentOpf)?
                .href,
        );
        // TODO: check if this would always work
        let ncx_path = nxc_path.into_os_string().into_string().unwrap();
        // println!("ncx path: {}", &ncx_path);
        let ncx_text = zip.get_file_content(&ncx_path)?;
        let navigation = parse_ncx(&ncx_text)?;

        // construct map filename -> content for all html files declared in manifest
        // let manifest_html_files: HashMap<String, String> = HashMap::new();
        let manifest_html_files: HashMap<String, String> = content_opf
            .manifest
            .values()
            .filter_map(|manifest_item| {
                if manifest_item.media_type == "application/xhtml+xml" {
                    Some(manifest_item.href.to_string())
                } else {
                    None
                }
            })
            .map(|filepath| {
                let mut full_path = content_opf_dir.clone();
                full_path.push(filepath.clone());
                let full_path = full_path.into_os_string().into_string().unwrap();
                zip.get_file_content(&full_path)
                    .map(|content| (filepath, content))
            })
            .collect::<Result<HashMap<_, _>, ZipError>>()?;

        Ok(EpubArchive {
            zip,
            manifest_html_files,
            content_opf_dir,
            content_opf,
            navigation,
        })
    }

    pub fn to_book(&self) -> Result<Book, ParseError> {
        let (preface_content, chapters) = self.item_refs_to_chapters()?;
        Ok(Book {
            title: self.content_opf.title.clone(),
            author: self.content_opf.author.clone(),
            preface_content,
            chapters,
        })
    }

    pub fn get_title(&self) -> &str {
        &self.content_opf.title
    }

    /// find all nav points that have a source that matches the given item's href
    /// returns matches in same order as they appear in given list
    fn get_matching_navpoints(
        &self,
        item_href: &str,
        nav_points: &'a [NavPoint],
    ) -> Vec<&'a NavPoint> {
        let mut result = Vec::new();
        for nav_point in nav_points {
            // nav_point src may have anchor suffix
            if nav_point.src.contains(item_href) {
                result.push(nav_point);
            }
            result.append(&mut self.get_matching_navpoints(item_href, &nav_point.children));
        }
        result
    }

    // TODO: make sure this method can not be called with zero nav_points
    fn item_refs_to_chapters(&self) -> Result<(String, Vec<Chapter>), ParseError> {
        // 1. construct list of navpoints in order they are visited by reader
        // 2. walk through spine and add resource contents to right nav_point
        // 3. convert nested navpoint structure to nested chapter structure

        let flattened_navpoints = self.navigation.get_flattened_nav_points();
        // nav_point -> [src_path1 ... src_pathn], where src_path1 mave have start anchor ("path#anchor")
        let nav_point_sources_map: HashMap<&NavPoint, Vec<&str>> = flattened_navpoints
            .iter()
            .map(|np| (*np, vec![np.src.as_str()]))
            .collect();
        // nav_point -> top level text content (not including nested nav_points' contents)
        let mut nav_point_content_map: HashMap<&NavPoint, Vec<String>> = flattened_navpoints
            .iter()
            .map(|np| (*np, Vec::new()))
            .collect();

        // ordered sources that occur before first nav_point
        let mut preface_sources: Vec<&str> = Vec::new();
        let mut preface_content: Vec<String> = Vec::new();
        let mut passed_preface = false;
        // ordered (source, nav_point) pairs
        //   where source is path with potential anchor
        let mut ordered_sources_navpoints: Vec<(&str, &NavPoint)> =
            Vec::with_capacity(self.content_opf.spine.len());

        let mut last_matched_nav_point = *flattened_navpoints.first().unwrap();
        for item_id in &self.content_opf.spine {
            // convert id to href
            let item_href = self
                .content_opf
                .manifest
                .get(item_id)
                .map(|manifest_item| &manifest_item.href)
                .ok_or(ParseError::EpubError(
                    MalformattedEpubError::MalformattedContentOpf,
                ))?;
            let matching_nav_points =
                self.get_matching_navpoints(item_href, &self.navigation.nav_points);
            // if no matches
            if matching_nav_points.is_empty() {
                // if beyond preface, should match previous nav_point
                // else, matches preface
                if passed_preface {
                    ordered_sources_navpoints.push((item_href, last_matched_nav_point));
                } else {
                    preface_sources.push(item_href);
                }
            } else {
                passed_preface = true;
                // if some matches,
                // append matched nav_points' sources in order
                for matching_nav_point in matching_nav_points {
                    let first_src_of_np = *nav_point_sources_map
                        .get(matching_nav_point)
                        .unwrap()
                        .first()
                        .unwrap();
                    ordered_sources_navpoints.push((first_src_of_np, matching_nav_point));
                    last_matched_nav_point = matching_nav_point;
                }
            }
        }

        // fill in contents for nav_points
        for i in 0..ordered_sources_navpoints.len() {
            let (src_path, nav_point) = ordered_sources_navpoints.get(i).unwrap();
            let next_src_path = ordered_sources_navpoints.get(i + 1).map(|(s, _np)| *s);
            let content_chunk = self.src_to_text(src_path, next_src_path)?;
            nav_point_content_map
                .get_mut(nav_point)
                .unwrap()
                .push(content_chunk);
        }
        // fill in content for preface
        for i in 0..preface_sources.len() {
            let src_path = preface_sources.get(i).unwrap();
            let next_src_path = if i == preface_sources.len() - 1 {
                preface_sources.last().copied()
            } else {
                ordered_sources_navpoints.first().map(|tpl_ref| tpl_ref.0)
            };
            let content_chunk = self.src_to_text(src_path, next_src_path)?;
            preface_content.push(content_chunk);
        }

        let chapters: Vec<Chapter> = self
            .navigation
            .nav_points
            .iter()
            .map(|np| convert_np_to_chapter(np, &nav_point_content_map))
            .collect();
        Ok((preface_content.join("\n"), chapters))
    }

    // get text starting at src (path with potential anchor)
    // end of text is
    // a. end of file given by src (if next_src is different file)
    // b. before anchor in next_src (if next_src is same file)
    fn src_to_text(&self, src: &str, next_src: Option<&str>) -> Result<String, ParseError> {
        let mut src_split = src.split('#');
        let src_file = src_split.next().unwrap();
        let src_anchor = src_split.next();
        let full_text = self.manifest_html_files.get(src_file).ok_or_else(|| {
            MalformattedEpubError::MalformattedTocNcx(format!(
                "File {} in TOC, but not in Manifest",
                src_file
            ))
        })?;
        let stop_anchor = if next_src.map(|s| s.starts_with(src_file)) == Some(true) {
            let mut next_src_split = next_src.unwrap().split('#');
            let _next_src_file = next_src_split.next();
            next_src_split.next()
        } else {
            None
        };
        util::html_to_text(full_text.as_str(), src_anchor, stop_anchor).map_err(|err| {
            ParseError::EpubError(MalformattedEpubError::MalformattedHTML(
                src_file.to_string(),
                err,
            ))
        })
    }
}

// given nav_point must be present in content map
fn convert_np_to_chapter(
    nav_point: &NavPoint,
    contents: &HashMap<&NavPoint, Vec<String>>,
) -> Chapter {
    let subchapters: Vec<Chapter> = nav_point
        .children
        .iter()
        .map(|child| convert_np_to_chapter(child, contents))
        .collect();
    let text = contents
        .get(nav_point)
        .expect("NavPoint should have been present in content map")
        .join("\n");
    Chapter {
        title: nav_point.label.clone().unwrap_or_else(|| "".to_string()),
        text,
        subchapters,
    }
}

fn parse_nav_points(nav_points: &Element, level: usize) -> Option<Vec<NavPoint>> {
    nav_points
        .children
        .iter()
        .filter_map(|node| {
            if let Some(el) = node.as_element() {
                if el.name == "navPoint" {
                    return Some(el);
                }
            }
            None
        })
        .map(|el| {
            let id = el.attributes.get("id")?.to_string();
            let play_order: Option<usize> = el.attributes.get("playOrder").and_then(|po| po.parse().ok());
            let src = el.get_child("content")?.attributes.get("src")?.to_string();
            let label = el
                .get_child("navLabel")
                .and_then(|el| el.get_child("text"))
                .and_then(|el| el.get_text())
                .map(|s| s.to_string());
            let children = parse_nav_points(el, level + 1)?;
            Some(NavPoint {
                id,
                label,
                play_order,
                level,
                src,
                children,
            })
        })
        .collect::<Option<Vec<_>>>()
}

fn parse_ncx(text: &str) -> Result<TocNcx, MalformattedEpubError> {
    let ncx = xmltree::Element::parse(text.as_bytes())
        .map_err(|_e| MalformattedEpubError::MalformattedTocNcx("Invalid XML".to_string()))?;
    let depths: Vec<usize> = ncx
        .get_child("head")
        .ok_or_else(|| MalformattedEpubError::MalformattedTocNcx("Missing head".to_string()))?
        .children
        .iter()
        .filter_map(|node| {
            if let Some(el) = node.as_element() {
                if el.name == "meta"
                    && el.attributes.get("name").map(|s| s.as_str()) == Some("dtb:depth")
                {
                    let depth = el.attributes.get("content")?;
                    return depth.parse().ok();
                }
            }
            None
        })
        .collect();
    let depth = if depths.len() != 1 {
        return Err(MalformattedEpubError::MalformattedTocNcx(
            "Depth info missing or duplicated".to_string(),
        ));
    } else {
        *depths.get(0).unwrap()
    };
    let nav_map = ncx
        .get_child("navMap")
        .ok_or_else(|| MalformattedEpubError::MalformattedTocNcx("Missing navMap".to_string()))?;
    let nav_points = parse_nav_points(nav_map, 1).ok_or_else(|| {
        MalformattedEpubError::MalformattedTocNcx("Could not parse NavPoints".to_string())
    })?;
    Ok(TocNcx { depth, nav_points })
}

fn parse_manifest(manifest: &Element) -> Option<Manifest> {
    Some(
        manifest
            .children
            .iter()
            .filter_map(|node| {
                if let Some(el) = node.as_element() {
                    if el.name == "item" {
                        let id = el.attributes.get("id")?.to_string();
                        return Some((
                            id.clone(),
                            ManifestItem {
                                id,
                                href: el.attributes.get("href")?.to_string(),
                                media_type: el.attributes.get("media-type")?.to_string(),
                                properties: el.attributes.get("properties").map(|s| s.to_string()),
                            },
                        ));
                    }
                }
                None
            })
            .collect::<HashMap<ItemId, ManifestItem>>(),
    )
}

pub fn parse_spine(spine: &Element) -> Option<Spine> {
    Some(
        spine
            .children
            .iter()
            .filter_map(|node| {
                if let Some(el) = node.as_element() {
                    if el.name == "itemref" {
                        let id = el.attributes.get("idref")?.to_string();
                        return Some(id);
                    }
                }
                None
            })
            .collect(),
    )
}

fn parse_content_opf(text: &str) -> Option<ContentOPF> {
    let package = xmltree::Element::parse(text.as_bytes()).ok()?;
    let metadata = package.get_child("metadata")?;
    let manifest = package.get_child("manifest")?;
    let spine = package.get_child("spine")?;
    let title = metadata.get_child("title")?.get_text()?.to_string();
    let author = metadata
        .get_child("creator")
        .map(|el| el.get_text())
        .flatten()
        .map(|s| s.to_string());
    let language = metadata.get_child("language")?.get_text()?.to_string();
    let manifest = parse_manifest(manifest)?;
    let spine = parse_spine(spine)?;
    Some(ContentOPF {
        title,
        author,
        language,
        manifest,
        spine,
    })
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    static EPUB_PAID_OFF: &[u8] = include_bytes!("../test_resources/paid_off.epub");
    static EPUB_SHAKESPEARES: &[u8] = include_bytes!("../test_resources/shakespeares.epub");
    static EPUB_SIMPLE: &[u8] = include_bytes!("../test_resources/simple.epub");

    #[test]
    fn epub_to_contentopf() {
        let epub_archive = EpubArchive::new(EPUB_PAID_OFF).unwrap();
        let content_opf = epub_archive.content_opf;
        assert_eq!("Paid Off", &content_opf.title);
        assert_eq!("Walter J. Coburn", &content_opf.author.unwrap());
        assert_eq!("en", &content_opf.language);
        assert!(!content_opf.manifest.is_empty());
        assert!(!content_opf.spine.is_empty());
    }

    #[test]
    fn epub_to_flat_ncx() {
        let epub_archive = EpubArchive::new(EPUB_PAID_OFF).unwrap();
        let toc_ncx = epub_archive.navigation;
        assert_eq!(1, toc_ncx.depth);
        assert_eq!(14, toc_ncx.nav_points.len());
        assert_eq!(
            (1..15).collect::<Vec<usize>>(),
            toc_ncx
                .nav_points
                .iter()
                .map(|np| np.play_order.unwrap())
                .collect::<Vec<usize>>()
        );
    }

    #[test]
    fn epub_to_nested_ncx() {
        let epub_archive = EpubArchive::new(EPUB_SHAKESPEARES).unwrap();
        let toc_ncx = epub_archive.navigation;
        assert_eq!(3, toc_ncx.depth);
        assert_eq!(
            "ACT I",
            toc_ncx.nav_points[3].children[3].label.as_ref().unwrap()
        );
    }

    #[test]
    fn simple_epub_to_book() {
        let expected_author = "蒲松龄";
        let expected_title = "聊斋志异白话文";
        let expected_chapter_titles = vec![
            "聊斋志异白话文",
            "卷一 考城隍",
            "卷一 耳中人",
            "卷一 尸变",
            "卷一 喷水",
            "卷一 瞳人语",
            "卷一 画壁",
            "卷一 山魈",
            "卷一 咬鬼",
            "卷一 捉狐",
            "卷一 荞中怪",
            "卷一 宅妖",
            "卷一 王六郎",
            "卷一 偷桃",
            "卷一 种梨",
            "卷一 劳山道士",
            "卷一 长清僧",
            "卷一 蛇人",
            "卷一 斫蟒",
            "卷一 犬奸",
        ];
        let expected_chapter2_start = "卷一 考城隍 我姐夫的祖父，名叫宋焘，是本县的廪生";
        let expected_chapter3_start = "卷一 耳中人 谭晋玄，是本县的一名秀才。";
        let expected_chapter4_start = "卷一 尸变 阳信县某老翁";

        let epub_archive = EpubArchive::new(EPUB_SIMPLE).unwrap();
        let book = epub_archive
            .to_book()
            .expect("simple.epub should be parsed to book without error");
        let chapter_titles = book.chapters.iter().map(|ch| &ch.title).collect::<Vec<_>>();
        let chapter2 = book.chapters.get(1).expect("Book should contain chapters");
        let chapter3 = book.chapters.get(2).expect("Book should contain chapters");
        let chapter4 = book.chapters.get(3).expect("Book should contain chapters");
        assert_eq!(Some(expected_author.to_string()), book.author);
        assert_eq!(expected_title, &book.title);
        assert_eq!(expected_chapter_titles, chapter_titles);
        assert_eq!("卷一 考城隍", chapter2.title);
        assert!(chapter2.text.starts_with(expected_chapter2_start));
        assert!(chapter3.text.starts_with(expected_chapter3_start));
        assert!(chapter4.text.starts_with(expected_chapter4_start));
    }
}
