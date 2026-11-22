const UNEXPLAINED_FAILURE = "Something went wrong, and Nazar could not find out why.";

export function describeFailure(failure: unknown): string {
  if (typeof failure === "string" && failure.length > 0) {
    return failure;
  }
  if (failure instanceof Error && failure.message.length > 0) {
    return failure.message;
  }
  return UNEXPLAINED_FAILURE;
}
