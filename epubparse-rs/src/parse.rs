use std::{
    collections::HashMap,
    io::{self, Read, Seek},
    path::PathBuf,
};

use io::Cursor;
use regex::Regex;
use xmltree::Element;
use zip::ZipArchive;

use crate::{errors::{MalformattedEpubError, ParseError}, types::{Book, Chapter}};

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

pub struct NavPoint {
    pub id: String,
    pub play_order: Option<usize>,
    pub level: usize,
    pub src: String,
    pub children: Vec<NavPoint>,
}

pub struct TocNcx {
    // maximum of 4 is allowed
    pub depth: usize,
    // ordered list of top-level nav points
    pub nav_points: Vec<NavPoint>,
}

pub struct EpubArchive<R: Read + Seek> {
    zip: ZipArchiveWrapper<R>,
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

    fn get_filenames(&self) -> Vec<String> {
        self.0
            .file_names()
            .into_iter()
            .map(|s| s.to_string())
            .collect()
    }
}

impl<R: Read + Seek> EpubArchive<R> {
    pub fn new(zip_reader: R) -> Result<Self, ParseError> {
        let mut zip = ZipArchiveWrapper(zip::ZipArchive::new(zip_reader)?);
        let filenames = zip.get_filenames();
        let container_text = zip.get_file_content("META-INF/container.xml")?;
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

    pub fn from_bytes(bytes: &[u8]) -> Result<EpubArchive<Cursor<&[u8]>>, ParseError> {
        let cursor = Cursor::new(bytes);
        EpubArchive::new(cursor)
    }

    pub fn to_book(&self) -> Book {
        let chapters: Vec<Chapter> = self
            .zip
            .get_filenames()
            .into_iter()
            .map(|fname| Chapter {
                title: fname,
                content: "".to_string(),
            })
            .collect();
        Book {
            title: "".to_string(),
            author: "".to_string(),
            chapters,
        }
    }

    pub fn get_title(&self) -> &str {
        &self.content_opf.title
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
            let children = parse_nav_points(el, level + 1)?;
            Some(NavPoint {
                id,
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

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    #[test]
    fn epub_to_contentopf() {
        let epub_bytes = fs::read("test_resources/paid_off.epub").unwrap();
        let epub_archive = EpubArchive::<Cursor<&[u8]>>::from_bytes(&epub_bytes).unwrap();
        let content_opf = epub_archive.content_opf;
        assert_eq!("Paid Off", &content_opf.title);
        assert_eq!("Walter J. Coburn", &content_opf.author.unwrap());
        assert_eq!("en", &content_opf.language);
        assert!(!content_opf.manifest.is_empty());
        assert!(!content_opf.spine.is_empty());
    }
}
