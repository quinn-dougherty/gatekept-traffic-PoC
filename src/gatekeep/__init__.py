"""Gatekeep, empirically"""

from gatekeep.__main__ import single_step_vid
from gatekeep.traffic_light import specification


def main() -> int:
    """Main entry point of the program."""
    proof_cert = single_step_vid(specification.safety)
    return 0
