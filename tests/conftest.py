import pytest
from edifact_parser import Parser, Order, OrderBuilder

@pytest.fixture
def parser():
    return Parser()

@pytest.fixture
def sample_edifact():
    return """UNA:+.?*'
UNB+UNOA:4+SENDER+RECEIVER+20240119:1200+REF123'
UNH+1+ORDERS:D:96A:UN'
BGM+220+123456+9'
LIN+1++ITEM123:BP'
QTY+21:5'
PRI+AAA:10.00'"""
