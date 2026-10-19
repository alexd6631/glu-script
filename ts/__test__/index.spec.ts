import test from 'ava'
import { evaluate, evaluateValue, parse, scan } from '../src/index'

function throwsMessage(source: string): string {
  try {
    evaluateValue(source)
  } catch (e) {
    return (e as Error).message
  }
  throw new Error(`expected '${source}' to throw`)
}

test('scanner: empty source yields eof', (t) => {
  t.deepEqual(scan(''), [{ kind: 'eof', lexeme: '', line: 1 }])
})

test('scanner: simple lexemes', (t) => {
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

test('scanner: two char lexemes', (t) => {
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

test('scanner: newlines and comments', (t) => {
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

test('scanner: numbers', (t) => {
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

test('parse: operator precedence', (t) => {
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

test('parse: unary', (t) => {
  t.deepEqual(parse('-1'), {
    tag: 'unary',
    op: 'negate',
    operand: { tag: 'literal', value: { tag: 'number', value: 1 } },
  })
})

test('evaluator: binding and reference', (t) => {
  t.is(evaluate('{ let x = 1 + 2; x }'), 3)
})

test('evaluator: binding returns bound value', (t) => {
  t.is(evaluate('{ let x = 5 }'), 5)
})

test('evaluator: nested block shadowing', (t) => {
  t.is(evaluate('{ let x = 1; { let x = 2; x } + x }'), 3)
})

test('evaluator: undefined variable', (t) => {
  t.is(throwsMessage('foo'), "Undefined variable 'foo'.")
})

test('evaluator: block scopes are popped', (t) => {
  t.is(throwsMessage('{ { let x = 1; 2 }; { x } }'), "Undefined variable 'x'.")
})

test('evaluator: list indexed access', (t) => {
  t.is(evaluate('[10, 20, 30][1]'), 20)
})

test('evaluator: string indexed access', (t) => {
  t.is(evaluate('"hello"[1]'), 'e')
})

test('evaluator: chained indexed access', (t) => {
  t.is(evaluate('[[1, 2], [3, 4]][1][0]'), 3)
})

test('evaluator: index out of range', (t) => {
  t.is(throwsMessage('[1, 2][5]'), 'Index out of range.')
})

test('evaluator: fractional index', (t) => {
  t.is(throwsMessage('[1, 2][0.5]'), 'Index must be a non-negative integer.')
})

test('evaluator: non-number index', (t) => {
  t.is(throwsMessage('[1, 2]["a"]'), 'Index must be a number.')
})

test('evaluator: non-indexable target', (t) => {
  t.is(throwsMessage('1[0]'), 'Only lists and strings can be indexed.')
})

test('evaluator: object property access', (t) => {
  t.is(evaluate('{a: 1}.a'), 1)
})

test('evaluator: chained property access', (t) => {
  t.is(evaluate('{a: {b: "x"}}.a.b'), 'x')
})

test('evaluator: missing property', (t) => {
  t.is(throwsMessage('{a: 1}.b'), "Undefined property 'b'.")
})

test('evaluator: non-object property target', (t) => {
  t.is(throwsMessage('[1, 2].a'), 'Only objects have properties.')
})

test('evaluator: if/else', (t) => {
  t.is(evaluate('if (true) 1 else 2'), 1)
  t.is(evaluate('if (false) 1 else 2'), 2)
  t.is(evaluate('if (false) 1'), undefined)
  t.is(evaluate('if (false) 1 else if (true) 2 else 3'), 2)
})

test('evaluator: if truthiness', (t) => {
  t.is(evaluate('if (1) "yes" else "no"'), 'yes')
  t.is(evaluate('if (nil) 1 else 2'), 2)
})

test('evaluator: if with blocks', (t) => {
  t.is(evaluate('if ({ let x = 5; x > 3 }) { let y = 10; y * 2 } else { 0 }'), 20)
})

test('evaluator: closures capture environment', (t) => {
  t.is(evaluate('{ let z = 10; let f = |x| x + z; f(5) }'), 15)
})

test('evaluator: call closure basics', (t) => {
  t.is(evaluate('(|x| x + 1)(5)'), 6)
  t.is(evaluate('(|x, y| x + y)(3, 4)'), 7)
  t.is(evaluate('(|| 42)()'), 42)
})

test('evaluator: call wrong arg count', (t) => {
  t.is(throwsMessage('(|x, y| x + y)(1)'), 'Expected 2 argument(s) but got 1.')
  t.is(throwsMessage('(|x| x)(1, 2)'), 'Expected 1 argument(s) but got 2.')
})

test('evaluator: call non-closure', (t) => {
  t.is(throwsMessage('(1)(2)'), 'Only closures and native functions can be called.')
})

test('evaluator: chained closures', (t) => {
  t.is(evaluate('(|x| |y| x + y)(3)(4)'), 7)
})

test('evaluator: recursion depth limit', (t) => {
  t.is(throwsMessage('{ let f = |n| f(n + 1); f(0) }'), 'Recursion limit of 256 exceeded.')
})

test('evaluator: string concat requires strings', (t) => {
  t.is(throwsMessage('1 + "a"'), 'Operands must be two numbers or two strings.')
})

test('evaluator: equality', (t) => {
  t.is(evaluate('1 == 1'), true)
  t.is(evaluate('1 != 2'), true)
  t.is(evaluate('"a" == "a"'), true)
  t.is(evaluate('[1, 2] == [1, 2]'), true)
  t.is(evaluate('{a: 1} == {a: 1}'), true)
})

test('stdlib: map/filter/fold/reduce', (t) => {
  t.deepEqual(evaluate('map([1, 2, 3], |x| x * 2)'), [2, 4, 6])
  t.deepEqual(evaluate('filter([1, 2, 3, 4], |x| x > 2)'), [3, 4])
  t.is(evaluate('fold([1, 2, 3], 0, |acc, x| acc + x)'), 6)
  t.is(evaluate('reduce([1, 2, 3], |acc, x| acc + x)'), 6)
})

test('stdlib: any/every/find', (t) => {
  t.is(evaluate('any([1, 2, 3], |x| x > 2)'), true)
  t.is(evaluate('every([1, 2, 3], |x| x > 2)'), false)
  t.is(evaluate('find([1, 2, 3], |x| x > 1)'), 2)
  t.is(evaluate('find([1, 2, 3], |x| x > 9)'), undefined)
})

test('examples: factorial', (t) => {
  t.deepEqual(
    evaluate(`{
      let fact = |n| if (n <= 1) 1 else n * fact(n - 1);
      [fact(1), fact(2), fact(3), fact(4), fact(5), fact(6)]
    }`),
    [1, 2, 6, 24, 120, 720],
  )
})

test('examples: mutual recursion parity', (t) => {
  t.is(
    evaluate(`{
      let fa = |n| if (n == 0) true else fb(n - 1);
      let fb = |n| if (n == 0) false else fa(n - 1);
      let isEven = |n| fa(n);
      isEven(101)
    }`),
    false,
  )
})

test('examples: fixed point combinator', (t) => {
  t.is(
    evaluate(`{
      let fix = |f| |x| f(fix(f))(x);
      let fact_f = |f| |n| if (n <= 1) 1 else n * f(n - 1);
      let fact = fix(fact_f);
      fact(5)
    }`),
    120,
  )
})

test('js interop: nested structures', (t) => {
  t.deepEqual(evaluate('{ a: 1, b: 2 + 2, c: [1, 2, { nested: "val" + "ue" }] }'), {
    a: 1,
    b: 4,
    c: [1, 2, { nested: 'value' }],
  })
})

test('js interop: closures render as string', (t) => {
  t.is(evaluate('|x| x'), '<closure>')
})
