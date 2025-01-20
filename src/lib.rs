use pyo3::prelude::*;
#[allow(unused_imports)]
use pyo3::types::{PyDict, PyList};
use std::collections::HashMap;
use std::error::Error;
use std::fmt;

#[derive(Debug)]
#[allow(dead_code)]
struct EdifactError {
    message: String,
}

impl fmt::Display for EdifactError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "EDIFACT Error: {}", self.message)
    }
}

impl Error for EdifactError {}

#[pyclass]
#[derive(Debug, PartialEq, Eq, Clone)]
struct Delimiters {
    component: char,
    data: char,
    decimal: char,
    escape: char,
    segment: char,
    reserved: char,
}

impl Default for Delimiters {
    fn default() -> Self {
        Delimiters {
            component: ':',
            data: '+',
            decimal: '.',
            escape: '?',
            segment: '\'',
            reserved: '*',
        }
    }
}

#[pyclass]
#[derive(Debug, Clone)]
#[allow(dead_code)]
struct Segment {
    #[pyo3(get)]
    tag: String,
    #[pyo3(get)]
    elements: Vec<Vec<String>>, // Components within elements
    position: usize,
}

#[pymethods]
impl Segment {
    #[new]
    fn new(tag: String, elements: Vec<Vec<String>>, position: usize) -> Self {
        Segment {
            tag,
            elements,
            position,
        }
    }

    fn get_element(&self, index: usize) -> Option<&Vec<String>> {
        self.elements.get(index)
    }

    fn get_component(&self, element_index: usize, component_index: usize) -> Option<&String> {
        self.elements
            .get(element_index)
            .and_then(|element| element.get(component_index))
    }

    fn __str__(&self) -> String {
        format!("{}: {:?}", self.tag, self.elements)
    }

    fn to_edifact(&self, delimiters: &Delimiters) -> String {
        let mut result = self.tag.clone();

        for element in &self.elements {
            result.push(delimiters.data);

            for (i, component) in element.iter().enumerate() {
                if i > 0 {
                    result.push(delimiters.component);
                }
                // Escape special characters
                let escaped = component
                    .chars()
                    .map(|c| {
                        if c == delimiters.data
                            || c == delimiters.component
                            || c == delimiters.decimal
                            || c == delimiters.segment
                            || c == delimiters.reserved
                        {
                            format!("{}{}", delimiters.escape, c)
                        } else {
                            c.to_string()
                        }
                    })
                    .collect::<String>();
                result.push_str(&escaped);
            }
        }

        result.push(delimiters.segment);
        result
    }
}

#[pyclass]
#[derive(Debug, Clone)]
struct Parser {
    delimiters: Delimiters,
}

#[pymethods]
impl Parser {
    #[new]
    fn new() -> Self {
        Parser {
            delimiters: Delimiters::default(),
        }
    }

    fn set_delimiters(&mut self, una_segment: &str) -> PyResult<()> {
        if una_segment.len() >= 9 && una_segment.starts_with("UNA") {
            self.delimiters = Delimiters {
                component: una_segment.chars().nth(3).unwrap(),
                data: una_segment.chars().nth(4).unwrap(),
                decimal: una_segment.chars().nth(5).unwrap(),
                escape: una_segment.chars().nth(6).unwrap(),
                reserved: una_segment.chars().nth(7).unwrap(),
                segment: una_segment.chars().nth(8).unwrap(),
            };
        }
        Ok(())
    }

    fn parse_segment(&self, segment_str: &str, position: usize) -> PyResult<Segment> {
        let mut chars = segment_str.chars().peekable();
        let mut tag = String::new();
        let mut elements: Vec<Vec<String>> = Vec::new();
        let mut current_element: Vec<String> = Vec::new();
        let mut current_component = String::new();
        let mut is_escaped = false;

        // Parse tag
        while let Some(c) = chars.next() {
            if c == self.delimiters.data {
                break;
            }
            tag.push(c);
        }

        // Parse elements and components
        loop {
            match chars.next() {
                Some(c) if is_escaped => {
                    current_component.push(c);
                    is_escaped = false;
                }
                Some(c) if c == self.delimiters.escape => {
                    is_escaped = true;
                }
                Some(c) if c == self.delimiters.component => {
                    // Add current component to current element and start new component
                    if !current_component.is_empty() {
                        current_element.push(current_component);
                    } else {
                        current_element.push(String::new());
                    }
                    current_component = String::new();
                }
                Some(c) if c == self.delimiters.data => {
                    // Finish current component and element
                    if !current_component.is_empty() || !current_element.is_empty() {
                        current_element.push(current_component);
                        elements.push(current_element);
                    } else {
                        // Handle empty element
                        elements.push(Vec::new());
                    }
                    current_element = Vec::new();
                    current_component = String::new();

                    // Handle consecutive data delimiters
                    while chars.peek() == Some(&self.delimiters.data) {
                        elements.push(Vec::new());
                        chars.next();
                    }
                }
                Some(c) if c == self.delimiters.segment => {
                    // Finish final component and element if not empty
                    if !current_component.is_empty() || !current_element.is_empty() {
                        if !current_component.is_empty() {
                            current_element.push(current_component);
                        }
                        elements.push(current_element);
                    }
                    break;
                }
                Some(c) => {
                    current_component.push(c);
                }
                None => {
                    // Handle end of input (similar to segment terminator)
                    if !current_component.is_empty() {
                        current_element.push(current_component);
                    }
                    if !current_element.is_empty() {
                        elements.push(current_element);
                    }
                    break;
                }
            }
        }

        Ok(Segment::new(tag, elements, position))
    }
}

