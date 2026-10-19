import type { GluValue } from './value'

export class Environment {
  readonly enclosing: Environment | null
  readonly #values = new Map<string, GluValue>()

  constructor(enclosing: Environment | null = null) {
    this.enclosing = enclosing
  }

  get(key: string): GluValue | undefined {
    const local = this.#values.get(key)
    if (local !== undefined) {
      return local
    }
    return this.enclosing !== null ? this.enclosing.get(key) : undefined
  }

  set(key: string, value: GluValue): void {
    this.#values.set(key, value)
  }
}
