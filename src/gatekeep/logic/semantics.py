import torch


def always_op(prop_values):
    return torch.min(prop_values)


def eventually_op(prop_values):
    return torch.max(prop_values)


def and_op(prop_values_list):
    return torch.min(
        torch.stack(
            [torch.min(torch.stack(prop_values)) for prop_values in prop_values_list]
        )
    )


def or_op(prop_values_list):
    return torch.max(
        torch.stack(
            [torch.max(torch.stack(prop_values)) for prop_values in prop_values_list]
        )
    )


def not_op(prop_values):
    return 1.0 - prop_values