#[pyclass]
#[allow(dead_code)]
struct Message {
    segments: Vec<Segment>,
    service_segments: HashMap<String, Segment>,
}

#[pymethods]
impl Message {
    #[new]
    fn new() -> Self {
        Message {
            segments: Vec::new(),
            service_segments: HashMap::new(),
        }
    }

    fn get_segments_by_tag(&self, tag: &str) -> Vec<Segment> {
        self.segments
            .iter()
            .filter(|s| s.tag == tag)
            .cloned()
            .collect()
    }
}

#[pyclass]
#[derive(Debug, Clone)]
struct Order {
    #[pyo3(get)]
    segments: Vec<Segment>,
    #[pyo3(get)]
    interchange_header: Option<Segment>,
    #[pyo3(get)]
    message_header: Option<Segment>,
    parser: Parser,
}

#[pymethods]
impl Order {
    #[new]
    fn new() -> Self {
        Order {
            segments: Vec::new(),
            interchange_header: None,
            message_header: None,
            parser: Parser::new(),
        }
    }

    #[staticmethod]
    fn from_edifact(content: String) -> PyResult<Order> {
        let mut order = Order::new();
        let mut position = 0;

        // Handle UNA segment if present
        if content.starts_with("UNA") {
            let una_line = content.lines().next().unwrap();
            order.parser.set_delimiters(una_line)?;
        }

        for line in content.lines() {
            if line.trim().is_empty() || line.starts_with("UNA") {
                continue;
            }

            let segment = order.parser.parse_segment(line, position)?;

            match segment.tag.as_str() {
                "UNB" => order.interchange_header = Some(segment.clone()),
                "UNH" => order.message_header = Some(segment.clone()),
                _ => order.segments.push(segment),
            }

            position += 1;
        }

        Ok(order)
    }

    fn get_segment(&self, tag: &str) -> Option<Segment> {
        self.segments.iter().find(|s| s.tag == tag).cloned()
    }

    fn get_all_segments(&self, tag: &str) -> Vec<Segment> {
        self.segments
            .iter()
            .filter(|s| s.tag == tag)
            .cloned()
            .collect()
    }

    fn get_order_lines(&self) -> PyResult<Vec<OrderLine>> {
        let mut lines = Vec::new();
        let mut current_line: Option<OrderLine> = None;

        for segment in &self.segments {
            match segment.tag.as_str() {
                "LIN" => {
                    if let Some(line) = current_line {
                        lines.push(line);
                    }
                    current_line = Some(OrderLine::new(segment.clone()));
                }
                "IMD" | "QTY" | "MOA" | "PRI" | "RFF" => {
                    if let Some(ref mut line) = current_line {
                        line.add_segment(segment.clone());
                    }
                }
                _ => {}
            }
        }

        if let Some(line) = current_line {
            lines.push(line);
        }

        Ok(lines)
    }

    fn to_edifact(&self) -> PyResult<String> {
        let mut result = String::new();

        // Add UNA segment if using non-default delimiters
        if self.parser.delimiters != Delimiters::default() {
            result.push_str(&format!(
                "UNA{}{}{}{}{}{}\n",
                self.parser.delimiters.component,
                self.parser.delimiters.data,
                self.parser.delimiters.decimal,
                self.parser.delimiters.escape,
                self.parser.delimiters.reserved,
                self.parser.delimiters.segment
            ));
        }

        // Add interchange header if present
        if let Some(ref header) = self.interchange_header {
            result.push_str(&header.to_edifact(&self.parser.delimiters));
            result.push('\n');
        }

        // Add message header if present
        if let Some(ref header) = self.message_header {
            result.push_str(&header.to_edifact(&self.parser.delimiters));
            result.push('\n');
        }

        // Add all other segments
        for segment in &self.segments {
            result.push_str(&segment.to_edifact(&self.parser.delimiters));
            result.push('\n');
        }

        Ok(result)
    }

