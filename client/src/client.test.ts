import { AdminClient, Client } from "./client";

describe("Client", () => {
  it("should be able to query", async () => {
    const admin = new AdminClient("http://localhost:9000");
    await admin.mutate(`
        CREATE TABLE IF NOT EXISTS person (id TEXT PRIMARY KEY, name TEXT NOT NULL)
    `);
    await admin.mutate(`
        INSERT OR IGNORE INTO person (id, name)
        VALUES ('alice', 'Alice Accountant'), ('bob', 'Bob Banker')
    `);
    await admin.setPolicy({
      queries: [
        {
          name: "get_person",
          rawSql: "SELECT name FROM person WHERE id = :id",
        },
      ],
      mutations: [],
    });
    const client = new Client("http://localhost:9000");
    const result = await client.query("get_person", { ":id": "alice" });
    expect(result).toContainEqual({ name: "Alice Accountant" });
  });

  it("should be able to mutate", async () => {
    const admin = new AdminClient("http://localhost:9000");
    await admin.mutate(`DROP TABLE IF EXISTS person`);
    await admin.mutate(`
        CREATE TABLE IF NOT EXISTS person (id TEXT PRIMARY KEY, name TEXT NOT NULL)
    `);
    await admin.setPolicy({
      queries: [
        {
          name: "get_person",
          rawSql: "SELECT name FROM person WHERE id = :id",
        },
      ],
      mutations: [
        {
          name: "add_person",
          rawSql: "INSERT INTO person (id, name) VALUES (:id, :name)",
        },
        {
          name: "delete_person",
          rawSql: "DELETE FROM person WHERE id = :id",
        },
      ],
    });
    const client = new Client("http://localhost:9000");

    expect(await client.query("get_person", { ":id": "alice" })).toHaveLength(
      0
    );

    await client.mutate("add_person", {
      ":id": "alice",
      ":name": "Alice Accountant",
    });
    expect(
      await client.query("get_person", { ":id": "alice" })
    ).toContainEqual({ name: "Alice Accountant" });

    await client.mutate("delete_person", { ":id": "alice" });
    expect(await client.query("get_person", { ":id": "alice" })).toHaveLength(
      0
    );
  });
});