import pytest
from gatekeep.gamma import lwr2


@pytest.fixture
def model():
    return lwr2.TrafficModel(10)
