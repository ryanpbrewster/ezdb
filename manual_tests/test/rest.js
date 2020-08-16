const request = require("request");
const expect = require("chai").expect;

describe("REST API", () => {
  it("PUT", async () => {
    const {resp, body} = await put(42);

    expect(resp.statusCode).to.eql(204);
    expect(body).to.be.undefined;
  });

  it("GET", async () => {
    await put({ a: 42, b: "asdf" });
    const {resp, body} = await get();

    expect(resp.statusCode).to.eql(200);
    expect(body).to.eql({ a: 42, b: "asdf" });
  });
});

async function get() {
  return req({ uri: "http://localhost:9000/v0/", method: "GET", json: true });
}

async function put(data) {
  return req({ uri: "http://localhost:9000/v0/", method: "PUT", json: data });
}

async function req(opts) {
  return new Promise((resolve, reject) => {
    request(opts, (err, resp, body) => {
      return err ? reject(err) : resolve({resp, body});
    });
  });
}
