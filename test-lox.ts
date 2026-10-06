import m from './index.js';

// m.testScan(`
// fun sample() {
//   {
//     "a": 1,
//     "b": input.value
//   }
// }

// sample()
// `);

// m.testScan('[1, 2, 3]')
m.testEvaluate('[1 +1 , 2 + 2, 3 * 4, [1, 2*2, 3*3]]')
// m.testEvaluate("2 * (3 + 1)")


m.testEvaluate(`
{
 a: 1,
 b: 2 + 2,
 c: [1, 2, { nested: "val" + "ue" }]
}
`)


m.testEvaluate(`
{
  let a = 1;
  let b = 2;

  let obj = {
    add: a + b,
    mul: a * b
  };

  obj
}
`)
