import type { AstNode, AstObjectKey, BOp, LOp, UOp } from './ast'
import { Parser } from './parser'
import { Scanner, type TokenType } from './scanner'
import { number as numValue, str, type GluValue } from './value'

const PRECEDENCE = [
  'none',
  'assignment',
  'or',
  'and',
  'equality',
  'comparison',
  'term',
  'factor',
  'unary',
  'call',
  'primary',
] as const

type Precedence = (typeof PRECEDENCE)[number]

function nextPrecedence(p: Precedence): Precedence {
  const idx = PRECEDENCE.indexOf(p)
  return PRECEDENCE[idx + 1]
}

function precedenceRank(p: Precedence): number {
  return PRECEDENCE.indexOf(p)
}

type PrefixParseFn = (compiler: Compiler, canAssign: boolean) => AstNode
type InfixParseFn = (compiler: Compiler, lhs: AstNode, canAssign: boolean) => AstNode

interface ParseRule {
  prefix: PrefixParseFn | null
  infix: InfixParseFn | null
  precedence: Precedence
}

export class Compiler {
  readonly parser: Parser

  constructor(source: string) {
    this.parser = new Parser(new Scanner(source))
  }

  compile(): AstNode | null {
    this.parser.advance()
    const expr = this.expression()
    return this.parser.hadError ? null : expr
  }

  private expression(): AstNode {
    return this.parsePrecedence('assignment')
  }

  literal(): AstNode {
    const token = this.parser.getPrevious()
    let value: GluValue
    switch (token.kind) {
      case 'false':
        value = { tag: 'bool', value: false }
        break
      case 'true':
        value = { tag: 'bool', value: true }
        break
      case 'nil':
        value = { tag: 'nil' }
        break
      case 'string':
        value = str(token.lexeme.slice(1, -1))
        break
      case 'number':
        value = numValue(Number(token.lexeme))
        break
      default:
        throw new Error('unreachable')
    }
    return { tag: 'literal', value }
  }

  grouping(): AstNode {
    const node = this.expression()
    this.parser.consume('rightParen', "Expect ')' after expression.")
    return node
  }

  unary(): AstNode {
    const operatorType = this.parser.getPrevious().kind
    const operand = this.parsePrecedence('unary')
    const op: UOp = operatorType === 'bang' ? 'not' : 'negate'
    return { tag: 'unary', op, operand }
  }

  binary(lhs: AstNode): AstNode {
    const operatorType = this.parser.getPrevious().kind
    const rule = getRule(operatorType)
    const right = this.parsePrecedence(nextPrecedence(rule.precedence))
    const op: BOp =
      BINARY_OPS[operatorType] ??
      (() => {
        throw new Error('unreachable')
      })()
    return { tag: 'binary', op, left: lhs, right }
  }

  logical(lhs: AstNode): AstNode {
    const operatorType = this.parser.getPrevious().kind
    const rule = getRule(operatorType)
    const right = this.parsePrecedence(nextPrecedence(rule.precedence))
    const op: LOp = operatorType === 'and' ? 'and' : 'or'
    return { tag: 'logical', op, left: lhs, right }
  }

  list(): AstNode {
    const items: AstNode[] = []
    if (!this.parser.check('rightBracket')) {
      do {
        items.push(this.expression())
      } while (this.parser.matches('comma'))
    }
    this.parser.consume('rightBracket', "Expected ']'.")
    return { tag: 'list', items }
  }

  variable(): AstNode {
    return { tag: 'variableRef', name: this.parser.getPrevious().lexeme }
  }

  blockOrObjectLiteral(): AstNode {
    return this.lookObjectLiteralAhead() ? this.objectLiteral() : this.block()
  }

  lookObjectLiteralAhead(): boolean {
    const lookaheadParser = this.parser.clone()
    if (lookaheadParser.matches('leftBracket')) {
      let level = 1
      while (!this.parser.check('eof')) {
        if (lookaheadParser.matches('rightBracket')) {
          level--
          if (level === 0) {
            return lookaheadParser.matches('colon')
          }
        } else {
          if (lookaheadParser.check('leftBracket')) {
            level++
          }
          lookaheadParser.advance()
        }
      }
      return false
    }
    return (
      (lookaheadParser.matches('identifier') || lookaheadParser.matches('string')) && lookaheadParser.matches('colon')
    )
  }

