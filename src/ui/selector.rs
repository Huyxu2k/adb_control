#[derive(Debug, Clone)]
pub enum Selector {
    Text(String),

    TextContains(String),

    ResourceId(String),

    Class(String),

    ContentDescription(String),

    Clickable,

    Enabled,

    TextAndClass { text: String, class_name: String },

    ResourceIdAndText { resource_id: String, text: String },
}

impl Selector {
    pub fn matches(&self, node: &UiNode) -> bool {
        match self {
            Self::Text(text) => node.text.as_deref() == Some(text),

            Self::TextContains(text) => node
                .text
                .as_deref()
                .map(|value| value.contains(text))
                .unwrap_or(false),

            Self::ResourceId(id) => node.resource_id.as_deref() == Some(id),

            Self::Class(class) => node.class_name.as_deref() == Some(class),

            Self::ContentDescription(desc) => node.content_desc.as_deref() == Some(desc),

            Self::Clickable => node.clickable,

            Self::Enabled => node.enabled,

            Self::TextAndClass { text, class_name } => {
                node.text.as_deref() == Some(text) && node.class_name.as_deref() == Some(class_name)
            }

            Self::ResourceIdAndText { resource_id, text } => {
                node.resource_id.as_deref() == Some(resource_id)
                    && node.text.as_deref() == Some(text)
            }
        }
    }
}

use super::UiNode;
