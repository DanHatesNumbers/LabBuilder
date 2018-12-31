pub struct IndentationAwareStringBuilder {
    indentation_type: IndentationType,
    tab_size: Option<usize>,
    current_indentation_level: usize,
    buffer: Vec<String>,
}

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

    pub fn with_indentation_type<'a>(
        &'a mut self,
        indentation_type: IndentationType,
    ) -> &'a mut IndentationAwareStringBuilder {
        self.indentation_type = indentation_type;

        self.tab_size = match self.indentation_type {
            IndentationType::Spaces => Some(4),
            IndentationType::Tabs => None,
        };

        self
    }

    pub fn with_tab_size<'a>(
        &'a mut self,
        tab_size: usize,
    ) -> &'a mut IndentationAwareStringBuilder {
        self.tab_size = Some(tab_size);

        self
    }

    pub fn add(&mut self, new_line: String) {
        let indent_string = match self.indentation_type {
            IndentationType::Spaces => vec![" ".to_string()],
            IndentationType::Tabs => vec!["\t".to_string()],
        };
        let indentation = indent_string.iter().cloned().cycle();
        let mut current_indentation = match self.indentation_type {
            IndentationType::Spaces => indentation
                .clone()
                .take(self.current_indentation_level * self.tab_size.unwrap_or(4))
                .collect::<String>(),
            IndentationType::Tabs => indentation
                .clone()
                .take(self.current_indentation_level)
                .collect::<String>(),
        };
        self.buffer
            .push(format!("{}{}", current_indentation, new_line).to_string());
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
