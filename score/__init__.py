"""Readability metrics for comparing rewrite candidates."""

from collections import Counter
from collections.abc import Mapping, Sequence

import libcst as cst
from libcst.metadata import ExpressionContext, ExpressionContextProvider, MetadataWrapper


def count_iterations(node: cst.BaseExpression) -> int:
    match node:
        case cst.Tuple(elements=elements) | cst.List(elements=elements) if all(
            isinstance(element, cst.Element) for element in elements
        ):
            return len(elements)
        case cst.SimpleString(evaluated_value=value) if isinstance(value, (str, bytes)):
            return len(value)
        case _:
            raise ValueError('Iteration count requires a literal tuple, list, or string')


def score_targets(
    targets: Sequence[cst.CSTNode],
    contexts: Mapping[cst.CSTNode, ExpressionContext],
    repetitions: int,
) -> Counter[str]:
    scores = sum((score_node(target, contexts, repetitions) for target in targets), Counter())
    if scores:
        scores[next(iter(scores))] += repetitions
    return scores


def score_node(
    node: cst.CSTNode, contexts: Mapping[cst.CSTNode, ExpressionContext], repetitions: int
) -> Counter[str]:
    match node:
        case cst.GeneratorExp() | cst.ListComp() | cst.SetComp() | cst.DictComp():
            scores: Counter[str] = Counter()
            generator = node.for_in
            while generator is not None:
                if generator.ifs or generator.asynchronous:
                    raise ValueError('Filtered and asynchronous comprehensions are unsupported')
                scores.update(score_node(generator.iter, contexts, repetitions))
                repetitions *= count_iterations(generator.iter)
                scores.update(score_targets((generator.target,), contexts, repetitions))
                generator = generator.inner_for_in
            expressions = (node.key, node.value) if isinstance(node, cst.DictComp) else (node.elt,)
            for expression in expressions:
                scores.update(score_node(expression, contexts, repetitions))
            return +scores
        case cst.Assign():
            return score_targets(
                tuple(target.target for target in node.targets), contexts, repetitions
            ) + score_node(node.value, contexts, repetitions)
        case cst.AnnAssign() | cst.AugAssign() | cst.NamedExpr():
            return (
                score_targets((node.target,), contexts, repetitions)
                + score_node(node.value, contexts, repetitions)
                if node.value is not None
                else Counter()
            )
        case cst.Name() | cst.Attribute() | cst.Subscript() if (
            contexts.get(node) == ExpressionContext.STORE
        ):
            name = node.value if isinstance(node, cst.Name) else cst.Module([]).code_for_node(node)
            return Counter({name: repetitions}) + sum(
                (score_node(child, contexts, repetitions) for child in node.children), Counter()
            )
        case cst.BaseCompoundStatement() | cst.IfExp() | cst.BooleanOperation() | cst.Lambda():
            raise ValueError(f'Unsupported control flow: {type(node).__name__}')
        case cst.BaseSmallStatement() if not isinstance(
            node, (cst.Assign, cst.AugAssign, cst.Expr, cst.Pass)
        ):
            raise ValueError(f'Unsupported statement: {type(node).__name__}')
        case _:
            return sum(
                (score_node(child, contexts, repetitions) for child in node.children), Counter()
            )


def score_assignments(source: str) -> Counter[str]:
    """Score each target one point, charging the first target for the shared right-hand side.

    Targets are ordered left to right, outer to inner. Comprehension iterations each
    incur the same cost. Scores combine identical target text across scopes; their sum
    is the total metric. Assume successful execution and full generator consumption.
    Unsupported control flow and unknown iteration counts raise ValueError without
    executing the source.
    """
    wrapper = MetadataWrapper(cst.parse_module(source))
    return score_node(wrapper.module, wrapper.resolve(ExpressionContextProvider), 1)
