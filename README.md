# EDIFACT Parser

A Rust-based EDIFACT parser with Python bindings, designed for parsing and creating UN/EDIFACT messages. This library provides a way to work with EDIFACT documents in Python applications while leveraging Rust's speed and safety guarantees.

## Features

- Python-friendly API
- Support for custom delimiters via UNA segments
- Handling of escape sequences
- Builder pattern for creating EDIFACT messages
- Support for parsing order messages
- Test coverage

## Installation

The package requires Python 3.9 or higher. Since this project is currently only available on GitHub, you'll need to install it from source:

1. Clone the repository:
```bash
git clone https://github.com/yourusername/edifact-parser
cd edifact-parser
```

2. Install uv:
```bash
pip install uv
```

3. Create and activate a virtual environment:
```bash
uv venv
source .venv/bin/activate  # On Windows: .venv\Scripts\activate
```

4. Install development dependencies:
```bash
uv pip install -r requirements-dev.txt
```

5. Build and install the package:
```bash
maturin develop --features "extension-module"
```

## Usage

### Parsing EDIFACT Messages

```python
from edifact_parser import Order

# Parse an EDIFACT message
edifact_content = """UNA:+.?*'
UNB+UNOA:4+SENDER+RECEIVER+20240119:1200+REF123'
UNH+1+ORDERS:D:96A:UN'
BGM+220+123456+9'
LIN+1++ITEM123:BP'
QTY+21+5'
PRI+AAA+10.00'"""

order = Order.from_edifact(edifact_content)

# Access order segments
lin_segments = order.get_all_segments("LIN")
order_lines = order.get_order_lines()
```

### Creating EDIFACT Messages

```python
from edifact_parser import OrderBuilder

# Create a new order using the builder pattern
builder = OrderBuilder()
order = (builder
    .with_interchange_header("SENDER", "RECEIVER", "20240119:1200", "REF123")
    .with_message_header("1", "ORDERS")
    .with_bgm("220", "123456", "9")
    .add_order_line("1", "ITEM123", "5", "10.00")
    .build())

# Convert back to EDIFACT format
edifact_string = order.to_edifact()
```

## Development

### Prerequisites

- Rust (latest stable version)
- Python 3.9+
- uv (for dependency management)
- maturin (will be installed by uv)
- pytest (will be installed by uv)

### Setting Up Development Environment

1. Clone the repository:
```bash
git clone https://github.com/yourusername/edifact-parser
cd edifact-parser
```

2. Install uv if you haven't already:
[Details on UV Official Site](https://docs.astral.sh/uv/getting-started/installation/)

3. Create and activate a virtual environment:
```bash
uv venv
source .venv/bin/activate  # On Windows: .venv\Scripts\activate
```

4. Install development dependencies:
```bash
uv pip install -r requirements-dev.txt
```

5. Build the package:
```bash
maturin develop
```

### Running Tests

Run the test suite using pytest:

```bash
uv run pytest
```

### CI/CD

This project uses uv for dependency management in CI/CD pipelines. The key benefits include:
- Faster dependency resolution and installation
- Reproducible builds through precise dependency locking
- Consistent environments across development and CI

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

Please ensure that:
- Tests pass when run with pytest
- New features include appropriate tests
- Documentation is updated as needed

## License

This project is licensed under the GNU Affero General Public License v3.0 License - see the LICENSE file for details.

## Acknowledgments

- UN/EDIFACT working group for the message standards
- The Rust and Python communities for their excellent tools and documentation
