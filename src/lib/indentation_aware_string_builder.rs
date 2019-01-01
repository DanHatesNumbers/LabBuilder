pub struct IndentationAwareStringBuilder {
    indentation_type: IndentationType,
    tab_size: Option<usize>,
    current_indentation_level: usize,
    buffer: Vec<String>,
}

#[allow(dead_code)]
pub enum IndentationType {
    Tabs,
    Spaces,
}

impl IndentationAwareStringBuilder {
    pub fn new() -> IndentationAwareStringBuilder {
        IndentationAwareStringBuilder {
            indentation_type: IndentationType::Spaces,
            tab_size: Some(4),
            current_indentation_level: 0,
            buffer: Vec::new(),
        }
    }

    pub fn with_indentation_type(
        &mut self,
        indentation_type: IndentationType,
    ) -> &mut IndentationAwareStringBuilder {
        self.indentation_type = indentation_type;

        self.tab_size = match self.indentation_type {
            IndentationType::Spaces => Some(4),
            IndentationType::Tabs => None,
        };

        self
    }

    pub fn with_tab_size(&mut self, tab_size: usize) -> &mut IndentationAwareStringBuilder {
        self.tab_size = Some(tab_size);

        self
    }

    pub fn add(&mut self, new_line: &str) {
        let indent_string = match self.indentation_type {
            IndentationType::Spaces => " ",
            IndentationType::Tabs => "\t",
        };
        let current_indentation = match self.indentation_type {
            IndentationType::Spaces => {
                indent_string.repeat(self.current_indentation_level * self.tab_size.unwrap_or(4))
            }
            IndentationType::Tabs => indent_string.repeat(self.current_indentation_level),
        };
        self.buffer
            .push(format!("{}{}", current_indentation, new_line));
    }

    pub fn increase_indentation(&mut self) {
        self.current_indentation_level += 1;
    }

    pub fn decrease_indentation(&mut self) {
        self.current_indentation_level -= 1;
    }

    pub fn build_string(self) -> String {
        self.buffer.join("\n")
    }
}
