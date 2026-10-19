import test from 'ava'
import { Evaluator, evaluate, evaluateValue, parse, scan } from '../src/index'

async function throwsMessage(source: string): Promise<string> {
  try {
    await evaluateValue(source)
  } catch (e) {
    return (e as Error).message
  }
  throw new Error(`expected '${source}' to throw`)
}

test('scanner: empty source yields eof', async (t) => {
  t.deepEqual(scan(''), [{ kind: 'eof', lexeme: '', line: 1 }])
})

test('scanner: simple lexemes', async (t) => {
  t.deepEqual(
    scan(' ( + . * )').map((tok) => [tok.kind, tok.lexeme]),
    [
      ['leftParen', '('],
      ['plus', '+'],
      ['dot', '.'],
      ['star', '*'],
      ['rightParen', ')'],
      ['eof', ''],
    ],
  )
})

test('scanner: two char lexemes', async (t) => {
  t.deepEqual(
    scan('! != =').map((tok) => [tok.kind, tok.lexeme]),
    [
      ['bang', '!'],
      ['bangEqual', '!='],
      ['equal', '='],
      ['eof', ''],
    ],
  )
})

test('scanner: newlines and comments', async (t) => {
  const tokens = scan('() // comment\n!= // comment\n.')
  t.deepEqual(
    tokens.map((tok) => [tok.kind, tok.line]),
    [
      ['leftParen', 1],
      ['rightParen', 1],
      ['bangEqual', 2],
      ['dot', 3],
      ['eof', 3],
    ],
  )
})

test('scanner: numbers', async (t) => {
  t.deepEqual(
    scan('42,42.4,42.').map((tok) => [tok.kind, tok.lexeme]),
    [
      ['number', '42'],
      ['comma', ','],
      ['number', '42.4'],
      ['comma', ','],
      ['number', '42'],
      ['dot', '.'],
      ['eof', ''],
    ],
  )
})

test('parse: operator precedence', async (t) => {
  t.deepEqual(parse('1 + 1 - 2 * 3 + 2/5'), {
    tag: 'binary',
    op: 'add',
    left: {
      tag: 'binary',
      op: 'sub',
      left: {
        tag: 'binary',
        op: 'add',
        left: { tag: 'literal', value: { tag: 'number', value: 1 } },
        right: { tag: 'literal', value: { tag: 'number', value: 1 } },
      },
      right: {
        tag: 'binary',
        op: 'mul',
        left: { tag: 'literal', value: { tag: 'number', value: 2 } },
        right: { tag: 'literal', value: { tag: 'number', value: 3 } },
      },
    },
    right: {
      tag: 'binary',
      op: 'div',
      left: { tag: 'literal', value: { tag: 'number', value: 2 } },
      right: { tag: 'literal', value: { tag: 'number', value: 5 } },
    },
  })
})

test('parse: unary', async (t) => {
  t.deepEqual(parse('-1'), {
    tag: 'unary',
    op: 'negate',
    operand: { tag: 'literal', value: { tag: 'number', value: 1 } },
  })
})

test('evaluator: binding and reference', async (t) => {
  t.is(await evaluate('{ let x = 1 + 2; x }'), 3)
})

test('evaluator: binding returns bound value', async (t) => {
  t.is(await evaluate('{ let x = 5 }'), 5)
})

test('evaluator: nested block shadowing', async (t) => {
  t.is(await evaluate('{ let x = 1; { let x = 2; x } + x }'), 3)
})

test('evaluator: undefined variable', async (t) => {
  t.is(await throwsMessage('foo'), "Undefined variable 'foo'.")
})

test('evaluator: block scopes are popped', async (t) => {
  t.is(await throwsMessage('{ { let x = 1; 2 }; { x } }'), "Undefined variable 'x'.")
})

test('evaluator: list indexed access', async (t) => {
  t.is(await evaluate('[10, 20, 30][1]'), 20)
})

test('evaluator: string indexed access', async (t) => {
  t.is(await evaluate('"hello"[1]'), 'e')
})

test('evaluator: chained indexed access', async (t) => {
  t.is(await evaluate('[[1, 2], [3, 4]][1][0]'), 3)
})

test('evaluator: index out of range', async (t) => {
  t.is(await throwsMessage('[1, 2][5]'), 'Index out of range.')
})

test('evaluator: fractional index', async (t) => {
  t.is(await throwsMessage('[1, 2][0.5]'), 'Index must be a non-negative integer.')
})

test('evaluator: non-number index', async (t) => {
  t.is(await throwsMessage('[1, 2]["a"]'), 'Index must be a number.')
})

test('evaluator: non-indexable target', async (t) => {
  t.is(await throwsMessage('1[0]'), 'Only lists and strings can be indexed.')
})

test('evaluator: object property access', async (t) => {
  t.is(await evaluate('{a: 1}.a'), 1)
})

test('evaluator: chained property access', async (t) => {
  t.is(await evaluate('{a: {b: "x"}}.a.b'), 'x')
})

test('evaluator: missing property', async (t) => {
  t.is(await throwsMessage('{a: 1}.b'), "Undefined property 'b'.")
})

test('evaluator: non-object property target', async (t) => {
  t.is(await throwsMessage('[1, 2].a'), 'Only objects have properties.')
})

test('evaluator: if/else', async (t) => {
  t.is(await evaluate('if (true) 1 else 2'), 1)
  t.is(await evaluate('if (false) 1 else 2'), 2)
  t.is(await evaluate('if (false) 1'), undefined)
  t.is(await evaluate('if (false) 1 else if (true) 2 else 3'), 2)
})

