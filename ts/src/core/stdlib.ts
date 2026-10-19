import { Evaluator, isTruthy } from './evaluator'
import type { Closure, GluValue, NativeFn } from './value'

interface ListAndClosure {
  list: GluValue[]
  closure: Closure
}

function requireListAndClosure(args: GluValue[]): ListAndClosure {
  if (args[0].tag !== 'list') {
    throw new Error('Expected array')
  }
  if (args[1].tag !== 'closure') {
    throw new Error('Expected closure')
  }
  return { list: args[0].items, closure: args[1].closure }
}

export function registerStdlib(evaluator: Evaluator): void {
  evaluator.registerNativeFunction('hello', {
    tag: 'native',
    arity: 0,
    fn: () => ({ tag: 'string', value: 'hello' }),
  })

  evaluator.registerNativeFunction('map', {
    tag: 'native',
    arity: 2,
    fn: async (evaluator, args) => {
      const { list, closure } = requireListAndClosure(args)
      const items: GluValue[] = []
      for (const item of list) {
        items.push(await evaluator.callClosure(closure, [item]))
      }
      return { tag: 'list', items }
    },
  })

  evaluator.registerNativeFunction('filter', {
    tag: 'native',
    arity: 2,
    fn: async (evaluator, args) => {
      const { list, closure } = requireListAndClosure(args)
      const items: GluValue[] = []
      for (const item of list) {
        if (isTruthy(await evaluator.callClosure(closure, [item]))) {
          items.push(item)
        }
      }
      return { tag: 'list', items }
    },
  })

  evaluator.registerNativeFunction('fold', {
    tag: 'native',
    arity: 3,
    fn: async (evaluator, args) => {
      if (args[0].tag !== 'list') {
        throw new Error('Expected array')
      }
      if (args[2].tag !== 'closure') {
        throw new Error('Expected closure')
      }
      let acc = args[1]
      for (const item of args[0].items) {
        acc = await evaluator.callClosure(args[2].closure, [acc, item])
      }
      return acc
    },
  })

  evaluator.registerNativeFunction('reduce', {
    tag: 'native',
    arity: 2,
    fn: async (evaluator, args) => {
      const { list, closure } = requireListAndClosure(args)
      const iter = list[Symbol.iterator]()
      const first = iter.next()
      if (first.done) {
        throw new Error('Cannot reduce an empty array.')
      }
      let acc = first.value
      for (const item of iter) {
        acc = await evaluator.callClosure(closure, [acc, item])
      }
      return acc
    },
  })

  evaluator.registerNativeFunction('any', {
    tag: 'native',
    arity: 2,
    fn: async (evaluator, args) => {
      const { list, closure } = requireListAndClosure(args)
      for (const item of list) {
        if (isTruthy(await evaluator.callClosure(closure, [item]))) {
          return { tag: 'bool', value: true }
        }
      }
      return { tag: 'bool', value: false }
    },
  })

  evaluator.registerNativeFunction('every', {
    tag: 'native',
    arity: 2,
    fn: async (evaluator, args) => {
      const { list, closure } = requireListAndClosure(args)
      for (const item of list) {
        if (!isTruthy(await evaluator.callClosure(closure, [item]))) {
          return { tag: 'bool', value: false }
        }
      }
      return { tag: 'bool', value: true }
    },
  })

  evaluator.registerNativeFunction('find', {
    tag: 'native',
    arity: 2,
    fn: async (evaluator, args) => {
      const { list, closure } = requireListAndClosure(args)
      for (const item of list) {
        if (isTruthy(await evaluator.callClosure(closure, [item]))) {
          return item
        }
      }
      return { tag: 'nil' }
    },
  })
}

export type { NativeFn }
