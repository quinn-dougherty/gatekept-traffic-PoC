import mesa
import colorsys
import numpy as np


def get_color_gradient(value, min_value, max_value):
    """
    Returns a color along a gradient from white to red based on the value.
    """
    # Normalize the value
    normalized = (value - min_value) / (max_value - min_value)

    # Use HSV color space for a smooth transition
    # Hue goes from 0 (red) to 0.3 (greenish)
    hue = 0.3 * (1 - normalized)
    saturation = 0.7 * normalized  # Increase saturation as density increases
    value = 1 - (0.3 * normalized)  # Slightly decrease brightness as density increases

    # Convert HSV to RGB
    r, g, b = colorsys.hsv_to_rgb(hue, saturation, value)

    # Convert to hex
    return "#{:02x}{:02x}{:02x}".format(int(r * 255), int(g * 255), int(b * 255))


class Road(mesa.Agent):
    def __init__(self, unique_id, model, direction, max_density):
        super().__init__(unique_id, model)
        self.direction = direction
        self.density = 0  # Current vehicle density
        self.max_density = max_density  # Maximum vehicle density
        self.flow = 0  # Current flow

    def step(self):
        # Calculate flow based on the LWR fundamental diagram
        # We'll use a triangular fundamental diagram
        critical_density = self.max_density / 2
        max_flow = 0.5  # Maximum flow rate

        if self.density <= critical_density:
            self.flow = (max_flow / critical_density) * self.density
        else:
            self.flow = max_flow - (
                max_flow / (self.max_density - critical_density)
            ) * (self.density - critical_density)

        # Consider traffic light state
        light = self.model.traffic_lights[self.direction]
        if light.state == "red":
            self.flow *= 0.1  # Reduce flow significantly at red light

        # Update density based on inflow and outflow
        inflow = self.model.get_inflow(self)
        outflow = self.flow

        self.density += inflow - outflow

        # Ensure density stays within bounds
        self.density = max(0, min(self.density, self.max_density))


class TrafficLight(mesa.Agent):
    def __init__(self, unique_id, model):
        super().__init__(unique_id, model)
        self.state = "red"

    def change_state(self, new_state):
        self.state = new_state


class TrafficModel(mesa.Model):
    def __init__(self, road_length=10, max_density=1, change_frequency=20):
        super().__init__()
        self.road_length = road_length
        self.max_density = max_density
        self.grid = mesa.space.MultiGrid(road_length, road_length, False)
        self.schedule = mesa.time.RandomActivation(self)
        self.change_frequency = change_frequency
        self.steps_since_last_change = 0

        # Create roads
        self.roads = {
            "N": Road(0, self, "N", max_density),
            "S": Road(1, self, "S", max_density),
            "E": Road(2, self, "E", max_density),
            "W": Road(3, self, "W", max_density),
        }
        for road in self.roads.values():
            self.schedule.add(road)

        # Create traffic lights
        self.traffic_lights = {
            "N": TrafficLight("N", self),
            "S": TrafficLight("S", self),
            "E": TrafficLight("E", self),
            "W": TrafficLight("W", self),
        }
        light_positions = {
            "N": (road_length // 2, road_length - 1),
            "S": (road_length // 2 - 1, 0),
            "E": (road_length - 1, road_length // 2),
            "W": (0, road_length // 2 - 1),
        }
        for direction, light in self.traffic_lights.items():
            self.grid.place_agent(light, light_positions[direction])
            self.schedule.add(light)

        self.datacollector = mesa.DataCollector(
            model_reporters={
                "Average Density": lambda m: sum(
                    road.density for road in m.roads.values()
                )
                / len(m.roads),
                "Light States": lambda m: {
                    dir: light.state for dir, light in m.traffic_lights.items()
                },
            }
        )

    def get_inflow(self, road):
        # Base inflow rate
        base_rate = 0.1

        # Add some randomness
        random_factor = np.random.uniform(0.8, 1.2)

        # Consider current density to reduce inflow when road is crowded
        density_factor = 1 - (road.density / road.max_density)

        return base_rate * random_factor * density_factor

    def unsafe_traffic_light_control(self):
        for light in self.traffic_lights.values():
            light.change_state(np.random.choice(["red", "green"]))

    def step(self):
        self.schedule.step()

        if self.steps_since_last_change >= self.change_frequency:
            self.unsafe_traffic_light_control()
            self.steps_since_last_change = 0

        self.datacollector.collect(self)


def agent_portrayal(agent):
    if isinstance(agent, TrafficLight):
        return {
            "Shape": "circle",
            "r": 0.8,
            "Filled": "true",
            "Layer": 1,
            "Color": agent.state,
        }
    else:  # This is a grid cell, color it based on the corresponding road's density
        portrayal = {"Shape": "rect", "w": 0.8, "h": 0.8, "Filled": "true", "Layer": 0}
        x, y = agent.pos
        mid = agent.model.road_length // 2

        if x == mid - 1 and y < mid:  # South lane
            road = agent.model.roads["S"]
        elif x == mid and y >= mid:  # North lane
            road = agent.model.roads["N"]
        elif y == mid - 1 and x < mid:  # West lane
            road = agent.model.roads["W"]
        elif y == mid and x >= mid:  # East lane
            road = agent.model.roads["E"]
        else:
            return {
                "Color": "#D3D3D3",
                "Shape": "rect",
                "w": 0.8,
                "h": 0.8,
                "Filled": "true",
                "Layer": 0,
            }  # Light gray for non-road cells

        portrayal["Color"] = get_color_gradient(road.density, 0, road.max_density)
    return portrayal
