use lopdf::{Document, Object};

/// Parsed PDF structure tree (Tagged PDF).
/// This provides semantic information about document structure
/// when the PDF has been properly tagged.
#[derive(Debug, Clone)]
pub struct StructureTree {
    pub root: Option<StructureNode>,
}

#[derive(Debug, Clone)]
pub struct StructureNode {
    pub struct_type: String,
    pub children: Vec<StructureNode>,
    pub page: Option<usize>,
    pub content_ids: Vec<u32>,
}

impl StructureTree {
    /// Parse the structure tree from a PDF document.
    pub fn parse(doc: &Document) -> Option<Self> {
        // Get the StructTreeRoot from the catalog
        let catalog_id = doc.trailer.get(b"Root").ok()?;
        let catalog = match catalog_id {
            Object::Reference(id) => doc.get_object(*id).ok()?.as_dict().ok()?,
            Object::Dictionary(d) => d,
            _ => return None,
        };

        let struct_tree_root = catalog.get(b"StructTreeRoot").ok()?;
        let struct_dict = match struct_tree_root {
            Object::Reference(id) => doc.get_object(*id).ok()?.as_dict().ok()?,
            Object::Dictionary(d) => d,
            _ => return None,
        };

        let root = parse_node(doc, struct_dict);

        Some(StructureTree { root })
    }
}

fn parse_node(doc: &Document, dict: &lopdf::Dictionary) -> Option<StructureNode> {
    let struct_type = dict
        .get(b"S")
        .ok()
        .and_then(|o| o.as_name().ok())
        .map(|n| String::from_utf8_lossy(n).to_string())
        .unwrap_or_default();

    let mut node = StructureNode {
        struct_type,
        children: Vec::new(),
        page: None,
        content_ids: Vec::new(),
    };

    // Parse children (K entry)
    if let Ok(k) = dict.get(b"K") {
        match k {
            Object::Array(arr) => {
                for child in arr {
                    if let Some(child_node) = parse_child(doc, child) {
                        node.children.push(child_node);
                    }
                }
            }
            Object::Reference(id) => {
                if let Ok(obj) = doc.get_object(*id) {
                    if let Ok(child_dict) = obj.as_dict() {
                        if let Some(child_node) = parse_node(doc, child_dict) {
                            node.children.push(child_node);
                        }
                    }
                }
            }
            Object::Integer(n) => {
                node.content_ids.push(*n as u32);
            }
            _ => {}
        }
    }

    Some(node)
}

fn parse_child(doc: &Document, obj: &Object) -> Option<StructureNode> {
    match obj {
        Object::Reference(id) => {
            let obj = doc.get_object(*id).ok()?;
            let dict = obj.as_dict().ok()?;
            parse_node(doc, dict)
        }
        Object::Dictionary(dict) => parse_node(doc, dict),
        Object::Integer(n) => Some(StructureNode {
            struct_type: String::new(),
            children: Vec::new(),
            page: None,
            content_ids: vec![*n as u32],
        }),
        _ => None,
    }
}
