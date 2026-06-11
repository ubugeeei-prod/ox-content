#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ParsedAttribute {
    pub(super) name: String,
    pub(super) value: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ParsedStartTag {
    pub(super) name: String,
    attributes: Vec<ParsedAttribute>,
}

impl ParsedStartTag {
    pub(super) fn parse(raw: &str) -> Option<Self> {
        if !raw.starts_with('<') || !raw.ends_with('>') || raw.starts_with("</") {
            return None;
        }

        let inner = &raw[1..raw.len() - 1];
        let bytes = inner.as_bytes();
        let mut index = 0;

        while index < bytes.len() && bytes[index].is_ascii_whitespace() {
            index += 1;
        }

        let name_start = index;
        while index < bytes.len() && !bytes[index].is_ascii_whitespace() && bytes[index] != b'/' {
            index += 1;
        }

        if name_start == index {
            return None;
        }

        let name = inner[name_start..index].to_string();
        let mut attributes = Vec::new();

        while index < bytes.len() {
            while index < bytes.len() && bytes[index].is_ascii_whitespace() {
                index += 1;
            }

            if index >= bytes.len() || bytes[index] == b'/' {
                break;
            }

            let attr_start = index;
            while index < bytes.len()
                && !bytes[index].is_ascii_whitespace()
                && bytes[index] != b'='
                && bytes[index] != b'/'
            {
                index += 1;
            }

            if attr_start == index {
                break;
            }

            let attr_name = inner[attr_start..index].to_string();

            while index < bytes.len() && bytes[index].is_ascii_whitespace() {
                index += 1;
            }

            let value = if index < bytes.len() && bytes[index] == b'=' {
                index += 1;
                while index < bytes.len() && bytes[index].is_ascii_whitespace() {
                    index += 1;
                }

                if index >= bytes.len() {
                    Some(String::new())
                } else if bytes[index] == b'"' || bytes[index] == b'\'' {
                    let quote = bytes[index];
                    index += 1;
                    let value_start = index;

                    while index < bytes.len() && bytes[index] != quote {
                        index += 1;
                    }

                    let value = inner[value_start..index].to_string();
                    if index < bytes.len() {
                        index += 1;
                    }
                    Some(value)
                } else {
                    let value_start = index;
                    while index < bytes.len()
                        && !bytes[index].is_ascii_whitespace()
                        && bytes[index] != b'/'
                    {
                        index += 1;
                    }
                    Some(inner[value_start..index].to_string())
                }
            } else {
                None
            };

            attributes.push(ParsedAttribute { name: attr_name, value });
        }

        Some(Self { name, attributes })
    }

    pub(super) fn to_html(&self) -> String {
        let mut html = String::new();
        html.push('<');
        html.push_str(&self.name);

        for attribute in &self.attributes {
            html.push(' ');
            html.push_str(&attribute.name);
            if let Some(value) = &attribute.value {
                html.push_str("=\"");
                html.push_str(value);
                html.push('"');
            }
        }

        html.push('>');
        html
    }

    pub(super) fn set_attribute(&mut self, name: &str, value: &str) {
        if let Some(attribute) = self.attributes.iter_mut().find(|attribute| attribute.name == name)
        {
            attribute.value = Some(value.to_string());
            return;
        }

        self.attributes
            .push(ParsedAttribute { name: name.to_string(), value: Some(value.to_string()) });
    }

    pub(super) fn class_names(&self) -> Vec<String> {
        self.attribute_value("class")
            .map(|value| value.split_whitespace().map(ToString::to_string).collect())
            .unwrap_or_default()
    }

    pub(super) fn merge_class_names(&mut self, class_names: &[String]) {
        if class_names.is_empty() {
            return;
        }

        let mut merged = self.class_names();
        for class_name in class_names {
            if !merged.contains(class_name) {
                merged.push(class_name.clone());
            }
        }
        self.set_class_names(&merged);
    }

    pub(super) fn set_class_names(&mut self, class_names: &[String]) {
        let value = class_names.join(" ");
        self.set_attribute("class", &value);
    }

    pub(super) fn data_attributes(&self) -> Vec<ParsedAttribute> {
        self.attributes
            .iter()
            .filter(|attribute| attribute.name.starts_with("data-"))
            .cloned()
            .collect()
    }

    pub(super) fn merge_data_attributes(&mut self, attributes: &[ParsedAttribute]) {
        for attribute in attributes {
            match &attribute.value {
                Some(value) => self.set_attribute(&attribute.name, value),
                None => {
                    if self.attributes.iter().all(|existing| existing.name != attribute.name) {
                        self.attributes.push(attribute.clone());
                    }
                }
            }
        }
    }

    fn attribute_value(&self, name: &str) -> Option<&str> {
        self.attributes.iter().find_map(|attribute| {
            if attribute.name == name {
                attribute.value.as_deref()
            } else {
                None
            }
        })
    }
}
