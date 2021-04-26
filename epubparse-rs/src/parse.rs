use std::{
    collections::HashMap,
    io::{self, Read, Seek},
    path::PathBuf,
};

use io::Cursor;
use regex::Regex;
use xmltree::Element;
use zip::ZipArchive;

use crate::{
    errors::{MalformattedEpubError, ParseError},
    types::{Book, Chapter},
};

struct ZipArchiveWrapper<R: Read + Seek>(ZipArchive<R>);

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
    zip: ZipArchiveWrapper<Cursor<&'a [u8]>>,
    filenames: Vec<String>,
    pub content_opf_dir: PathBuf,
    pub content_opf: ContentOPF,
    pub navigation: TocNcx,
}

impl<R: Read + Seek> ZipArchiveWrapper<R> {
    fn get_file_content(&mut self, filepath: &str) -> Result<String, ParseError> {
        let mut file = self.0.by_name(filepath)?;
        let mut buffer = String::new();
        file.read_to_string(&mut buffer)?;
        Ok(buffer)
    }

    fn get_html_file_content(&mut self, full_uri: &str, stop_anchor: Option<&str>) -> Result<String, ParseError> {
        let uri_split = full_uri.split("#").collect::<Vec<&str>>();
        let filepath = *uri_split.get(0).unwrap();
        let start_anchor = uri_split.get(1);
        let file_content = self.get_file_content(filepath)?;
        unimplemented!()
    }

    fn get_filenames(&self) -> Vec<String> {
        self.0
            .file_names()
            .into_iter()
            .map(|s| s.to_string())
            .collect()
    }
}

impl<'a> EpubArchive<'a> {
    pub fn new(bytes: &'a [u8]) -> Result<Self, ParseError> {
        let mut zip = ZipArchiveWrapper(zip::ZipArchive::new(Cursor::new(bytes))?);
        let filenames = zip.get_filenames();
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
        println!("ncx path: {}", &ncx_path);
        let ncx_text = zip.get_file_content(&ncx_path)?;
        let navigation = parse_ncx(&ncx_text).ok_or(MalformattedEpubError::MalformattedTocNcx)?;
        Ok(EpubArchive {
            zip,
            filenames,
            content_opf_dir,
            content_opf,
            navigation,
        })
    }

    // TODO: implement correctly
    pub fn to_book(&self) -> Book {
        let chapters = self.item_refs_to_chapters();
        Book {
            title: self.content_opf.title.clone(),
            author: "".to_string(),
            chapters,
        }
    }

    pub fn get_title(&self) -> &str {
        &self.content_opf.title
    }

    /// find all nav points that have a source that matches the given item's href
    fn get_matching_navpoints(
        &self,
        item_id: &ItemId,
        nav_points: &'a [NavPoint],
    ) -> Vec<&'a NavPoint> {
        let mut result = Vec::new();
        if let Some(item_href) = self
            .content_opf
            .manifest
            .get(item_id)
            .map(|manifest_item| &manifest_item.href)
        {
            for nav_point in nav_points {
                // nav_point src may have anchor suffix
                if nav_point.src.contains(item_href) {
                    result.push(nav_point);
                }
                result.append(&mut self.get_matching_navpoints(item_id, &nav_point.children));
            }
        }
        result
    }

    fn item_refs_to_chapters(&self) -> Vec<Chapter> {
        let flattened_navpoints = self.navigation.get_flattened_nav_points();
        // nav_point -> [src_path1 ... src_pathn], where src pathes can have anchors ("path#anchor")
        let mut nav_point_sources_map: HashMap<&NavPoint, Vec<&str>> = flattened_navpoints
            .iter()
            .map(|np| (*np, Vec::new()))
            .collect();
        let mut preface_sources: Vec<&str> = Vec::new();

        for item_id in &self.content_opf.spine {
            let matching_nav_points =
                self.get_matching_navpoints(item_id, &self.navigation.nav_points);
            if matching_nav_points.is_empty() {
                preface_sources.push(item_id);
            }
            for matching_nav_point in matching_nav_points {
                nav_point_sources_map
                    .get_mut(matching_nav_point)
                    .unwrap()
                    .push(item_id);
            }
        }

        unimplemented!()
    }
}

pub fn parse_nav_points(nav_points: &Element, level: usize) -> Option<Vec<NavPoint>> {
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
            let play_order: Option<usize> = el.attributes.get("playOrder")?.parse().ok();
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

pub fn parse_ncx(text: &str) -> Option<TocNcx> {
    let ncx = xmltree::Element::parse(text.as_bytes()).ok()?;
    let depths: Vec<usize> = ncx
        .get_child("head")?
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
        return None;
    } else {
        *depths.get(0).unwrap()
    };
    let nav_map = ncx.get_child("navMap")?;
    let nav_points = parse_nav_points(nav_map, 1)?;
    Some(TocNcx { depth, nav_points })
}

pub fn parse_manifest(manifest: &Element) -> Option<Manifest> {
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

pub fn parse_content_opf(text: &str) -> Option<ContentOPF> {
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

pub fn convert_archive_to_book(epub_archive: &EpubArchive) -> Result<Book, ParseError> {
    let title = epub_archive.content_opf.title.clone();
    let author = epub_archive
        .content_opf
        .author
        .as_ref()
        .unwrap_or(&"".to_string())
        .to_string();
    let spine = &epub_archive.content_opf.spine;
    let mut unconnected_chapters: Vec<Chapter> = Vec::new();
    let mut index = 0;
    for item_id in spine {
        let item = epub_archive
            .content_opf
            .manifest
            .get(item_id)
            .ok_or(ParseError::EpubError(
                MalformattedEpubError::MalformattedManifest,
            ))?;
    }
    let chapters: Vec<Chapter> = epub_archive
        .navigation
        .nav_points
        .iter()
        .map(|np| nav_point_to_chapter(np, epub_archive))
        .collect();
    Ok(Book {
        title,
        author,
        chapters,
    })
}

pub fn nav_point_to_chapter(nav_point: &NavPoint, archive: &EpubArchive) -> Chapter {
    unimplemented!()
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    #[test]
    fn epub_to_contentopf() {
        let epub_bytes = fs::read("test_resources/paid_off.epub").unwrap();
        let epub_archive = EpubArchive::new(&epub_bytes).unwrap();
        let content_opf = epub_archive.content_opf;
        assert_eq!("Paid Off", &content_opf.title);
        assert_eq!("Walter J. Coburn", &content_opf.author.unwrap());
        assert_eq!("en", &content_opf.language);
        assert!(!content_opf.manifest.is_empty());
        assert!(!content_opf.spine.is_empty());
    }

    #[test]
    fn epub_to_flat_ncx() {
        let epub_bytes = fs::read("test_resources/paid_off.epub").unwrap();
        let epub_archive = EpubArchive::new(&epub_bytes).unwrap();
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
        let epub_bytes = fs::read("test_resources/shakespeares.epub").unwrap();
        let epub_archive = EpubArchive::new(&epub_bytes).unwrap();
        let toc_ncx = epub_archive.navigation;
        assert_eq!(3, toc_ncx.depth);
        assert_eq!(
            "ACT I",
            toc_ncx.nav_points[3].children[3].label.as_ref().unwrap()
        );
    }
}
