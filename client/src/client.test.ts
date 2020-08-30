import { AdminClient, Client } from "./client";
import { makeId } from "./utils";

describe("Client", () => {
  it("should be able to query", async () => {
    const config = { projectId: makeId() };
    const admin = new AdminClient(config);
    await admin.mutate(`
        CREATE TABLE person (id TEXT PRIMARY KEY, name TEXT NOT NULL)
    `);
    await admin.mutate(`
        INSERT INTO person (id, name)
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
    const client = new Client(config);
    const result = await client.query("get_person", { ":id": "alice" });
    expect(result).toContainEqual({ name: "Alice Accountant" });
  });

  it("should be able to mutate", async () => {
    const config = { projectId: makeId() };
    const admin = new AdminClient(config);
    await admin.mutate(`
        CREATE TABLE person (id TEXT PRIMARY KEY, name TEXT NOT NULL)
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
    const client = new Client(config);

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
