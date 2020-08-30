import got, { Got } from "got";

export class AdminClient {
  private client: Got;
  constructor(address: string) {
    this.client = got.extend({
      prefixUrl: address,
      headers: { authorization: "Bearer admin" },
      throwHttpErrors: false,
      allowGetBody: true,
    });
  }

  async setPolicy(policy: Policy): Promise<void> {
    const response = await this.client.put(`v0/policy`, { json: policy });
    if (response.statusCode !== 200) {
      throw new ApiError(response.statusCode, response.body);
    }
    return JSON.parse(response.body);
  }

  async mutate(rawSql: string): Promise<void> {
    const response = await this.client.post(`v0/raw`, { body: rawSql });
    if (response.statusCode !== 200) {
      throw new ApiError(response.statusCode, response.body);
    }
    return JSON.parse(response.body);
  }

  async query(rawSql: string): Promise<Values[]> {
    const response = await this.client.get(`v0/raw`, { body: rawSql });
    if (response.statusCode !== 200) {
      throw new ApiError(response.statusCode, response.body);
    }
    return JSON.parse(response.body);
  }
}

export class Client {
  private client: Got;
  constructor(address: string) {
    this.client = got.extend({
      prefixUrl: address,
      headers: { authorization: "Bearer admin" },
      throwHttpErrors: false,
      allowGetBody: true,
    });
  }

  async query(name: string, params: Values): Promise<Values[]> {
    const response = await this.client.get(`v0/named/${name}`, {
      json: params,
    });
    if (response.statusCode !== 200) {
      throw new ApiError(response.statusCode, response.body);
    }
    return JSON.parse(response.body);
  }

  async mutate(name: string, params: Values): Promise<void> {
    const response = await this.client.post(`v0/named/${name}`, {
      json: params,
    });
    if (response.statusCode !== 200) {
      throw new ApiError(response.statusCode, response.body);
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

class ApiError extends Error {
  constructor(statusCode: number, body: string) {
    super(body);
    this.name = statusCode.toString();
    Error.captureStackTrace(this, ApiError);
  }
}
