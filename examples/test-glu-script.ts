import m from "../index.js";

function evaluate(s: string): unknown {
  const val = m.testEvaluate(s);
  console.log(val);
  return val;
}

evaluate("[1 +1 , 2 + 2, 3 * 4, [1, 2*2, 3*3]]");

const res = evaluate(`
{
 a: 1,
 b: 2 + 2,
 c: [1, 2, { nested: "val" + "ue" }]
}
`);
console.log((res as any).c[2]);

evaluate(`
{
  let a = 1;
  let b = 2;

  let obj = {
    add: a + b,
    mul: a * b
  };

  obj
}
`);

evaluate(`
{
  let obj = {
    a: {
      b: {
        c: 42
      }
    }
  };

  obj.a.b.c * 2
}`);


evaluate(`
{
  let f = |x| x * 2;
  f(f(42))
}
`)

evaluate(`
{
  let fact = |n| if (n <= 1) 1 else n * fact(n - 1);

  [
    fact(1),
    fact(2),
    fact(3),
    fact(4),
    fact(5),
    fact(6)
  ]
}
`)



evaluate(`
{
  let fa = |n| if (n == 0) true else fb(n - 1);
  let fb = |n| if (n == 0) false else fa(n - 1);
  let isEven = |n| fa(n);

  isEven(101)
}
`)

evaluate(`
{
  let adder = |n| |x| x + n;
  let addOne = adder(1);
  addOne(1)
}
`)

evaluate(`
{
  let fix = |f| |x| f(fix(f))(x);
  let fact_f = |f| |n| if (n <= 1) 1 else n * f(n - 1);
  let fact = fix(fact_f);

  fact(5)
}
`)
