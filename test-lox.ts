import m from "./index.js";

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
