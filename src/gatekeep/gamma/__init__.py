import random
import mesa


class TrafficLight(mesa.Agent):
    def __init__(self, unique_id, model):
        super().__init__(unique_id, model)
        self.state = "red"

    def change_state(self, new_state):
        self.state = new_state


class Car(mesa.Agent):
    def __init__(self, unique_id, model, direction, debug=True):
        super().__init__(unique_id, model)
        self.direction = direction  # 'N', 'S', 'E', 'W'
        self.crashed = False
        self.passed_light = False
        self.DEBUG = debug

    def move(self):
        if self.crashed:
            return None

        x, y = self.pos
        if self.direction == "N":
            new_pos = (x, y + 1)
        elif self.direction == "S":
            new_pos = (x, y - 1)
        elif self.direction == "E":
            new_pos = (x + 1, y)
        elif self.direction == "W":
            new_pos = (x - 1, y)

        if self.DEBUG:
            print(
                f"Car {self.unique_id} direction: {self.direction}, current pos: {self.pos}, new pos: {new_pos}"
            )

        if self.model.grid.out_of_bounds(new_pos):
            if self.DEBUG:
                print(f"Car {self.unique_id} has left the grid, removing...")
            self.model.remove_car(self)
            return

        # Check if the car is at the traffic light
        if self.at_traffic_light():
            light = self.model.traffic_lights[self.direction]
            if light.state == "red":
                if self.DEBUG:
                    print(f"Car {self.unique_id} is at red light, stopping...")
                return  # Stop at red light

        cell_contents = self.model.grid.get_cell_list_contents(new_pos)
        other_cars = [obj for obj in cell_contents if isinstance(obj, Car)]

        if not other_cars:
            if self.DEBUG:
                print(f"Car {self.unique_id} moving to {new_pos}")
            self.model.grid.move_agent(self, new_pos)
        else:
            if self.DEBUG:
                print(
                    f"Car {self.unique_id} has collided with car {other_cars[0].unique_id}!"
                )
            self.check_collision(other_cars[0].pos)

    def at_traffic_light(self):
        x, y = self.pos
        if self.direction == "N" and y == 2:
            return True
        elif self.direction == "S" and y == 8:
            return True
        elif self.direction == "E" and x == 2:
            return True
        elif self.direction == "W" and x == 8:
            return True
        return False

    def check_if_passed_light(self, new_pos):
        x, y = new_pos
        if self.direction == "N" and y >= 3:
            self.passed_light = True
        elif self.direction == "S" and y <= 7:
            self.passed_light = True
        elif self.direction == "E" and x >= 3:
            self.passed_light = True
        elif self.direction == "W" and x <= 7:
            self.passed_light = True

    def check_collision_(self, other_car):
        if not other_car.crashed:
            self.crashed = True
            other_car.crashed = True
            self.model.collision_count += 1
            self.model.remove_crashed_cars(self, other_car)

    def check_collision(self, new_pos):
        if self.passed_light:
            return
        light = self.model.traffic_lights[self.direction]
        if light.state == "green":
            other_agents = self.model.grid.get_cell_list_contents(new_pos)
            for agent in other_agents:
                if isinstance(agent, Car) and not agent.crashed:
                    self.crashed = True
                    agent.crashed = True
                    self.model.collision_count += 1
                    self.model.remove_crashed_cars(self, agent)
                    return

    def step(self):
        self.move()


class TrafficModel(mesa.Model):
    def __init__(self, N):
        super().__init__()
        self.num_agents = N
        self.grid = mesa.space.MultiGrid(11, 11, False)
        self.schedule = mesa.time.RandomActivation(self)
        self.collision_count = 0
        self.steps_since_last_change = 0
        self.change_frequency = 20  # Change lights every 20 steps

        # Create traffic lights
        self.traffic_lights = {
            "N": TrafficLight("N", self),
            "S": TrafficLight("S", self),
            "E": TrafficLight("E", self),
            "W": TrafficLight("W", self),
        }

        # Place traffic lights on the grid
        self.grid.place_agent(self.traffic_lights["N"], (6, 7))
        self.grid.place_agent(self.traffic_lights["S"], (4, 3))
        self.grid.place_agent(self.traffic_lights["E"], (7, 4))
        self.grid.place_agent(self.traffic_lights["W"], (3, 6))

        # Create agents
        for i in range(self.num_agents):
            self.add_car()

        self.datacollector = mesa.DataCollector(
            model_reporters={"Collisions": "collision_count"}
        )

    def add_car(self):
        direction = random.choice(["N", "S", "E", "W"])
        a = Car(self.next_id(), self, direction)
        self.schedule.add(a)
        if direction == "N":
            x = 4
            y = 0
        elif direction == "S":
            x = 6
            y = 10
        elif direction == "E":
            x = 0
            y = 6
        else:  # 'W'
            x = 10
            y = 4
        self.grid.place_agent(a, (x, y))

    def remove_car(self, car):
        self.grid.remove_agent(car)
        self.schedule.remove(car)
        self.add_car()  # Add a new car to replace the removed one

    def remove_crashed_cars(self, *cars):
        for car in cars:
            self.grid.remove_agent(car)
            self.schedule.remove(car)
        for _ in cars:  # Add new cars to replace the crashed ones
            self.add_car()

    def unsafe_traffic_light_control(self):
        # Randomly set traffic lights, allowing unsafe configurations
        for light in self.traffic_lights.values():
            light.change_state(random.choice(["red", "green"]))

    def step(self):
        self.steps_since_last_change += 1

        if self.steps_since_last_change >= self.change_frequency:
            self.unsafe_traffic_light_control()
            self.steps_since_last_change = 0

        self.schedule.step()
        self.datacollector.collect(self)


class TrafficLightText(mesa.visualization.TextElement):
    def __init__(self):
        pass

    def render(self, model):
        return (
            "Traffic Lights: "
            f"N: {model.traffic_lights['N'].state}, "
            f"S: {model.traffic_lights['S'].state}, "
            f"E: {model.traffic_lights['E'].state}, "
            f"W: {model.traffic_lights['W'].state}"
        )


def agent_portrayal(agent):
    if isinstance(agent, Car):
        portrayal = {
            "Shape": "circle",
            "Layer": 1,
            "Filled": "true",
            "r": 0.5,
            "Color": "red" if agent.crashed else "blue",
        }
    elif isinstance(agent, TrafficLight):
        portrayal = {
            "Shape": "rect",
            "Filled": "true",
            "w": 0.8,
            "h": 0.8,
            "Layer": 0,
            "Color": agent.state,
            "text": agent.unique_id,
            "text_color": "black",
        }
    return portrayal
