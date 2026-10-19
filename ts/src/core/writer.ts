declare const process: {
  stdout: { write(s: string): void }
  stderr: { write(s: string): void }
}

export interface Writer {
  write(s: string): void
}

export class ConsoleWriter implements Writer {
  write(s: string): void {
    process.stdout.write(s)
  }
}

export class ConsoleErrWriter implements Writer {
  write(s: string): void {
    process.stderr.write(s)
  }
}

export class Writers {
  writer: Writer
  errWriter: Writer
  traceWriter: Writer

  constructor(writer: Writer = new ConsoleWriter(), errWriter: Writer = new ConsoleErrWriter()) {
    this.writer = writer
    this.errWriter = errWriter
    this.traceWriter = errWriter
  }
}

export class StringCollector implements Writer {
  readonly chunks: string[] = []
  write(s: string): void {
    this.chunks.push(s)
  }
  toString(): string {
    return this.chunks.join('')
  }
}
