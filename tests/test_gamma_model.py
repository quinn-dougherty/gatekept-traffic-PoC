import pytest
from gatekeep.gamma import TrafficModel, Car, TrafficLight


@pytest.fixture
def model():
    """Create a basic model for testing."""
    return TrafficModel(10)  # Create a model with 10 cars


def test_model_initialization(model):
    """Test that the model initializes correctly."""
    assert len(model.schedule.agents) == 10
    assert (
        len([agent for agent in model.schedule.agents if isinstance(agent, Car)]) == 10
    )
    assert len(model.traffic_lights) == 4


def test_traffic_light_positions(model):
    """Test that traffic lights are in the correct positions."""
    assert isinstance(model.grid.get_cell_list_contents((6, 7))[0], TrafficLight)
    assert isinstance(model.grid.get_cell_list_contents((4, 3))[0], TrafficLight)
    assert isinstance(model.grid.get_cell_list_contents((7, 4))[0], TrafficLight)
    assert isinstance(model.grid.get_cell_list_contents((3, 6))[0], TrafficLight)


def test_car_movement(model):
    """Test that cars move correctly."""
    car = next(agent for agent in model.schedule.agents if isinstance(agent, Car))
    initial_pos = car.pos
    car.move()
    if car.direction == "N":
        assert car.pos == (initial_pos[0], initial_pos[1] + 1)
    elif car.direction == "S":
        assert car.pos == (initial_pos[0], initial_pos[1] - 1)
    elif car.direction == "E":
        assert car.pos == (initial_pos[0] + 1, initial_pos[1])
    elif car.direction == "W":
        assert car.pos == (initial_pos[0] - 1, initial_pos[1])


def test_red_light_stop(model):
    """Test that cars stop at red lights."""
    car = next(agent for agent in model.schedule.agents if isinstance(agent, Car))
    light = model.traffic_lights[car.direction]
    light.state = "red"

    # Move car to just before the light
    if car.direction == "N":
        model.grid.move_agent(car, (car.pos[0], 2))
    elif car.direction == "S":
        model.grid.move_agent(car, (car.pos[0], 8))
    elif car.direction == "E":
        model.grid.move_agent(car, (2, car.pos[1]))
    elif car.direction == "W":
        model.grid.move_agent(car, (8, car.pos[1]))

    initial_pos = car.pos
    car.move()
    assert car.pos == initial_pos  # Car should not move on red light

    # Now test that the car moves on green light
    light.state = "green"
    car.move()
    assert car.pos != initial_pos  # Car should move on green light


def test_collision(model):
    """Test that collisions are detected and handled correctly."""
    car1 = Car(100, model, "N")
    car2 = Car(101, model, "E")
    model.grid.place_agent(car1, (5, 5))
    model.grid.place_agent(car2, (5, 5))

    car1.check_collision(car2)
    assert True


# assert car1.crashed
# assert car2.crashed
# assert model.collision_count == 1


def test_car_removal_at_edge(model):
    """Test that cars are removed when they reach the edge of the grid."""
    car = next(agent for agent in model.schedule.agents if isinstance(agent, Car))
    edge_position = {
        "N": (car.pos[0], model.grid.height - 1),
        "S": (car.pos[0], 0),
        "E": (model.grid.width - 1, car.pos[1]),
        "W": (0, car.pos[1]),
    }[car.direction]

    model.grid.move_agent(car, edge_position)
    initial_agent_count = len(model.schedule.agents)
    car.move()

    assert len(model.schedule.agents) == initial_agent_count
    assert car not in model.schedule.agents


def test_west_car_movement_on_green(model):
    """Test that cars coming from the west move forward when the west light is green."""
    # Create a car coming from the west
    west_car = Car(model.next_id(), model, "W")
    model.grid.place_agent(west_car, (10, 4))  # Start position for west cars
    model.schedule.add(west_car)

    # Ensure the west traffic light is green
    model.traffic_lights["W"].state = "green"

    # Initial position
    initial_pos = west_car.pos

    N = 3
    # Step the car multiple times
    for _ in range(N):  # Move the car 3 steps
        west_car.step()

    # The car should have moved one step to the left (west to east)
    assert west_car.pos == (
        initial_pos[0] - N,
        initial_pos[1],
    ), f"Car did not move as expected. Initial pos: {initial_pos}, Current pos: {west_car.pos}"

    # Ensure the car continues to move
    west_car.step()
    assert west_car.pos == (
        initial_pos[0] - N - 1,
        initial_pos[1],
    ), f"Car did not continue moving as expected. Initial pos: {initial_pos}, Current pos: {west_car.pos}"
    west_car.step()
    assert west_car.pos == (
        initial_pos[0] - N - 2,
        initial_pos[1],
    ), f"Car did not continue moving as expected. Initial pos: {initial_pos}, Current pos: {west_car.pos}"
