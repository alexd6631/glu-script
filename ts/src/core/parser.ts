import { Scanner, type Token, type TokenType } from './scanner'
import { Writers } from './writer'

export class Parser {
  private readonly scanner: Scanner
  private current: Token | null = null
  private previous: Token | null = null
  private panicMode = false
  readonly writers: Writers
  hadError = false

  constructor(scanner: Scanner, writers: Writers = new Writers()) {
    this.scanner = scanner
    this.writers = writers
  }

  advance(): void {
    this.previous = this.current
    for (;;) {
      this.current = this.scanner.scanToken()
      if (this.current.kind !== 'error') {
        break
      }
      this.errorAtCurrent(this.current.lexeme)
    }
  }

  consume(kind: TokenType, message: string): void {
    if (this.current?.kind === kind) {
      this.advance()
      return
    }
    this.errorAtCurrent(message)
  }

  matches(kind: TokenType): boolean {
    if (!this.check(kind)) {
      return false
    }
    this.advance()
    return true
  }

  check(kind: TokenType): boolean {
    return this.getCurrent().kind === kind
  }

  getCurrent(): Token {
    if (this.current === null) {
      throw new Error('Parser accessed before advance()')
    }
    return this.current
  }

  getPrevious(): Token {
    if (this.previous === null) {
      throw new Error('Parser has no previous token')
    }
    return this.previous
  }

  errorAtCurrent(message: string): void {
    this.errorAt(this.getCurrent(), message)
  }

  errorAt(token: Token, message: string): void {
    if (this.panicMode) {
      return
    }
    this.panicMode = true
    let output = `[line ${token.line}] Error`
    if (token.kind === 'eof') {
      output += ' at end'
    } else if (token.kind !== 'error') {
      output += ` at ${token.lexeme}`
    }
    output += `: ${message}\n`
    this.writers.errWriter.write(output)
    this.hadError = true
  }

  clone(): Parser {
    const parser = new Parser(this.scanner.clone(), this.writers)
    parser.current = this.current
    parser.previous = this.previous
    parser.hadError = this.hadError
    parser.panicMode = this.panicMode
    return parser
  }
}
