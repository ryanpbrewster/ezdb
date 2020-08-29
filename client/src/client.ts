import got from "got";

export class AdminClient {
  constructor(public readonly address: string) {}

  async setPolicy(policy: Policy): Promise<void> {
    const response = await got.put(`${this.address}/v0/policy`, {
      json: policy,
      throwHttpErrors: false,
    });
    if (response.statusCode !== 200) {
      throw new Error(response.body);
    }
    return JSON.parse(response.body);
  }

  async mutate(rawSql: string): Promise<void> {
    const response = await got.post(`${this.address}/v0/raw`, {
      body: rawSql,
      throwHttpErrors: false,
    });
    if (response.statusCode !== 200) {
      throw new Error(response.body);
    }
    return JSON.parse(response.body);
  }

  async query(rawSql: string): Promise<Values[]> {
    const response = await got.get(`${this.address}/v0/raw`, {
      body: rawSql,
      throwHttpErrors: false,
    });
    if (response.statusCode !== 200) {
      throw new Error(response.body);
    }
    return JSON.parse(response.body);
  }
}

export class Client {
  constructor(public readonly address: string) {}

  async query(name: string, params: Values): Promise<Values[]> {
    const response = await got.get(`${this.address}/v0/named/${name}`, {
      json: params,
      allowGetBody: true,
      throwHttpErrors: false,
    });
    if (response.statusCode !== 200) {
      throw new Error(response.body);
    }
    return JSON.parse(response.body);
  }
}

export type Value = number | string;
export type Values = { [key: string]: Value };

export interface Policy {
  readonly queries: QueryPolicy[];
  readonly mutations: MutationPolicy[];
}

export interface QueryPolicy {
  readonly name: string;
  readonly rawSql: string;
}
export interface MutationPolicy {
  readonly name: string;
  readonly rawSql: string;
}