    fn create_segment(&self, tag: &str, elements: Vec<Vec<String>>) -> PyResult<Segment> {
        Ok(Segment::new(tag.to_string(), elements, self.segments.len()))
    }

    fn add_segment(&mut self, segment: Segment) {
        self.segments.push(segment);
    }
}

#[pyclass]
struct OrderLine {
    #[pyo3(get)]
    line_segment: Segment,
    #[pyo3(get)]
    description: Option<Segment>,
    #[pyo3(get)]
    quantity: Option<Segment>,
    #[pyo3(get)]
    amount: Option<Segment>,
    #[pyo3(get)]
    price: Option<Segment>,
    #[pyo3(get)]
    reference: Option<Segment>,
}

#[pymethods]
impl OrderLine {
    #[new]
    fn new(line_segment: Segment) -> Self {
        OrderLine {
            line_segment,
            description: None,
            quantity: None,
            amount: None,
            price: None,
            reference: None,
        }
    }

    fn add_segment(&mut self, segment: Segment) {
        match segment.tag.as_str() {
            "IMD" => self.description = Some(segment),
            "QTY" => self.quantity = Some(segment),
            "MOA" => self.amount = Some(segment),
            "PRI" => self.price = Some(segment),
            "RFF" => self.reference = Some(segment),
            _ => {}
        }
    }
}

// Add builder patterns for creating EDIFACT messages
#[pyclass]
#[derive(Debug, Clone)]
pub struct OrderBuilder {
    order: Order,
}

#[pymethods]
impl OrderBuilder {
    #[new]
    fn new() -> Self {
        OrderBuilder {
            order: Order::new(),
        }
    }

    fn with_interchange_header(
        &mut self,
        sender: &str,
        recipient: &str,
        date: &str,
        control_ref: &str,
        py: Python,
    ) -> PyResult<Py<OrderBuilder>> {
        let elements = vec![
            vec!["UNOA".to_string(), "4".to_string()],
            vec![sender.to_string()],
            vec![recipient.to_string()],
            vec![date.to_string()],
            vec![control_ref.to_string()],
            vec!["ORDERS".to_string()],
        ];

        self.order.interchange_header = Some(Segment::new("UNB".to_string(), elements, 0));
        Py::new(py, self.clone())
    }

    fn with_message_header(
        &mut self,
        message_ref: &str,
        message_type: &str,
        py: Python,
    ) -> PyResult<Py<OrderBuilder>> {
        let elements = vec![
            vec![message_ref.to_string()],
            vec![
                message_type.to_string(),
                "D".to_string(),
                "01B".to_string(),
                "UN".to_string(),
            ],
        ];

        self.order.message_header = Some(Segment::new("UNH".to_string(), elements, 1));
        Py::new(py, self.clone())
    }

    fn with_bgm(
        &mut self,
        message_name: &str,
        doc_number: &str,
        message_function: &str,
        py: Python,
    ) -> PyResult<Py<OrderBuilder>> {
        let elements = vec![
            vec![message_name.to_string()],
            vec![doc_number.to_string()],
            vec![message_function.to_string()],
        ];

        self.order.add_segment(Segment::new(
            "BGM".to_string(),
            elements,
            self.order.segments.len(),
        ));
        Py::new(py, self.clone())
    }

    fn add_order_line(
        &mut self,
        line_number: &str,
        item_number: &str,
        quantity: &str,
        price: &str,
        py: Python,
    ) -> PyResult<Py<OrderBuilder>> {
        // LIN segment
        let lin_elements = vec![
            vec![line_number.to_string()],
            vec![],
            vec![item_number.to_string(), "BP".to_string()],
        ];
        self.order.add_segment(Segment::new(
            "LIN".to_string(),
            lin_elements,
            self.order.segments.len(),
        ));

        // QTY segment
        let qty_elements = vec![vec!["21".to_string()], vec![quantity.to_string()]];
        self.order.add_segment(Segment::new(
            "QTY".to_string(),
            qty_elements,
            self.order.segments.len(),
        ));

        // PRI segment
        let pri_elements = vec![vec!["AAA".to_string()], vec![price.to_string()]];
        self.order.add_segment(Segment::new(
            "PRI".to_string(),
            pri_elements,
            self.order.segments.len(),
        ));

        Py::new(py, self.clone())
    }

