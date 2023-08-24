import axios, { AxiosInstance, AxiosResponse } from "axios";
import { FormikErrors } from "formik";

export interface User {
  email: string;
  email_verified: boolean;
  name: string;
  nickname: string;
  picture: string;
  sub: string;
  updated_at: string;
  admin: boolean;
}

export interface Account {
  name: string;
  id: string;
  created_at: string;
  updated_at: string;
}
export interface PartialAccount {
  name: string;
}

export interface Membership {
  user_email: string;
  account_id: string;
  id: string;
  created_at: string;
}

export interface QueueJob {
  id: string;
  created_at: string;
  updated_at: string;
  scheduled_at: string | null;
  failure_count: number;
  status: "Success" | "Pending" | "Failed";
  job: {
    type: string;
    version: string;
    [key: string]: unknown;
  };
  error_message: { [key: string]: unknown };
  child_id: string | null;
  parent_id: string | null;
}

type VdafDefinition =
  | { type: "sum"; bits: number }
  | { type: "count" }
  | { type: "histogram"; buckets: number[] };

export interface Task {
  id: string;
  name: string;
  leader_aggregator_id: string;
  helper_aggregator_id: string;
  vdaf: VdafDefinition;
  min_batch_size: number;
  time_precision_seconds: number;
  account_id: string;
  created_at: string;
  updated_at: string;
  expiration: string | null;
  max_batch_size: number | null;
  report_count: number;
  aggregate_collection_count: number;
  hpke_config_id: string;
}

export interface CollectorAuthToken {
  type: string;
  token: string;
}

export type NewTask = Omit<
  Task,
  | "id"
  | "report_count"
  | "aggregate_collection_count"
  | "account_id"
  | "created_at"
  | "updated_at"
  | "vdaf"
  | "expiration"
> & {
  vdaf: {
    type: "sum" | "count" | "histogram";
    bits?: number;
    buckets?: number[];
  };
};

export interface UpdateTask {
  name: string;
}

export interface CreateMembership {
  user_email: string;
}

export type Role =
  | "Leader"
  | "Helper"
  | "Either"
  | "leader"
  | "helper"
  | "either";

export interface Aggregator {
  id: string;
  account_id: string | null;
  created_at: string;
  updated_at: string;
  deleted_at: string | null;
  api_url: string;
  dap_url: string;
  role: Role;
  name: string;
  is_first_party: boolean;
  vdafs: string[];
  query_types: string[];
}

export interface NewAggregator {
  name: string;
  api_url: string;
  bearer_token: string;
  is_first_party?: boolean;
}

export interface UpdateAggregator {
  name?: string;
  bearer_token?: string;
}

export interface ApiToken {
  id: string;
  account_id: string;
  token_hash: string;
  created_at: string;
  deleted_at?: string;
  name?: string;
  last_used_at?: string;
}

export interface HpkeConfig {
  id: string;
  contents: {
    id: number;
    kem_id: string;
    kdf_id: string;
    aead_id: string;
    public_key: string;
  };
  created_at: string;
  deleted_at: null | string;
  updated_at: string;
  name: null | string;
}

const mime = "application/vnd.divviup+json;version=0.1";

export class ApiClient {
  private client?: Promise<AxiosInstance> | AxiosInstance;

  static async fetchBaseUrl(): Promise<URL> {
    const url = new URL(window.location.href);
    url.pathname = "/api_url";
    const contents = await axios.get(url.toString());
    return new URL(await contents.data);
  }

  private async buildClient(): Promise<AxiosInstance> {
    const baseUrl = await ApiClient.fetchBaseUrl();
    return axios.create({
      baseURL: baseUrl.toString(),
      withCredentials: true,
      headers: {
        "Content-Type": mime,
        Accept: mime,
      },
      validateStatus(status) {
        return (status >= 200 && status < 300) || status == 400;
      },
    });
  }

  private async populateClient(): Promise<AxiosInstance> {
    if (!this.client) {
      this.client = this.buildClient();
    }
    return this.client;
  }

  async loginUrl(): Promise<string> {
    return (await this.populateClient()).getUri({ url: "/login" });
  }

  async redirectToLogin(): Promise<null> {
    const loginUrl = await this.loginUrl();
    window.location.href = loginUrl;
    return null;
  }

  async logoutUrl(): Promise<string> {
    return (await this.populateClient()).getUri({ url: "/logout" });
  }

  async getCurrentUser(): Promise<User> {
    const res = await this.get("/api/users/me");
    return res.data as User;
  }

  private async get(path: string): Promise<AxiosResponse> {
    const client = await this.populateClient();
    return client.get(path);
  }

  private async post(path: string, body?: unknown): Promise<AxiosResponse> {
    const client = await this.populateClient();
    return client.post(path, body);
  }

  private async delete(path: string): Promise<AxiosResponse> {
    const client = await this.populateClient();
    return client.delete(path);
  }

