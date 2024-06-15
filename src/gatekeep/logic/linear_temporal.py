class Proposition:
    def __init__(self, name):
        self.name = name

    def __str__(self):
        return self.name

    def __repr__(self):
        return f"Proposition({self.name})"

    def __and__(self, other):
        return And(self, other)

    def __or__(self, other):
        return Or(self, other)

    def __invert__(self):
        return Not(self)

    def __eq__(self, other):
        return self.name == other.name


class Tru(Proposition):
    def __init__(self):
        pass

    def __str__(self):
        return "T"

    def __repr__(self):
        return "Tru"


class Not(Proposition):
    def __init__(self, prop):
        self.prop = prop

    def __str__(self):
        return f"(! {self.prop})"

    def __repr__(self):
        return f"Not({repr(self.prop)})"


class And(Proposition):
    def __init__(self, *props):
        self.props = props

    def __str__(self):
        return f"({' & '.join(str(prop) for prop in self.props)})"

    def __repr__(self):
        return f"And({', '.join(repr(prop) for prop in self.props)})"


class Or(Proposition):
    def __init__(self, *props):
        self.props = props

    def __str__(self):
        return f"({' | '.join(str(prop) for prop in self.props)})"

    def __repr__(self):
        return f"Or({', '.join(repr(prop) for prop in self.props)})"


class Always(Proposition):
    def __init__(self, prop):
        self.prop = prop

    def __str__(self):
        return f"(◻{self.prop})"

    def __repr__(self):
        return f"Always({repr(self.prop)})"


class Eventually(Proposition):
    def __init__(self, prop):
        self.prop = prop

    def __str__(self):
        return f"(◇{self.prop})"

    def __repr__(self):
        return f"Eventually({repr(self.prop)})"


class Until(Proposition):
    def __init__(self, prop1, prop2):
        self.prop1 = prop1
        self.prop2 = prop2

    def __str__(self):
        return f"{self.prop1} U {self.prop2}"

    def __repr__(self):
        return f"Until({self.prop1}, {self.prop2})"


def Implies(a, b):
    return Or(Not(a), b)


def Iff(a, b):
    return And(Implies(a, b), Implies(b, a))
