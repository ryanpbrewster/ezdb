import { hello } from "./hello";

describe("Hello function", () => {
  it("should return hello world", () => {
    const result = hello("World");
    expect(result).toBe("Hello, World!");
  });
});
