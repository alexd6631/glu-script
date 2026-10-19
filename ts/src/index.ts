import { Compiler } from './core/compiler'
import { Evaluator } from './core/evaluator'
import { Scanner, type Token } from './core/scanner'
import type { AstNode } from './core/ast'
import type { GluValue } from './core/value'

export type { AstNode } from './core/ast'
export type { GluValue, NativeFn, MaybeAsync } from './core/value'
export type { Token } from './core/scanner'
export { GluEvalError, Evaluator, MAX_CALL_DEPTH } from './core/evaluator'
export { Scanner } from './core/scanner'
export { Compiler } from './core/compiler'
export { display as displayValue } from './core/value'

export class GluParseError extends Error {
  constructor(message: string) {
    super(message)
    this.name = 'GluParseError'
  }
}

export function scan(source: string): Token[] {
  return new Scanner(source).toArray()
}

export function parse(source: string): AstNode {
  const compiler = new Compiler(source)
  const ast = compiler.compile()
  if (ast === null) {
    throw new GluParseError('failed to parse expression')
  }
  return ast
}

export async function evaluate(source: string): Promise<unknown> {
  return toJs(await evaluateValue(source))
}

export async function evaluateValue(source: string): Promise<GluValue> {
  const ast = parse(source)
  return new Evaluator().evaluate(ast)
}

export async function evaluateWith(evaluator: Evaluator, source: string): Promise<unknown> {
  return toJs(await evaluator.evaluate(parse(source)))
}

export function toJs(value: GluValue): unknown {
  switch (value.tag) {
    case 'nil':
      return undefined
    case 'bool':
      return value.value
    case 'number':
      return value.value
    case 'string':
      return value.value
    case 'list':
      return value.items.map(toJs)
    case 'object': {
      const obj: Record<string, unknown> = {}
      for (const [key, v] of value.fields) {
        obj[key] = toJs(v)
      }
      return obj
    }
    case 'closure':
      return '<closure>'
    case 'native':
      return '<native_fn>'
  }
}