test('evaluator: if truthiness', async (t) => {
  t.is(await evaluate('if (1) "yes" else "no"'), 'yes')
  t.is(await evaluate('if (nil) 1 else 2'), 2)
})

test('evaluator: if with blocks', async (t) => {
  t.is(await evaluate('if ({ let x = 5; x > 3 }) { let y = 10; y * 2 } else { 0 }'), 20)
})

test('evaluator: closures capture environment', async (t) => {
  t.is(await evaluate('{ let z = 10; let f = |x| x + z; f(5) }'), 15)
})

test('evaluator: call closure basics', async (t) => {
  t.is(await evaluate('(|x| x + 1)(5)'), 6)
  t.is(await evaluate('(|x, y| x + y)(3, 4)'), 7)
  t.is(await evaluate('(|| 42)()'), 42)
})

test('evaluator: call wrong arg count', async (t) => {
  t.is(await throwsMessage('(|x, y| x + y)(1)'), 'Expected 2 argument(s) but got 1.')
  t.is(await throwsMessage('(|x| x)(1, 2)'), 'Expected 1 argument(s) but got 2.')
})

test('evaluator: call non-closure', async (t) => {
  t.is(await throwsMessage('(1)(2)'), 'Only closures and native functions can be called.')
})

test('evaluator: chained closures', async (t) => {
  t.is(await evaluate('(|x| |y| x + y)(3)(4)'), 7)
})

test('evaluator: recursion depth limit', async (t) => {
  t.is(await throwsMessage('{ let f = |n| f(n + 1); f(0) }'), 'Recursion limit of 256 exceeded.')
})

test('evaluator: string concat requires strings', async (t) => {
  t.is(await throwsMessage('1 + "a"'), 'Operands must be two numbers or two strings.')
})

test('evaluator: equality', async (t) => {
  t.is(await evaluate('1 == 1'), true)
  t.is(await evaluate('1 != 2'), true)
  t.is(await evaluate('"a" == "a"'), true)
  t.is(await evaluate('[1, 2] == [1, 2]'), true)
  t.is(await evaluate('{a: 1} == {a: 1}'), true)
})

test('stdlib: map/filter/fold/reduce', async (t) => {
  t.deepEqual(await evaluate('map([1, 2, 3], |x| x * 2)'), [2, 4, 6])
  t.deepEqual(await evaluate('filter([1, 2, 3, 4], |x| x > 2)'), [3, 4])
  t.is(await evaluate('fold([1, 2, 3], 0, |acc, x| acc + x)'), 6)
  t.is(await evaluate('reduce([1, 2, 3], |acc, x| acc + x)'), 6)
})

test('stdlib: any/every/find', async (t) => {
  t.is(await evaluate('any([1, 2, 3], |x| x > 2)'), true)
  t.is(await evaluate('every([1, 2, 3], |x| x > 2)'), false)
  t.is(await evaluate('find([1, 2, 3], |x| x > 1)'), 2)
  t.is(await evaluate('find([1, 2, 3], |x| x > 9)'), undefined)
})

test('examples: factorial', async (t) => {
  t.deepEqual(
    await evaluate(`{
      let fact = |n| if (n <= 1) 1 else n * fact(n - 1);
      [fact(1), fact(2), fact(3), fact(4), fact(5), fact(6)]
    }`),
    [1, 2, 6, 24, 120, 720],
  )
})

test('examples: mutual recursion parity', async (t) => {
  t.is(
    await evaluate(`{
      let fa = |n| if (n == 0) true else fb(n - 1);
      let fb = |n| if (n == 0) false else fa(n - 1);
      let isEven = |n| fa(n);
      isEven(101)
    }`),
    false,
  )
})

test('examples: fixed point combinator', async (t) => {
  t.is(
    await evaluate(`{
      let fix = |f| |x| f(fix(f))(x);
      let fact_f = |f| |n| if (n <= 1) 1 else n * f(n - 1);
      let fact = fix(fact_f);
      fact(5)
    }`),
    120,
  )
})

test('js interop: nested structures', async (t) => {
  t.deepEqual(await evaluate('{ a: 1, b: 2 + 2, c: [1, 2, { nested: "val" + "ue" }] }'), {
    a: 1,
    b: 4,
    c: [1, 2, { nested: 'value' }],
  })
})

test('js interop: closures render as string', async (t) => {
  t.is(await evaluate('|x| x'), '<closure>')
})

test('async natives: promise-returning native function', async (t) => {
  const evaluator = new Evaluator()
  evaluator.registerNative('fetchId', 0, async () => {
    await new Promise((resolve) => setTimeout(resolve, 5))
    return { tag: 'number', value: 42 }
  })
  const result = await import('../src/index').then((m) => m.evaluateWith(evaluator, 'fetchId() + 1'))
  t.is(result, 43)
})

test('async natives: mixed with closures', async (t) => {
  const evaluator = new Evaluator()
  evaluator.registerNative('delayedDouble', 1, async (_eval, args) => {
    await new Promise((resolve) => setTimeout(resolve, 5))
    if (args[0].tag !== 'number') {
      throw new Error('Expected number')
    }
    return { tag: 'number', value: args[0].value * 2 }
  })
  const m = await import('../src/index')
  const result = await m.evaluateWith(evaluator, '{ let f = |x| delayedDouble(x) + 1; f(20) }')
  t.is(result, 41)
})

test('async natives: stdlib map works with async-aware evaluator', async (t) => {
  t.deepEqual(await evaluate('map([1, 2, 3], |x| x * x)'), [1, 4, 9])
})
