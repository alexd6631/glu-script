export type Line = number

export type TokenType =
  | 'error'
  | 'eof'
  | 'leftParen'
  | 'rightParen'
  | 'leftBrace'
  | 'rightBrace'
  | 'leftBracket'
  | 'rightBracket'
  | 'semicolon'
  | 'comma'
  | 'pipe'
  | 'dot'
  | 'colon'
  | 'minus'
  | 'plus'
  | 'slash'
  | 'star'
  | 'percent'
  | 'bangEqual'
  | 'bang'
  | 'equalEqual'
  | 'equal'
  | 'lessEqual'
  | 'less'
  | 'greaterEqual'
  | 'greater'
  | 'string'
  | 'number'
  | 'identifier'
  | 'and'
  | 'else'
  | 'if'
  | 'nil'
  | 'print'
  | 'or'
  | 'while'
  | 'false'
  | 'let'
  | 'for'
  | 'fun'
  | 'true'
  | 'switch'
  | 'case'
  | 'default'

export interface Token {
  kind: TokenType
  lexeme: string
  line: Line
}

export const KEYWORDS: Readonly<Record<string, TokenType>> = {
  and: 'and',
  case: 'case',
  default: 'default',
  else: 'else',
  false: 'false',
  for: 'for',
  fun: 'fun',
  if: 'if',
  nil: 'nil',
  or: 'or',
  print: 'print',
  switch: 'switch',
  true: 'true',
  let: 'let',
  while: 'while',
}

export function syntheticToken(name: string): Token {
  return { kind: 'eof', lexeme: name, line: 0 }
}

function isAlpha(c: string): boolean {
  return /[a-zA-Z_]/.test(c)
}

function isDigit(c: string): boolean {
  return c >= '0' && c <= '9'
}

export class Scanner {
  private readonly source: string
  private start = 0
  private current = 0
  private line: Line = 1
  private done = false

  constructor(source: string) {
    this.source = source
  }

  scanToken(): Token {
    this.skipWhitespace()
    this.start = this.current
    if (this.isAtEnd()) {
      return this.makeToken('eof')
    }
    const c = this.advance()
    if (isAlpha(c)) {
      return this.identifier()
    }
    if (isDigit(c)) {
      return this.number()
    }
    switch (c) {
      case '(':
        return this.makeToken('leftParen')
      case ')':
        return this.makeToken('rightParen')
      case '{':
        return this.makeToken('leftBrace')
      case '}':
        return this.makeToken('rightBrace')
      case '[':
        return this.makeToken('leftBracket')
      case ']':
        return this.makeToken('rightBracket')
      case ':':
        return this.makeToken('colon')
      case ';':
        return this.makeToken('semicolon')
      case ',':
        return this.makeToken('comma')
      case '|':
        return this.makeToken('pipe')
      case '.':
        return this.makeToken('dot')
      case '-':
        return this.makeToken('minus')
      case '+':
        return this.makeToken('plus')
      case '/':
        return this.makeToken('slash')
      case '*':
        return this.makeToken('star')
      case '%':
        return this.makeToken('percent')
      case '!':
        return this.makeToken(this.matches('=') ? 'bangEqual' : 'bang')
      case '=':
        return this.makeToken(this.matches('=') ? 'equalEqual' : 'equal')
      case '<':
        return this.makeToken(this.matches('=') ? 'lessEqual' : 'less')
      case '>':
        return this.makeToken(this.matches('=') ? 'greaterEqual' : 'greater')
      case '"':
        return this.string()
    }
    return this.errorToken('Unexpected character.')
  }

  private string(): Token {
    while (this.peek() !== '"' && !this.isAtEnd()) {
      if (this.peek() === '\n') {
        this.line++
      }
      this.advance()
    }
    if (this.isAtEnd()) {
      return this.errorToken('Unterminated string')
    }
    this.advance()
    return this.makeToken('string')
  }

  private number(): Token {
    while (isDigit(this.peek())) {
      this.advance()
    }
    if (this.peek() === '.' && isDigit(this.peekNext())) {
      this.advance()
      while (isDigit(this.peek())) {
        this.advance()
      }
    }
    return this.makeToken('number')
  }

  private identifier(): Token {
    while (isAlpha(this.peek()) || isDigit(this.peek())) {
      this.advance()
    }
    const kind = KEYWORDS[this.currentLexeme()] ?? 'identifier'
    return this.makeToken(kind)
  }

  private skipWhitespace(): void {
    for (;;) {
      const c = this.peek()
      if (c === ' ' || c === '\r' || c === '\t') {
        this.advance()
      } else if (c === '\n') {
        this.line++
        this.advance()
      } else if (c === '/' && this.peekNext() === '/') {
        while (this.peek() !== '\n' && !this.isAtEnd()) {
          this.advance()
        }
      } else {
        return
      }
    }
  }

  private makeToken(kind: TokenType): Token {
    return { kind, lexeme: this.currentLexeme(), line: this.line }
  }

  private currentLexeme(): string {
    return this.source.slice(this.start, this.current)
  }

  private errorToken(message: string): Token {
    return { kind: 'error', lexeme: message, line: this.line }
  }

  private isAtEnd(): boolean {
    return this.current >= this.source.length
  }

  private advance(): string {
    return this.source[this.current++] ?? '\0'
  }

  private matches(expected: string): boolean {
    if (this.isAtEnd() || this.source[this.current] !== expected) {
      return false
    }
    this.current++
    return true
  }

  private peek(): string {
    return this.source[this.current] ?? '\0'
  }

  private peekNext(): string {
    return this.source[this.current + 1] ?? '\0'
  }

  [Symbol.iterator](): Iterator<Token> {
    return {
      next: (): IteratorResult<Token> => {
        if (this.done) {
          return { done: true, value: undefined }
        }
        const token = this.scanToken()
        if (token.kind === 'eof') {
          this.done = true
        }
        return { done: false, value: token }
      },
    }
  }

  clone(): Scanner {
    const scanner = new Scanner(this.source)
    scanner.start = this.start
    scanner.current = this.current
    scanner.line = this.line
    scanner.done = this.done
    return scanner
  }

  toArray(): Token[] {
    const tokens: Token[] = []
    for (const token of this) {
      tokens.push(token)
    }
    return tokens
  }
}