  private async patch(path: string, body: unknown): Promise<AxiosResponse> {
    const client = await this.populateClient();
    return client.patch(path, body);
  }

  async accounts(): Promise<Account[]> {
    const res = await this.get("/api/accounts");
    return res.data as Account[];
  }

  async account(id: string): Promise<Account> {
    const res = await this.get(`/api/accounts/${id}`);
    return res.data as Account;
  }

  async createAccount(account: PartialAccount): Promise<Account> {
    const res = await this.post("/api/accounts", account);
    return res.data as Account;
  }

  async updateAccount(id: string, account: PartialAccount): Promise<Account> {
    const res = await this.patch(`/api/accounts/${id}`, account);
    return res.data as Account;
  }

  async accountMemberships(accountId: string): Promise<Membership[]> {
    const res = await this.get(`/api/accounts/${accountId}/memberships`);
    return res.data as Membership[];
  }

  async createMembership(
    accountId: string,
    membership: CreateMembership,
  ): Promise<Membership | { error: ValidationErrorsFor<CreateMembership> }> {
    const res = await this.post(
      `/api/accounts/${accountId}/memberships`,
      membership,
    );
    return res.data as Membership;
  }

  async accountTasks(accountId: string): Promise<Task[]> {
    const res = await this.get(`/api/accounts/${accountId}/tasks`);
    return res.data as Task[];
  }

  async task(taskId: string): Promise<Task> {
    const res = await this.get(`/api/tasks/${taskId}`);
    return res.data as Task;
  }

  async taskCollectorAuthTokens(taskId: string): Promise<CollectorAuthToken[]> {
    const res = await this.get(`/api/tasks/${taskId}/collector_auth_tokens`);
    return res.data as CollectorAuthToken[];
  }

  async deleteMembership(membershipId: string): Promise<null> {
    await this.delete(`/api/memberships/${membershipId}`);
    return null;
  }

  async accountAggregators(accountId: string): Promise<Aggregator[]> {
    const res = await this.get(`/api/accounts/${accountId}/aggregators`);
    return res.data as Aggregator[];
  }

  async createAggregator(
    accountId: string,
    aggregator: NewAggregator,
  ): Promise<Aggregator | { error: ValidationErrorsFor<NewAggregator> }> {
    const res = await this.post(
      `/api/accounts/${accountId}/aggregators`,
      aggregator,
    );
    switch (res.status) {
      case 201:
        return res.data as Aggregator;
      case 400:
        return { error: res.data } as {
          error: ValidationErrorsFor<NewAggregator>;
        };
      default:
        throw res;
    }
  }

  async updateAggregator(
    aggregatorId: string,
    aggregator: UpdateAggregator,
  ): Promise<Aggregator | { error: ValidationErrorsFor<UpdateAggregator> }> {
    const res = await this.patch(
      `/api/aggregators/${aggregatorId}`,
      aggregator,
    );
    switch (res.status) {
      case 200:
        return res.data as Aggregator;
      case 400:
        return { error: res.data } as {
          error: ValidationErrorsFor<UpdateAggregator>;
        };
      default:
        throw res;
    }
  }

  async deleteAggregator(aggregatorId: string): Promise<null> {
    await this.delete(`/api/aggregators/${aggregatorId}`);
    return null;
  }

  async aggregator(aggregatorId: string): Promise<Aggregator> {
    const res = await this.get(`/api/aggregators/${aggregatorId}`);
    return res.data as Aggregator;
  }

  async sharedAggregators(): Promise<Aggregator[]> {
    const res = await this.get("/api/aggregators");
    return res.data as Aggregator[];
  }

  async createSharedAggregator(
    aggregator: NewAggregator,
  ): Promise<Aggregator | { error: ValidationErrorsFor<NewAggregator> }> {
    const res = await this.post(`/api/aggregators`, aggregator);
    switch (res.status) {
      case 201:
        return res.data as Aggregator;
      case 400:
        return { error: res.data } as {
          error: ValidationErrorsFor<NewAggregator>;
        };
      default:
        throw res;
    }
  }

  async updateTask(
    taskId: string,
    task: UpdateTask,
  ): Promise<Task | { error: ValidationErrorsFor<UpdateTask> }> {
    const res = await this.patch(`/api/tasks/${taskId}`, task);
    switch (res.status) {
      case 201:
        return res.data as Task;
      case 400:
        return { error: res.data } as {
          error: ValidationErrorsFor<UpdateTask>;
        };
      default:
        throw res;
    }
  }

  async createTask(
    accountId: string,
    task: NewTask,
  ): Promise<Task | { error: ValidationErrorsFor<NewTask> }> {
    const res = await this.post(`/api/accounts/${accountId}/tasks`, task);
    switch (res.status) {
      case 201:
        return res.data as Task;
      case 400:
        return { error: res.data } as { error: ValidationErrorsFor<NewTask> };
      default:
        throw res;
    }
  }

