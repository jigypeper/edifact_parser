import pytest
from edifact_parser import Parser

def test_parser_default_delimiters(parser):
    assert parser is not None
    
def test_parser_custom_delimiters(parser):
    parser.set_delimiters("UNA|^.?@~")
    segment = parser.parse_segment("BGM^220^123456^9~", 0)
    assert segment.tag == "BGM"
    assert segment.elements[0][0] == "220"
    assert segment.elements[1][0] == "123456"
    assert segment.elements[2][0] == "9"

def test_parser_escaped_characters(parser):
    parser.set_delimiters("UNA:+.?*'")
    segment = parser.parse_segment("FTX+AAA+BBB?+CCC'", 0)
    assert segment.tag == "FTX"
    assert segment.elements[1][0] == "BBB+CCC"
