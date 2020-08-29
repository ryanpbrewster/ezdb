const request = require("request");
const expect = require("chai").expect;

describe("REST API", () => {
  it("PUT", async () => {
    const {resp, body} = await execRaw("CREATE TABLE IF NOT EXISTS person (id INTEGER PRIMARY KEY, name TEXT NOT NULL)");

    expect(resp.statusCode).to.eql(200);
    expect(body).to.be.null;
  });

  it("GET", async () => {
    await execRaw("DELETE FROM person");
    await execRaw("INSERT INTO person (name) VALUES ('alice'), ('bob'), ('carol')");
    const {resp, body} = await queryRaw("SELECT id, name FROM person");

    expect(resp.statusCode).to.eql(200);
    expect(body).to.eql([
      {id: 1, name: "alice"},
      {id: 2, name: "bob"},
      {id: 3, name: "carol"},
    ]);
  });
});

async function queryRaw(body) {
  return req({ uri: "http://localhost:9000/v0/raw", method: "GET", body });
}

async function execRaw(body) {
  return req({ uri: "http://localhost:9000/v0/raw", method: "POST", body });
}

async function req(opts) {
  return new Promise((resolve, reject) => {
    request(opts, (err, resp, body) => {
      return err ? reject(err) : resolve({resp, body: JSON.parse(body)});
    });
  });
}