    fn build(&self) -> Order {
        self.order.clone()
    }
}

#[pymodule]
fn edifact_parser(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Segment>()?;
    m.add_class::<Parser>()?;
    m.add_class::<Message>()?;
    m.add_class::<OrderLine>()?;
    m.add_class::<Order>()?;
    m.add_class::<OrderBuilder>()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use pyo3::Python;

    // Helper function to create a test parser with default delimiters
    fn setup_test_parser() -> Parser {
        let mut parser = Parser::new();
        parser.set_delimiters("UNA:+.?*'").unwrap();
        parser
    }

    #[test]
    fn test_default_delimiters() {
        Python::with_gil(|_py| {
            let parser = Parser::new();
            assert_eq!(parser.delimiters.component, ':');
            assert_eq!(parser.delimiters.data, '+');
            assert_eq!(parser.delimiters.decimal, '.');
            assert_eq!(parser.delimiters.escape, '?');
            assert_eq!(parser.delimiters.segment, '\'');
            assert_eq!(parser.delimiters.reserved, '*');
        });
    }

    #[test]
    fn test_custom_delimiters() {
        Python::with_gil(|_py| {
            let mut parser = Parser::new();
            parser.set_delimiters("UNA|^.?@~").unwrap();
            assert_eq!(parser.delimiters.component, '|');
            assert_eq!(parser.delimiters.data, '^');
            assert_eq!(parser.delimiters.decimal, '.');
            assert_eq!(parser.delimiters.escape, '?');
            assert_eq!(parser.delimiters.reserved, '@');
            assert_eq!(parser.delimiters.segment, '~');
        });
    }

    #[test]
    fn test_basic_segment_parsing() {
        Python::with_gil(|_py| {
            let parser = setup_test_parser();
            let segment = parser.parse_segment("BGM+220+123456+9'", 0).unwrap();

            assert_eq!(segment.tag, "BGM");
            assert_eq!(segment.elements.len(), 3);
            assert_eq!(segment.elements[0][0], "220");
            assert_eq!(segment.elements[1][0], "123456");
            assert_eq!(segment.elements[2][0], "9");
        });
    }

    #[test]
    fn test_component_parsing() {
        Python::with_gil(|_py| {
            let parser = setup_test_parser();
            let segment = parser.parse_segment("NAD+BY+5021376940009::9'", 0).unwrap();

            assert_eq!(segment.tag, "NAD");
            assert_eq!(segment.elements[1].len(), 3);
            assert_eq!(segment.elements[1][0], "5021376940009");
            assert_eq!(segment.elements[1][1], "");
            assert_eq!(segment.elements[1][2], "9");
        });
    }

    #[test]
    fn test_escaped_characters() {
        Python::with_gil(|_py| {
            let parser = setup_test_parser();

            // Test basic escape
            let segment = parser.parse_segment("FTX+AAA+BBB?+CCC'", 0).unwrap();
            assert_eq!(segment.tag, "FTX");
            assert_eq!(segment.elements[1][0], "BBB+CCC");

            // Test escaping data separator
            let segment = parser.parse_segment("FTX+AAA+BBB?+CCC+DDD'", 0).unwrap();
            assert_eq!(segment.elements[1][0], "BBB+CCC");
            assert_eq!(segment.elements[2][0], "DDD");

            // Test escaping component separator
            let segment = parser.parse_segment("FTX+AAA+BBB?:CCC'", 0).unwrap();
            assert_eq!(segment.elements[1][0], "BBB:CCC");

            // Test escaping segment terminator
            let segment = parser.parse_segment("FTX+AAA+BBB?\'CCC'", 0).unwrap();
            assert_eq!(segment.elements[1][0], "BBB'CCC");

            // Test multiple escapes
            let segment = parser
                .parse_segment("FTX+AAA+BBB?+CCC?:DDD?\'EEE'", 0)
                .unwrap();
            assert_eq!(segment.elements[1][0], "BBB+CCC:DDD'EEE");
        });
    }

    // Add new test for complex escape sequences
    #[test]
    fn test_complex_escape_sequences() {
        Python::with_gil(|_py| {
            let parser = setup_test_parser();

            // Test multiple consecutive escapes
            let segment = parser.parse_segment("FTX+AAA+BBB?+?:?\'CCC'", 0).unwrap();
            assert_eq!(segment.elements[1][0], "BBB+:'CCC");

            // Test escape at end of component
            let segment = parser.parse_segment("FTX+AAA+BBB?++CCC'", 0).unwrap();
            assert_eq!(segment.elements[1][0], "BBB+");
            assert_eq!(segment.elements[2][0], "CCC");

            // Test empty components with escapes
            let segment = parser.parse_segment("FTX+AAA+?++?:+CCC'", 0).unwrap();
            assert_eq!(segment.elements[1][0], "+");
            assert_eq!(segment.elements[2][0], ":");
            assert_eq!(segment.elements[3][0], "CCC");
        });
    }

    #[test]
    fn test_order_parsing() {
        Python::with_gil(|_py| {
            let sample_order = "UNA:+.?*'
UNB+UNOA:4+SENDER+RECEIVER+20240119:1200+REF123'
UNH+1+ORDERS:D:96A:UN'
BGM+220+123456+9'
LIN+1++ITEM123:BP'
QTY+21:5'
PRI+AAA:10.00'";

            let order = Order::from_edifact(sample_order.to_string()).unwrap();

            assert!(order.interchange_header.is_some());
            assert!(order.message_header.is_some());
            assert!(!order.segments.is_empty());

            // Test header contents
            if let Some(ref header) = order.interchange_header {
                assert_eq!(header.tag, "UNB");
                assert_eq!(header.elements[0][0], "UNOA");
                assert_eq!(header.elements[0][1], "4");
                assert_eq!(header.elements[1][0], "SENDER");
            }
        });
    }

    #[test]
    fn test_order_lines() {
        Python::with_gil(|_py| {
            let sample_order = "UNA:+.?*'
UNB+UNOA:4+SENDER+RECEIVER+20240119:1200+REF123'
UNH+1+ORDERS:D:96A:UN'
BGM+220+123456+9'
LIN+1++ITEM123:BP'
QTY+21+5'
PRI+AAA+10.00'";

            let order = Order::from_edifact(sample_order.to_string()).unwrap();
            let lines = order.get_order_lines().unwrap();

            assert_eq!(lines.len(), 1);
            let line = &lines[0];

            assert_eq!(line.line_segment.tag, "LIN");
            assert_eq!(line.line_segment.elements[0][0], "1");
            assert_eq!(line.line_segment.elements[2][0], "ITEM123");

            if let Some(ref qty) = line.quantity {
                assert_eq!(qty.elements[1][0], "5");
            }

            if let Some(ref price) = line.price {
                assert_eq!(price.elements[1][0], "10.00");
            }
        });
    }

    #[test]
    fn test_segment_to_edifact() {
        Python::with_gil(|_py| {
            let delimiters = Delimiters::default();
            let segment = Segment::new(
                "DTM".to_string(),
                vec![
                    vec!["137".to_string()],
                    vec!["20240119".to_string()],
                    vec!["102".to_string()],
                ],
                0,
            );

            assert_eq!(segment.to_edifact(&delimiters), "DTM+137+20240119+102'");
        });
    }

    #[test]
    fn test_get_component() {
        Python::with_gil(|_py| {
            let segment = Segment::new(
                "NAD".to_string(),
                vec![
                    vec!["BY".to_string()],
                    vec!["12345".to_string(), "92".to_string()],
                ],
                0,
            );

            assert_eq!(segment.get_component(1, 1), Some(&"92".to_string()));
            assert_eq!(segment.get_component(1, 2), None);
            assert_eq!(segment.get_component(2, 0), None);
        });
    }

    #[test]
    fn test_order_builder() {
        Python::with_gil(|_py| {
            let mut builder = OrderBuilder::new();
            let order = builder.build();

            assert!(order.segments.is_empty());
            assert!(order.interchange_header.is_none());
            assert!(order.message_header.is_none());
        });
    }

    #[test]
    fn test_message_creation() {
        Python::with_gil(|_py| {
            let message = Message::new();
            assert!(message.segments.is_empty());
            assert!(message.service_segments.is_empty());
        });
    }

    #[test]
    fn test_order_line_creation() {
        Python::with_gil(|_py| {
            let line_segment = Segment::new(
                "LIN".to_string(),
                vec![
                    vec!["1".to_string()],
                    vec![],
                    vec!["ITEM123".to_string(), "BP".to_string()],
                ],
                0,
            );

            let order_line = OrderLine::new(line_segment);
            assert!(order_line.quantity.is_none());
            assert!(order_line.price.is_none());
            assert!(order_line.description.is_none());
            assert!(order_line.amount.is_none());
            assert!(order_line.reference.is_none());
        });
    }
}
