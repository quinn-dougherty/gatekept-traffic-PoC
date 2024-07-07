import random
import colorsys
import mesa

DEBUG = True


class CrashCell(mesa.Agent):
    def step(self):
        if self.pos is not None:
            self.model.grid.remove_agent(self)

    def __repr__(self) -> str:
        return f"CrashCell({self.pos})"


class RoadCell(mesa.Agent):
    def __init__(self, unique_id, model, road, index):
        super().__init__(unique_id, model)
        self.road = road
        self.index = index


class TrafficLight(mesa.Agent):
    def __init__(self, unique_id, model):
        super().__init__(unique_id, model)
        self.state = "red"

    def change_state(self, new_state):
        self.state = new_state


class Road(mesa.Agent):
    def __init__(self, unique_id, model, direction, max_density, length):
        super().__init__(unique_id, model)
        self.direction = direction
        self.density = [0] * length
        self.max_density = max_density
        self.length = length
        self.flow = 0
        self.queue = 0  # Add a queue to represent waiting traffic at red lights
        self.intersection_index = self.get_intersection_index()

    def get_intersection_index(self):
        if self.direction in ["N", "E"]:
            return (3 * self.length) // 4
        else:  # 'S', 'W'
            return self.length // 4

    def step(self):
        light = self.model.traffic_lights[self.direction]
        inflow = self.model.get_inflow(self) if light.state == "green" else 0

        # Handle traffic at the light
        if light.state == "red":
            if self.direction in ["N", "E"]:
                self.queue += self.density[self.intersection_index]
                self.density[self.intersection_index] = 0
                self.density = (
                    [inflow] + self.density[:-2] + [self.density[-2] + self.density[-1]]
                )
            else:  # S and W
                self.queue += self.density[0]
                self.density[0] = 0
                self.density = (
                    [self.density[0] + self.density[1]] + self.density[2:] + [inflow]
                )

        else:  # Green light
            # Move all traffic including the queue
            outflow = min(
                self.density[self.intersection_index] + self.queue, self.max_density
            )
            if self.direction in ["N", "E"]:
                self.density = [inflow] + self.density[:-1]
            else:  # S and W
                self.density = self.density[1:] + [inflow]
            self.density[self.intersection_index] = outflow
            self.queue = max(
                0,
                (self.density[self.intersection_index] + self.queue) - self.max_density,
            )

        self.check_for_crash()

        # Ensure densities stay within bounds
        self.density = [max(0, min(d, self.max_density)) for d in self.density]

    def check_for_crash(self):
        intersection_density = self.density[self.intersection_index]

        if (
            intersection_density > 0
            and self.model.traffic_lights[self.direction].state == "green"
        ):
            perpendicular_roads = (
                ["E", "W"] if self.direction in ["N", "S"] else ["N", "S"]
            )
            for perp_dir in perpendicular_roads:
                perp_road = self.model.roads[perp_dir]
                perp_intersection_density = perp_road.density[
                    perp_road.intersection_index
                ]
                if (
                    perp_intersection_density > 0
                    and self.model.traffic_lights[perp_dir].state == "green"
                ):
                    # Derive crash coordinates from traffic light positions
                    crash_x = self.model.traffic_lights[self.direction].pos[0]
                    crash_y = self.model.traffic_lights[perp_dir].pos[1]
                    crash_cell = CrashCell(f"{self.direction}x{perp_dir}", self.model)
                    self.model.grid.place_agent(crash_cell, (crash_x, crash_y))
                    self.model.schedule.add(
                        crash_cell
                    )  # Add crash_cell to the schedule
                    self.model.crash_count += 1
                    return  # Exit after detecting a crash to avoid double counting