  objectLiteral(): AstNode {
    const entries: Array<{ key: AstObjectKey; value: AstNode }> = []
    if (!this.parser.check('rightBrace')) {
      do {
        let key: AstObjectKey
        if (this.parser.matches('identifier')) {
          key = { tag: 'literal', key: this.parser.getPrevious().lexeme }
        } else if (this.parser.matches('string')) {
          const lexeme = this.parser.getPrevious().lexeme
          key = { tag: 'literal', key: lexeme.slice(1, -1) }
        } else if (this.parser.matches('leftBracket')) {
          const keyExpr = this.expression()
          this.parser.consume('rightBracket', "Expected ']'.")
          key = { tag: 'computed', expr: keyExpr }
        } else {
          this.parser.errorAtCurrent('Expected a string or an identifier.')
          key = { tag: 'literal', key: 'error' }
        }
        this.parser.consume('colon', "Expected ':'.")
        entries.push({ key, value: this.expression() })
      } while (this.parser.matches('comma'))
    }
    this.parser.consume('rightBrace', "Expected '}'.")
    return { tag: 'object', entries }
  }

  block(): AstNode {
    const items: AstNode[] = []
    if (!this.parser.check('rightBrace')) {
      do {
        items.push(this.expression())
      } while (this.parser.matches('semicolon'))
    }
    this.parser.consume('rightBrace', "Expected '}'.")
    return { tag: 'block', items }
  }

  letBinding(): AstNode {
    this.parser.consume('identifier', 'Expected an identifier.')
    const name = this.parser.getPrevious().lexeme
    this.parser.consume('equal', "Expected '='.")
    return { tag: 'binding', name, expr: this.expression() }
  }

  indexedAccess(lhs: AstNode): AstNode {
    const indexExpr = this.expression()
    this.parser.consume('rightBracket', "Expected ']'.")
    return { tag: 'index', target: lhs, index: indexExpr }
  }

  dot(lhs: AstNode): AstNode {
    this.parser.consume('identifier', "Expect property name after '.'.")
    return { tag: 'property', target: lhs, name: this.parser.getPrevious().lexeme }
  }

  ifExpression(): AstNode {
    this.parser.consume('leftParen', "Expect '(' after 'if'.")
    const condition = this.expression()
    this.parser.consume('rightParen', "Expect ')' after condition.")
    const thenBranch = this.expression()
    const elseBranch = this.parser.matches('else') ? this.expression() : null
    return { tag: 'if', node: { condition, thenBranch, elseBranch } }
  }

  closure(): AstNode {
    const params: string[] = []
    if (!this.parser.check('pipe')) {
      do {
        this.parser.consume('identifier', 'Expected parameter name.')
        params.push(this.parser.getPrevious().lexeme)
      } while (this.parser.matches('comma'))
    }
    this.parser.consume('pipe', "Expected '|' after parameters.")
    return { tag: 'closure', node: { params, body: this.expression() } }
  }

  call(lhs: AstNode): AstNode {
    const args: AstNode[] = []
    if (!this.parser.check('rightParen')) {
      do {
        args.push(this.expression())
      } while (this.parser.matches('comma'))
    }
    this.parser.consume('rightParen', "Expected ')' after arguments.")
    return { tag: 'call', callee: lhs, args }
  }

  private parsePrecedence(precedence: Precedence): AstNode {
    this.parser.advance()
    const prefixRule = getRule(this.parser.getPrevious().kind).prefix
    if (prefixRule === null) {
      this.parser.errorAtCurrent('Expect expression.')
      return { tag: 'error' }
    }
    const canAssign = precedenceRank(precedence) <= precedenceRank('assignment')
    let lhs = prefixRule(this, canAssign)
    while (precedenceRank(precedence) <= precedenceRank(getRule(this.parser.getCurrent().kind).precedence)) {
      this.parser.advance()
      const infixRule = getRule(this.parser.getPrevious().kind).infix
      if (infixRule === null) {
        break
      }
      lhs = infixRule(this, lhs, canAssign)
    }
    if (canAssign && this.parser.matches('equal')) {
      this.parser.errorAtCurrent('Invalid assignment target.')
      return { tag: 'error' }
    }
    return lhs
  }
}

