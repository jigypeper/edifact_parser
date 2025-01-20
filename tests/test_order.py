import pytest
from edifact_parser import Order

def test_order_from_edifact(sample_edifact):
    order = Order.from_edifact(sample_edifact)
    assert order.interchange_header is not None
    assert order.message_header is not None
    assert len(order.segments) > 0

def test_order_get_segments(sample_edifact):
    order = Order.from_edifact(sample_edifact)
    lin_segments = order.get_all_segments("LIN")
    assert len(lin_segments) == 1
    assert lin_segments[0].tag == "LIN"

def test_order_lines(sample_edifact):
    order = Order.from_edifact(sample_edifact)
    lines = order.get_order_lines()
    assert len(lines) == 1
    assert lines[0].line_segment.tag == "LIN"
    assert lines[0].quantity is not None
    assert lines[0].price is not None
