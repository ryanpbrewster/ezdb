export function makeId(): string {
  return "id" + Math.random().toString(36).substring(2);
}