  async accountApiTokens(accountId: string): Promise<ApiToken[]> {
    const res = await this.get(`/api/accounts/${accountId}/api_tokens`);
    return res.data as ApiToken[];
  }

  async createApiToken(
    accountId: string,
  ): Promise<ApiToken & { token: string }> {
    const res = await this.post(`/api/accounts/${accountId}/api_tokens`);
    return res.data as ApiToken & { token: string };
  }

  async deleteApiToken(tokenId: string): Promise<null> {
    await this.delete(`/api/api_tokens/${tokenId}`);
    return null;
  }

  async updateApiToken(
    tokenId: string,
    token: { name: string },
  ): Promise<null> {
    await this.patch(`/api/api_tokens/${tokenId}`, token);
    return null;
  }

  async queue(searchParams: URLSearchParams): Promise<QueueJob[]> {
    const res = await this.get(`/api/admin/queue?${searchParams}`);
    return res.data as QueueJob[];
  }

  async queueJob(id: string): Promise<QueueJob> {
    const res = await this.get(`/api/admin/queue/${id}`);
    return res.data as QueueJob;
  }

  async deleteHpkeConfig(hpkeConfigId: string) {
    await this.delete(`/api/hpke_configs/${hpkeConfigId}`);
    return null;
  }

  async updateHpkeConfig(hpkeConfigId: string, hpkeConfig: { name: string }) {
    await this.patch(`/api/hpke_configs/${hpkeConfigId}`, hpkeConfig);
    return null;
  }

  async hpkeConfig(hpkeConfigId: string): Promise<HpkeConfig> {
    const res = await this.get(`/api/hpke_configs/${hpkeConfigId}`);
    return res.data as HpkeConfig;
  }

  async createHpkeConfig(
    accountId: string,
    hpkeConfig: { contents: string; name: string },
  ): Promise<
    | HpkeConfig
    | { error: ValidationErrorsFor<{ contents: string; name: string }> }
  > {
    const res = await this.post(
      `/api/accounts/${accountId}/hpke_configs`,
      hpkeConfig,
    );
    switch (res.status) {
      case 201:
        return res.data as HpkeConfig;
      case 400:
        return { error: res.data } as {
          error: ValidationErrorsFor<{ contents: string; name: string }>;
        };
      default:
        throw res;
    }
  }

  async accountHpkeConfigs(accountId: string): Promise<HpkeConfig[]> {
    const res = await this.get(`/api/accounts/${accountId}/hpke_configs`);
    return res.data as HpkeConfig[];
  }
}

function errorToMessage({ message, code, params }: ValidationError) {
  if (message) return message;
  if (code === "required") {
    return "is required";
  } else if (code === "url") {
    return "must be a well-formed url";
  } else if (code === "https-url") {
    return "must be a well-formed https:// url";
  } else if (code === "no-first-party") {
    return "one of the aggregators must be operated by Divvi Up";
  } else if (code === "base64") {
    return "must be base64";
  } else if (code === "same") {
    return "must not be the same";
  } else if (code === "token-not-recognized") {
    return "bearer token not recognized";
  } else if (code === "http-error") {
    return "error connecting to url";
  } else if (code === "enum" && Array.isArray(params.values)) {
    return `must be one of these values: ${params.values.join(", ")}`;
  } else if (code === "length") {
    if ("min" in params && "max" in params)
      return `length must be between ${params.min} and ${params.max}`;
    if ("min" in params) return `length must be greater than ${params.min}`;
    if ("max" in params) return `length must be less than ${params.max}`;
  } else if (code === "range") {
    if ("min" in params) return `must be greater than ${params.min}`;
    if ("max" in params) return `must be less than ${params.max}`;
  } else {
    console.log({ code, params });
    return code;
  }
}

function errorsToMessage(validationErrors: ValidationError[]) {
  return [...new Set(validationErrors.map(errorToMessage))].join(", ");
}

function formikErrors_(v: ValidationErrors): FormikLikeErrors {
  const o = {} as FormikLikeErrors;
  for (const key in v) {
    const e = v[key] as ValidationError[] | ValidationErrors;
    if (Array.isArray(e)) {
      o[key] = errorsToMessage(e);
    } else if (typeof e === "object" && e !== null) {
      o[key] = formikErrors_(e);
    }
  }

  return o;
}

export function formikErrors<T extends object>(
  v: ValidationErrorsFor<T>,
): FormikErrors<T> {
  return formikErrors_(v as ValidationErrors) as unknown as FormikErrors<T>;
}

export interface ValidationErrors {
  [key: string]: ValidationError[] | ValidationErrors;
}

export interface FormikLikeErrors {
  [key: string]: string | FormikLikeErrors;
}

export type ValidationErrorsFor<T extends object> = {
  [K in keyof T]?: T[K] extends object
    ? ValidationErrorsFor<T[K]>
    : ValidationError[];
};

export interface ValidationError {
  code: string;
  message: null | string;
  params: { [key: string]: unknown };
}

export default ApiClient;