const BINARY_OPS: Partial<Record<TokenType, BOp>> = {
  plus: 'add',
  minus: 'sub',
  star: 'mul',
  slash: 'div',
  percent: 'mod',
  less: 'less',
  lessEqual: 'lessEqual',
  greater: 'greater',
  greaterEqual: 'greaterEqual',
  equalEqual: 'equal',
  bangEqual: 'notEqual',
}

const DEFAULT_RULE: ParseRule = { prefix: null, infix: null, precedence: 'none' }

const RULES = new Map<TokenType, ParseRule>([
  ['leftParen', { prefix: (c) => c.grouping(), infix: (c, lhs) => c.call(lhs), precedence: 'call' }],
  ['number', { prefix: (c) => c.literal(), infix: null, precedence: 'none' }],
  ['nil', { prefix: (c) => c.literal(), infix: null, precedence: 'none' }],
  ['true', { prefix: (c) => c.literal(), infix: null, precedence: 'none' }],
  ['false', { prefix: (c) => c.literal(), infix: null, precedence: 'none' }],
  ['identifier', { prefix: (c) => c.variable(), infix: null, precedence: 'none' }],
  ['string', { prefix: (c) => c.literal(), infix: null, precedence: 'none' }],
  ['bangEqual', { prefix: null, infix: (c, lhs) => c.binary(lhs), precedence: 'equality' }],
  ['equalEqual', { prefix: null, infix: (c, lhs) => c.binary(lhs), precedence: 'equality' }],
  ['greater', { prefix: null, infix: (c, lhs) => c.binary(lhs), precedence: 'comparison' }],
  ['greaterEqual', { prefix: null, infix: (c, lhs) => c.binary(lhs), precedence: 'comparison' }],
  ['less', { prefix: null, infix: (c, lhs) => c.binary(lhs), precedence: 'comparison' }],
  ['lessEqual', { prefix: null, infix: (c, lhs) => c.binary(lhs), precedence: 'comparison' }],
  ['bang', { prefix: (c) => c.unary(), infix: null, precedence: 'none' }],
  ['minus', { prefix: (c) => c.unary(), infix: (c, lhs) => c.binary(lhs), precedence: 'term' }],
  ['plus', { prefix: null, infix: (c, lhs) => c.binary(lhs), precedence: 'term' }],
  ['star', { prefix: null, infix: (c, lhs) => c.binary(lhs), precedence: 'factor' }],
  ['slash', { prefix: null, infix: (c, lhs) => c.binary(lhs), precedence: 'factor' }],
  ['percent', { prefix: null, infix: (c, lhs) => c.binary(lhs), precedence: 'factor' }],
  ['and', { prefix: null, infix: (c, lhs) => c.logical(lhs), precedence: 'and' }],
  ['or', { prefix: null, infix: (c, lhs) => c.logical(lhs), precedence: 'or' }],
  ['dot', { prefix: null, infix: (c, lhs) => c.dot(lhs), precedence: 'call' }],
  ['leftBracket', { prefix: (c) => c.list(), infix: (c, lhs) => c.indexedAccess(lhs), precedence: 'call' }],
  ['leftBrace', { prefix: (c) => c.blockOrObjectLiteral(), infix: null, precedence: 'none' }],
  ['let', { prefix: (c) => c.letBinding(), infix: null, precedence: 'none' }],
  ['if', { prefix: (c) => c.ifExpression(), infix: null, precedence: 'none' }],
  ['pipe', { prefix: (c) => c.closure(), infix: null, precedence: 'none' }],
])

function getRule(tokenType: TokenType): ParseRule {
  return RULES.get(tokenType) ?? DEFAULT_RULE
}
