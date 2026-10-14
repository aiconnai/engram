export class EngramError extends Error {
  public readonly code?: string;
  public readonly details?: unknown;

  constructor(message: string, code?: string, details?: unknown) {
    super(message);
    this.name = "EngramError";
    this.code = code;
    this.details = details;
  }
}
