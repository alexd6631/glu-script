import type { ClosureNode } from './ast'
import type { Environment } from './environment'
import type { Evaluator } from './evaluator'

export type GluValue =
  | { tag: 'nil' }
  | { tag: 'bool'; value: boolean }
  | { tag: 'number'; value: number }
  | { tag: 'string'; value: string }
  | { tag: 'list'; items: GluValue[] }
  | { tag: 'object'; fields: Map<string, GluValue> }
  | { tag: 'closure'; closure: Closure }
  | { tag: 'native'; fn: NativeFn; arity: number }

export interface Closure {
  node: ClosureNode
  env: Environment
}

export type NativeFn = (evaluator: Evaluator, args: GluValue[]) => GluValue

export const nil: GluValue = { tag: 'nil' }
export const bool = (value: boolean): GluValue => ({ tag: 'bool', value })
export const number = (value: number): GluValue => ({ tag: 'number', value })
export const str = (value: string): GluValue => ({ tag: 'string', value })
export const list = (items: GluValue[]): GluValue => ({ tag: 'list', items })
export const object = (fields: Map<string, GluValue>): GluValue => ({ tag: 'object', fields })

export function valuesEqual(a: GluValue, b: GluValue): boolean {
  if (a.tag !== b.tag) {
    return false
  }
  switch (a.tag) {
    case 'nil':
      return true
    case 'bool':
    case 'number':
    case 'string':
      return a.value === (b as typeof a).value
    case 'list':
      return (
        a.items.length === (b as typeof a).items.length &&
        a.items.every((item, i) => valuesEqual(item, (b as typeof a).items[i]))
      )
    case 'object': {
      const other = b as typeof a
      if (a.fields.size !== other.fields.size) {
        return false
      }
      for (const [key, value] of a.fields) {
        if (!other.fields.has(key) || !valuesEqual(value, other.fields.get(key)!)) {
          return false
        }
      }
      return true
    }
    case 'closure':
      return a.closure.node === (b as typeof a).closure.node && a.closure.env === (b as typeof a).closure.env
    case 'native':
      return a.fn === (b as typeof a).fn && a.arity === (b as typeof a).arity
  }
}

export function display(value: GluValue): string {
  switch (value.tag) {
    case 'nil':
      return 'nil'
    case 'bool':
      return `${value.value}`
    case 'number':
      return `${value.value}`
    case 'string':
      return `"${value.value}"`
    case 'list':
      return `[${value.items.map(display).join(', ')}]`
    case 'object':
      return `{${[...value.fields].map(([key, v]) => `"${key}": ${display(v)}`).join(', ')}}`
    case 'closure':
      return '<closure>'
    case 'native':
      return '<native_fn>'
  }
}
