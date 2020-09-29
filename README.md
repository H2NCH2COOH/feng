# [风] Feng - Logic Expression Language

Feng is a logic expression language that is designed to be:
- General yet limited
- LISP like syntax and functions
- Expression only, no execution
- With a tool to __expand__ and __reduce__ the written expressions into a more compact one that should be read by other programs

## Design

Feng is designed initially for one purpose: To express ever chaning logic without the need to change what's already there.
To reach the above goal, an assumption is made:

Any change in the logic can be expressed into two parts:

__A precondition that can be evaluated without side-effects, and an action that causes all the side-effects.__

Any changes are thus made by adding the new precondition to all the existing preconditions, and then the combined logic can be evaluated within by a runtime and an action that should be executed is given.

This way, if we can express the preconditions in an independent language, we can then change the logic by simply appending new expressions.