class TrafficModel(mesa.Model):
    def __init__(self, road_length=10, max_density=1, change_frequency=20):
        super().__init__()
        self.road_length = road_length
        self.max_density = max_density
        self.grid = mesa.space.MultiGrid(road_length, road_length, False)
        self.schedule = mesa.time.RandomActivation(self)
        self.change_frequency = change_frequency
        self.steps_since_last_change = 0

        # Create roads (full length of the grid)
        self.roads = {
            key: Road(i, self, key, max_density, road_length)
            for i, key in enumerate(["N", "S", "E", "W"])
        }
        for road in self.roads.values():
            self.schedule.add(road)

        # Place road cells
        cell_id = 4  # Start after road IDs
        for x in range(road_length):
            for y in range(road_length):
                if x == road_length // 2 - 1:  # North-bound lane
                    road = self.roads["S"]
                    index = road_length - 1 - y
                elif x == road_length // 2:  # South-bound lane
                    road = self.roads["N"]
                    index = y
                elif y == road_length // 2 - 1:  # East-bound lane
                    road = self.roads["W"]
                    index = x
                elif y == road_length // 2:  # West-bound lane
                    road = self.roads["E"]
                    index = road_length - 1 - x
                else:
                    continue  # Skip non-road cells

                cell = RoadCell(cell_id, self, road, index)
                self.grid.place_agent(cell, (x, y))
                cell_id += 1

        # Create traffic lights
        self.traffic_lights = {
            direction: TrafficLight(direction, self)
            for direction in ["N", "S", "E", "W"]
        }
        mid = road_length // 2
        quarter = road_length // 4
        light_positions = {
            "N": (mid, mid + quarter - 1),  # Top center
            "S": (mid - 1, mid - quarter),  # Bottom center
            "E": (mid + quarter - 1, mid - 1),  # left center
            "W": (mid - quarter, mid),  # right center
        }
        for direction, light in self.traffic_lights.items():
            self.grid.place_agent(light, light_positions[direction])
            self.schedule.add(light)

        self.crash_count = 0

        self.datacollector = mesa.DataCollector(
            model_reporters={
                "Average Density": lambda m: sum(
                    sum(road.density) / len(road.density) for road in m.roads.values()
                )
                / len(m.roads),
                "Light States": lambda m: {
                    dir: light.state for dir, light in m.traffic_lights.items()
                },
                "N Road Avg Density": lambda m: sum(m.roads["N"].density)
                / len(m.roads["N"].density),
                "S Road Avg Density": lambda m: sum(m.roads["S"].density)
                / len(m.roads["S"].density),
                "E Road Avg Density": lambda m: sum(m.roads["E"].density)
                / len(m.roads["E"].density),
                "W Road Avg Density": lambda m: sum(m.roads["W"].density)
                / len(m.roads["W"].density),
                "N Road Queue": lambda m: m.roads["N"].queue,
                "S Road Queue": lambda m: m.roads["S"].queue,
                "E Road Queue": lambda m: m.roads["E"].queue,
                "W Road Queue": lambda m: m.roads["W"].queue,
                "Crashes": lambda m: m.crash_count,
            }
        )

    def get_inflow(self, road):
        return random.uniform(1e-3, 2e-1)  # Simplified inflow

    def unsafe_traffic_light_control(self):
        for light in self.traffic_lights.values():
            light.change_state(random.choice(["red", "green"]))

    def step(self):
        self.schedule.step()
        self.steps_since_last_change += 1

        if self.steps_since_last_change >= self.change_frequency:
            self.unsafe_traffic_light_control()
            self.steps_since_last_change = 0

        self.datacollector.collect(self)


def get_color_gradient(value, min_value, max_value):
    normalized = (value - min_value) / (max_value - min_value)
    hue = 0.3 * (1 - normalized)
    saturation = 0.7 * normalized
    value = 1 - (0.3 * normalized)
    r, g, b = colorsys.hsv_to_rgb(hue, saturation, value)
    return "#{:02x}{:02x}{:02x}".format(int(r * 255), int(g * 255), int(b * 255))


def agent_portrayal(agent):
    if isinstance(agent, TrafficLight):
        portrayal = {
            "Shape": "rect",
            "w": 0.8,
            "h": 0.8,
            "Layer": 2,
            "Color": "red" if agent.state == "red" else "green",
            "Filled": "true",
        }
    elif isinstance(agent, RoadCell):
        color = get_color_gradient(
            agent.road.density[agent.index], 0, agent.road.max_density
        )
        is_at_light = agent.index == agent.road.intersection_index
        light_is_red = agent.model.traffic_lights[agent.road.direction].state == "red"

        portrayal = {
            "Shape": "arrowHead",
            "scale": 0.6,
            "Layer": 1,
            "Color": color,
            "Filled": "true",
        }

        # Set arrow direction
        if agent.road.direction == "N":
            portrayal["heading_x"] = 0
            portrayal["heading_y"] = -1
        elif agent.road.direction == "S":
            portrayal["heading_x"] = 0
            portrayal["heading_y"] = 1
        elif agent.road.direction == "E":
            portrayal["heading_x"] = 1
            portrayal["heading_y"] = 0
        else:  # West
            portrayal["heading_x"] = -1
            portrayal["heading_y"] = 0

        if is_at_light and light_is_red:
            portrayal["Color"] = "darkred"
            portrayal["scale"] = 0.4

    elif isinstance(agent, CrashCell):
        portrayal = {
            "Shape": "circle",
            "Color": "orange",
            "r": 0.4,
            "Layer": 2,
            "Filled": "true",
        }
    else:
        portrayal = {
            "Shape": "rect",
            "w": 0.8,
            "h": 0.8,
            "Filled": "true",
            "Layer": 0,
            "Color": "#D3D3D3",  # Light gray for non-road cells
        }
    return portrayal
