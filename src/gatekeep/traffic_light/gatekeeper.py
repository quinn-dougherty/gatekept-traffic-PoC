import dask.bag as db
from dask.distributed import Client
from dask.diagnostics import ProgressBar
from gatekeep.traffic_light.observe_trajectories import (
    simulate_trajectories,
    compile_spec,
)


class Gatekeeper:
    def __init__(
        self,
        sim,
        spec,
        controller,
        num_trajectories=int(1e1),
        num_steps=int(1e1),
        epsilon=1e-6,
    ):
        self.sim = sim
        self.spec = spec
        self.controller = controller
        self.num_trajectories = num_trajectories
        self.num_steps = num_steps
        self.epsilon = epsilon

    def run_step(self):
        generate_proof_cert = compile_spec(self.spec, self.sim)
        action = self.controller.select_action(state=None)
        trajectories = simulate_trajectories(
            self.sim,
            action,
            num_trajectories=self.num_trajectories,
            num_steps=self.num_steps,
        )
        proof_cert = generate_proof_cert(action)
        proof_cert["action"] = action
        next_state, reward, done, _ = self.sim.step(action)
        return next_state, reward, done, proof_cert, trajectories

    def loop(self):

        def run_simulation(num_iterations):
            proof_certs = []
            all_trajectories = []

            for _ in range(num_iterations):
                state, world_reward, world_done, info = self.sim.world.step(
                    self.controller.select_action(state=None)
                )
                next_state, sim_reward, sim_done, proof_cert, trajectories = (
                    self.run_step()
                )
                self.sim.render()
                if proof_cert is not None:
                    proof_certs.append(proof_cert)
                    all_trajectories.extend(trajectories)

                if world_done:
                    break

            return proof_certs, all_trajectories

        def parallel_loop(num_iterations, num_partitions):
            with ProgressBar():
                # Create a Dask client
                client = Client()

                # Create a Dask bag with the iteration range
                iterations = db.from_sequence(
                    [num_iterations] * num_iterations, npartitions=num_partitions
                )

                # Map the run_simulation function to each iteration
                results = iterations.map(run_simulation).compute()

                # Aggregate the results
                proof_certs = [cert for sublist in results for cert in sublist[0]]
                all_trajectories = [traj for sublist in results for traj in sublist[1]]

                # Close the Dask client
                client.close()

            return proof_certs, all_trajectories

        num_iterations = 10
        num_partitions = 5
        return parallel_loop(num_iterations, num_partitions)

    def render_and_save_video(self):
        # self.env.render()
        self.sim.close()
