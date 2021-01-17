import { AdminClient, Client } from "./client";
import { makeId } from "./utils";

async function main() {
  const config = { projectId: "bench" };
  const admin = new AdminClient(config);
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
  const client = new Client(config);

  console.log(await client.query("get_person", { ":id": "alice" }));
  await client.mutate("add_person", {
    ":id": "alice",
    ":name": "Alice Accountant",
  }).catch(() => {});
  console.log(await client.query("get_person", { ":id": "alice" }));

  let total = 0;
  let inflight = 0;
  function kick() {
    while (inflight < 128) {
      inflight++;
      client.query("get_person", { ":id": "alice" }).then(() => {
        inflight--;
        total++;
        kick();
      }).catch((err) => console.error(err));
    }
  }
  kick();
  let prev = 0;
  while (true) {
    console.log(`QPS = ${total - prev}`);
    prev = total;
    await new Promise(r => setTimeout(r, 1000));
  }
}

main()
  .then(() => console.log("ok"))
  .catch((err) => console.error(err));
