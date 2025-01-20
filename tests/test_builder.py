import pytest
from edifact_parser import OrderBuilder

def test_order_builder():
    builder = OrderBuilder()
    builder = (builder
        .with_interchange_header("SENDER", "RECEIVER", "20240119:1200", "REF123")
        .with_message_header("1", "ORDERS")
        .with_bgm("220", "123456", "9")
        .add_order_line("1", "ITEM123", "5", "10.00"))
    
    order = builder.build()
    assert order.interchange_header is not None
    assert order.message_header is not None
    assert len(order.segments) > 0
