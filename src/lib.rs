mod validation;

#[derive(Debug)]
pub struct Tool {
    pub vendor: Option<String>,
    pub name: Option<String>,
}

#[derive(Debug)]
pub struct Metadata {
    pub timestamp: Option<String>,
    pub tools: Option<Vec<Tool>>,
}

#[derive(Debug)]
pub struct Bom {
    pub serial_number: Option<String>,
    pub meta_data: Option<Metadata>,
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_validates() {
    }
}
