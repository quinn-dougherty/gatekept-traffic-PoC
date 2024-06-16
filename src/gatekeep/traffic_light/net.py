import sumolib  

def create_four_way_intersection(net):
    # Define the coordinates for the intersection center
    center = (0, 0)
    
    # Create nodes at the ends of each road
    net.addNode('n1', -100, 0)
    net.addNode('n2', 100, 0)
    net.addNode('n3', 0, 100)
    net.addNode('n4', 0, -100)
    
    # Create nodes at the intersection
    net.addNode('nc1', -50, 0)
    net.addNode('nc2', 50, 0)
    net.addNode('nc3', 0, 50)
    net.addNode('nc4', 0, -50)
    
    # Connect the nodes with edges (roads)
    net.addEdge('e1', 'n1', 'nc1')
    net.addEdge('e2', 'nc1', 'nc2')
    net.addEdge('e3', 'nc2', 'n2')
    net.addEdge('e4', 'n3', 'nc3')
    net.addEdge('e5', 'nc3', 'nc4')
    net.addEdge('e6', 'nc4', 'n4')
    
    # Define traffic lights at the intersection
    tl_logic = sumolib.net.TrafficLightLogic("tl1", type="static", programID="1")
    tl_logic.addPhase(50, "rrrrGGGG")
    tl_logic.addPhase(6, "rrrryyyy")
    tl_logic.addPhase(50, "GGGGrrrr")
    tl_logic.addPhase(6, "yyyyrrrr")
    net.addTrafficLight('tl1', tl_logic)
    
    # Set traffic light positions
    net.setTrafficLightPosition('tl1', center[0], center[1])
    
    return net

# Example usage
net = sumolib.net.Net()
create_four_way_intersection(net)
