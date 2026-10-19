import type { AstNode, AstObjectKey } from './ast'
import { Environment } from './environment'
import { registerStdlib } from './stdlib'
import { bool, number, str, type Closure, type GluValue, type MaybeAsync, type NativeFn, valuesEqual } from './value'

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

  registerNative(name: string, arity: number, fn: NativeFn): void {
    this.#environment.set(name, { tag: 'native', arity, fn })
  }

  async evaluate(node: AstNode): Promise<GluValue> {
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

  async #binding(name: string, expr: AstNode): Promise<GluValue> {
    const value = await this.evaluate(expr)
    this.#environment.set(name, value)
    return value
  }

  async #variableRef(name: string): Promise<GluValue> {
    const value = this.#environment.get(name)
    if (value === undefined) {
      throw new GluEvalError(`Undefined variable '${name}'.`)
    }
    return value
  }

  async #unary(op: string, node: AstNode): Promise<GluValue> {
    const value = await this.evaluate(node)
    if (op === 'not') {
      return bool(!isTruthy(value))
    }
    if (value.tag !== 'number') {
      throw new GluEvalError('Operand must be a number.')
    }
    return number(-value.value)
  }

  async #logical(op: string, left: AstNode, right: AstNode): Promise<GluValue> {
    const lhs = await this.evaluate(left)
    if (op === 'and') {
      return isTruthy(lhs) ? this.evaluate(right) : lhs
    }
    return isTruthy(lhs) ? lhs : this.evaluate(right)
  }

  async #binary(op: string, left: AstNode, right: AstNode): Promise<GluValue> {
    const lhs = await this.evaluate(left)
    const rhs = await this.evaluate(right)
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

  async #list(nodes: AstNode[]): Promise<GluValue> {
    const values: GluValue[] = []
    for (const node of nodes) {
      values.push(await this.evaluate(node))
    }
    return { tag: 'list', items: values }
  }

  async #object(entries: Array<{ key: AstObjectKey; value: AstNode }>): Promise<GluValue> {
    const fields = new Map<string, GluValue>()
    for (const entry of entries) {
      let key: string
      if (entry.key.tag === 'literal') {
        key = entry.key.key
      } else {
        const keyValue = await this.evaluate(entry.key.expr)
        if (keyValue.tag !== 'string') {
          throw new GluEvalError('Object keys must be strings.')
        }
        key = keyValue.value
      }
      fields.set(key, await this.evaluate(entry.value))
    }
    return { tag: 'object', fields }
  }

  async #index(target: AstNode, index: AstNode): Promise<GluValue> {
    const targetValue = await this.evaluate(target)
    const indexValue = await this.evaluate(index)
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

  async #property(target: AstNode, name: string): Promise<GluValue> {
    const targetValue = await this.evaluate(target)
    if (targetValue.tag !== 'object') {
      throw new GluEvalError('Only objects have properties.')
    }
    const field = targetValue.fields.get(name)
    if (field === undefined) {
      throw new GluEvalError(`Undefined property '${name}'.`)
    }
    return field
  }

  async #block(nodes: AstNode[]): Promise<GluValue> {
    const enclosing = this.#environment
    this.#environment = new Environment(enclosing)
    let result: GluValue = { tag: 'nil' }
    try {
      for (const node of nodes) {
        result = await this.evaluate(node)
      }
    } finally {
      this.#environment = enclosing
    }
    return result
  }

  async #ifExpression(condition: AstNode, thenBranch: AstNode, elseBranch: AstNode | null): Promise<GluValue> {
    if (isTruthy(await this.evaluate(condition))) {
      return this.evaluate(thenBranch)
    }
    return elseBranch !== null ? this.evaluate(elseBranch) : { tag: 'nil' }
  }

  #closure(node: { params: string[]; body: AstNode }): GluValue {
    return { tag: 'closure', closure: { node, env: this.#environment } }
  }

  async #call(callee: AstNode, args: AstNode[]): Promise<GluValue> {
    const calleeValue = await this.evaluate(callee)
    const argValues: GluValue[] = []
    for (const arg of args) {
      argValues.push(await this.evaluate(arg))
    }
    switch (calleeValue.tag) {
      case 'closure':
        return this.callClosure(calleeValue.closure, argValues)
      case 'native':
        return this.callNative(calleeValue, argValues)
    }
    throw new GluEvalError('Only closures and native functions can be called.')
  }

  async callClosure(closure: Closure, args: GluValue[]): Promise<GluValue> {
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
      return await this.evaluate(closure.node.body)
    } finally {
      this.#callDepth--
      this.#environment = savedEnv
    }
  }

  async callNative(native: { fn: NativeFn; arity: number }, args: GluValue[]): Promise<GluValue> {
    if (native.arity !== args.length) {
      throw new GluEvalError(`Expected ${native.arity} argument(s) but got ${args.length}.`)
    }
    return native.fn(this, args)
  }
}

export type { MaybeAsync }
