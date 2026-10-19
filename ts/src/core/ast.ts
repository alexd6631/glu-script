import type { GluValue } from './value'

export type UOp = 'not' | 'negate'
export type BOp =
  | 'add'
  | 'sub'
  | 'mul'
  | 'div'
  | 'mod'
  | 'less'
  | 'lessEqual'
  | 'greater'
  | 'greaterEqual'
  | 'equal'
  | 'notEqual'
export type LOp = 'and' | 'or'

export interface IfNode {
  condition: AstNode
  thenBranch: AstNode
  elseBranch: AstNode | null
}

export interface ClosureNode {
  params: string[]
  body: AstNode
}

export type AstObjectKey = { tag: 'literal'; key: string } | { tag: 'computed'; expr: AstNode }

export type AstNode =
  | { tag: 'literal'; value: GluValue }
  | { tag: 'list'; items: AstNode[] }
  | { tag: 'object'; entries: Array<{ key: AstObjectKey; value: AstNode }> }
  | { tag: 'block'; items: AstNode[] }
  | { tag: 'unary'; op: UOp; operand: AstNode }
  | { tag: 'binary'; op: BOp; left: AstNode; right: AstNode }
  | { tag: 'logical'; op: LOp; left: AstNode; right: AstNode }
  | { tag: 'index'; target: AstNode; index: AstNode }
  | { tag: 'property'; target: AstNode; name: string }
  | { tag: 'binding'; name: string; expr: AstNode }
  | { tag: 'variableRef'; name: string }
  | { tag: 'if'; node: IfNode }
  | { tag: 'closure'; node: ClosureNode }
  | { tag: 'call'; callee: AstNode; args: AstNode[] }
  | { tag: 'error' }
