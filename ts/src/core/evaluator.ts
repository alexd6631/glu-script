import type { AstNode, AstObjectKey } from './ast'
import { Environment } from './environment'
import { registerStdlib } from './stdlib'
import { bool, number, str, type Closure, type GluValue, valuesEqual } from './value'

export const MAX_CALL_DEPTH = 256

export class GluEvalError extends Error {
  constructor(message: string) {
    super(message)
    this.name = 'GluEvalError'
  }
}

export function isTruthy(value: GluValue): boolean {
  return !(value.tag === 'nil' || (value.tag === 'bool' && !value.value))
}

function checkNumber(value: GluValue): number {
  if (value.tag === 'number') {
    return value.value
  }
  throw new GluEvalError('Operands must be numbers.')
}

export class Evaluator {
  #environment: Environment
  #callDepth = 0

  constructor() {
    this.#environment = new Environment()
    registerStdlib(this)
  }

  registerNativeFunction(name: string, fn: GluValue): void {
    this.#environment.set(name, fn)
  }

  evaluate(node: AstNode): GluValue {
    switch (node.tag) {
      case 'literal':
        return node.value
      case 'list':
        return this.#list(node.items)
      case 'object':
        return this.#object(node.entries)
      case 'block':
        return this.#block(node.items)
      case 'binary':
        return this.#binary(node.op, node.left, node.right)
      case 'logical':
        return this.#logical(node.op, node.left, node.right)
      case 'index':
        return this.#index(node.target, node.index)
      case 'property':
        return this.#property(node.target, node.name)
      case 'unary':
        return this.#unary(node.op, node.operand)
      case 'binding':
        return this.#binding(node.name, node.expr)
      case 'variableRef':
        return this.#variableRef(node.name)
      case 'if':
        return this.#ifExpression(node.node.condition, node.node.thenBranch, node.node.elseBranch)
      case 'closure':
        return this.#closure(node.node)
      case 'call':
        return this.#call(node.callee, node.args)
      case 'error':
        throw new GluEvalError('Invalid expression.')
    }
  }

  #binding(name: string, expr: AstNode): GluValue {
    const value = this.evaluate(expr)
    this.#environment.set(name, value)
    return value
  }

  #variableRef(name: string): GluValue {
    const value = this.#environment.get(name)
    if (value === undefined) {
      throw new GluEvalError(`Undefined variable '${name}'.`)
    }
    return value
  }

  #unary(op: string, node: AstNode): GluValue {
    const value = this.evaluate(node)
    if (op === 'not') {
      return bool(!isTruthy(value))
    }
    if (value.tag !== 'number') {
      throw new GluEvalError('Operand must be a number.')
    }
    return number(-value.value)
  }

  #logical(op: string, left: AstNode, right: AstNode): GluValue {
    const lhs = this.evaluate(left)
    if (op === 'and') {
      return isTruthy(lhs) ? this.evaluate(right) : lhs
    }
    return isTruthy(lhs) ? lhs : this.evaluate(right)
  }

  #binary(op: string, left: AstNode, right: AstNode): GluValue {
    const lhs = this.evaluate(left)
    const rhs = this.evaluate(right)
    switch (op) {
      case 'equal':
        return bool(valuesEqual(lhs, rhs))
      case 'notEqual':
        return bool(!valuesEqual(lhs, rhs))
      case 'add':
        if (lhs.tag === 'number' && rhs.tag === 'number') {
          return number(lhs.value + rhs.value)
        }
        if (lhs.tag === 'string' && rhs.tag === 'string') {
          return str(lhs.value + rhs.value)
        }
        throw new GluEvalError('Operands must be two numbers or two strings.')
      case 'sub':
        return number(checkNumber(lhs) - checkNumber(rhs))
      case 'mul':
        return number(checkNumber(lhs) * checkNumber(rhs))
      case 'div':
        return number(checkNumber(lhs) / checkNumber(rhs))
      case 'mod':
        return number(checkNumber(lhs) % checkNumber(rhs))
      case 'less':
        return bool(checkNumber(lhs) < checkNumber(rhs))
      case 'lessEqual':
        return bool(checkNumber(lhs) <= checkNumber(rhs))
      case 'greater':
        return bool(checkNumber(lhs) > checkNumber(rhs))
      case 'greaterEqual':
        return bool(checkNumber(lhs) >= checkNumber(rhs))
    }
    throw new GluEvalError(`Unknown binary operator '${op}'.`)
  }

  #list(nodes: AstNode[]): GluValue {
    const values = nodes.map((node) => this.evaluate(node))
    return { tag: 'list', items: values }
  }

  #object(entries: Array<{ key: AstObjectKey; value: AstNode }>): GluValue {
    const fields = new Map<string, GluValue>()
    for (const entry of entries) {
      let key: string
      if (entry.key.tag === 'literal') {
        key = entry.key.key
      } else {
        const keyValue = this.evaluate(entry.key.expr)
        if (keyValue.tag !== 'string') {
          throw new GluEvalError('Object keys must be strings.')
        }
        key = keyValue.value
      }
      fields.set(key, this.evaluate(entry.value))
    }
    return { tag: 'object', fields }
  }

  #index(target: AstNode, index: AstNode): GluValue {
    const targetValue = this.evaluate(target)
    const indexValue = this.evaluate(index)
    if (indexValue.tag !== 'number') {
      throw new GluEvalError('Index must be a number.')
    }
    const n = indexValue.value
    if (!(n >= 0 && Number.isInteger(n))) {
      throw new GluEvalError('Index must be a non-negative integer.')
    }
    const idx = n
    switch (targetValue.tag) {
      case 'list': {
        const item = targetValue.items[idx]
        if (item === undefined) {
          throw new GluEvalError('Index out of range.')
        }
        return item
      }
      case 'string': {
        const c = targetValue.value[idx]
        if (c === undefined) {
          throw new GluEvalError('Index out of range.')
        }
        return str(c)
      }
    }
    throw new GluEvalError('Only lists and strings can be indexed.')
  }

  #property(target: AstNode, name: string): GluValue {
    const targetValue = this.evaluate(target)
    if (targetValue.tag !== 'object') {
      throw new GluEvalError('Only objects have properties.')
    }
    const field = targetValue.fields.get(name)
    if (field === undefined) {
      throw new GluEvalError(`Undefined property '${name}'.`)
    }
    return field
  }

  #block(nodes: AstNode[]): GluValue {
    const enclosing = this.#environment
    this.#environment = new Environment(enclosing)
    let result: GluValue = { tag: 'nil' }
    try {
      for (const node of nodes) {
        result = this.evaluate(node)
      }
    } finally {
      this.#environment = enclosing
    }
    return result
  }

  #ifExpression(condition: AstNode, thenBranch: AstNode, elseBranch: AstNode | null): GluValue {
    if (isTruthy(this.evaluate(condition))) {
      return this.evaluate(thenBranch)
    }
    return elseBranch !== null ? this.evaluate(elseBranch) : { tag: 'nil' }
  }

  #closure(node: { params: string[]; body: AstNode }): GluValue {
    return { tag: 'closure', closure: { node, env: this.#environment } }
  }

  #call(callee: AstNode, args: AstNode[]): GluValue {
    const calleeValue = this.evaluate(callee)
    const argValues = args.map((arg) => this.evaluate(arg))
    switch (calleeValue.tag) {
      case 'closure':
        return this.callClosure(calleeValue.closure, argValues)
      case 'native':
        return this.callNative(calleeValue, argValues)
    }
    throw new GluEvalError('Only closures and native functions can be called.')
  }

  callClosure(closure: Closure, args: GluValue[]): GluValue {
    if (closure.node.params.length !== args.length) {
      throw new GluEvalError(`Expected ${closure.node.params.length} argument(s) but got ${args.length}.`)
    }
    const callEnv = new Environment(closure.env)
    for (let i = 0; i < closure.node.params.length; i++) {
      callEnv.set(closure.node.params[i], args[i])
    }
    if (this.#callDepth >= MAX_CALL_DEPTH) {
      throw new GluEvalError(`Recursion limit of ${MAX_CALL_DEPTH} exceeded.`)
    }
    const savedEnv = this.#environment
    this.#environment = callEnv
    this.#callDepth++
    try {
      return this.evaluate(closure.node.body)
    } finally {
      this.#callDepth--
      this.#environment = savedEnv
    }
  }

  callNative(
    native: { fn: (evaluator: Evaluator, args: GluValue[]) => GluValue; arity: number },
    args: GluValue[],
  ): GluValue {
    if (native.arity !== args.length) {
      throw new GluEvalError(`Expected ${native.arity} argument(s) but got ${args.length}.`)
    }
    return native.fn(this, args)
  }
}
